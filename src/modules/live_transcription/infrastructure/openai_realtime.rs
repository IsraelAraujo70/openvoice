use crate::modules::audio::infrastructure::system;
use crate::modules::live_transcription::domain::{
    LiveTranscriptionConfig, NoiseReductionMode, RuntimeEvent, TurnDetectionMode,
};
use base64::Engine;
use serde_json::{Value, json};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket, connect};

pub type SharedReceiver = Arc<Mutex<Receiver<RuntimeEvent>>>;

const SOCKET_TIMEOUT_MS: u64 = 20;

pub struct SessionHandle {
    receiver: SharedReceiver,
    stop_flag: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

struct RealtimeTelemetry {
    enabled: bool,
    session_started_at: Instant,
    first_audio_sent_at: Option<Instant>,
    first_delta_at: Option<Instant>,
    first_completed_at: Option<Instant>,
    audio_chunks_sent: usize,
    completed_segments: usize,
}

impl SessionHandle {
    pub fn receiver(&self) -> SharedReceiver {
        Arc::clone(&self.receiver)
    }

    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);

        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

impl RealtimeTelemetry {
    fn new() -> Self {
        Self {
            enabled: flag_from_env("OPENVOICE_LOG_REALTIME_METRICS"),
            session_started_at: Instant::now(),
            first_audio_sent_at: None,
            first_delta_at: None,
            first_completed_at: None,
            audio_chunks_sent: 0,
            completed_segments: 0,
        }
    }

    fn mark_audio_chunk_sent(&mut self, chunk_len: usize) {
        self.audio_chunks_sent += 1;

        if self.first_audio_sent_at.is_none() {
            let now = Instant::now();
            self.first_audio_sent_at = Some(now);
            self.log(format!(
                "first_audio_chunk_ms={} bytes={chunk_len}",
                now.duration_since(self.session_started_at).as_millis()
            ));
        }
    }

    fn mark_delta(&mut self, delta_len: usize) {
        if self.first_delta_at.is_none() {
            let now = Instant::now();
            self.first_delta_at = Some(now);
            self.log(format!(
                "first_delta_ms={} chars={delta_len}",
                now.duration_since(self.session_started_at).as_millis()
            ));
        }
    }

    fn mark_completed(&mut self, transcript_len: usize) {
        self.completed_segments += 1;

        if self.first_completed_at.is_none() {
            let now = Instant::now();
            self.first_completed_at = Some(now);
            self.log(format!(
                "first_completed_ms={} chars={transcript_len}",
                now.duration_since(self.session_started_at).as_millis()
            ));
        }
    }

    fn finish(&self) {
        self.log(format!(
            "session_ms={} chunks_sent={} completed_segments={}",
            self.session_started_at.elapsed().as_millis(),
            self.audio_chunks_sent,
            self.completed_segments
        ));
    }

    fn log(&self, message: String) {
        if self.enabled {
            eprintln!("[openvoice][realtime][metrics] {message}");
        }
    }
}

pub fn start_session(config: LiveTranscriptionConfig) -> Result<SessionHandle, String> {
    let (event_tx, event_rx) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(event_rx));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let worker_stop_flag = Arc::clone(&stop_flag);

    let worker = thread::spawn(move || run_session(config, worker_stop_flag, event_tx));

    Ok(SessionHandle {
        receiver,
        stop_flag,
        worker: Some(worker),
    })
}

fn run_session(
    config: LiveTranscriptionConfig,
    stop_flag: Arc<AtomicBool>,
    event_tx: Sender<RuntimeEvent>,
) {
    let (audio_tx, audio_rx) = mpsc::channel();
    let mut telemetry = RealtimeTelemetry::new();
    let audio_stream = match system::start_default_live_stream(audio_tx) {
        Ok(stream) => stream,
        Err(error) => {
            let _ = event_tx.send(RuntimeEvent::Error(error));
            return;
        }
    };

    let url = String::from("wss://api.openai.com/v1/realtime?intent=transcription");
    let mut request = match tungstenite::client::IntoClientRequest::into_client_request(url) {
        Ok(request) => request,
        Err(error) => {
            let _ = event_tx.send(RuntimeEvent::Error(format!(
                "Falha ao criar request do realtime: {error}"
            )));
            let _ = audio_stream.stop();
            return;
        }
    };

    let authorization = match format!("Bearer {}", config.bearer_token()).parse() {
        Ok(value) => value,
        Err(error) => {
            let _ = event_tx.send(RuntimeEvent::Error(format!(
                "Falha ao montar o header Authorization: {error}"
            )));
            let _ = audio_stream.stop();
            return;
        }
    };

    request.headers_mut().insert("Authorization", authorization);
    request.headers_mut().insert(
        "OpenAI-Beta",
        "realtime=v1".parse().expect("valid beta header"),
    );

    let (mut socket, _) = match connect(request) {
        Ok(connection) => connection,
        Err(error) => {
            let _ = event_tx.send(RuntimeEvent::Error(format!(
                "Falha ao conectar ao OpenAI Realtime: {error}"
            )));
            let _ = audio_stream.stop();
            return;
        }
    };

    if let Err(error) = configure_stream_timeout(&mut socket) {
        let _ = event_tx.send(RuntimeEvent::Warning(error));
    }

    let session_update = build_session_update(&config);
    if let Err(error) = socket.send(Message::Text(session_update.to_string())) {
        let _ = event_tx.send(RuntimeEvent::Error(format!(
            "Falha ao configurar a sessao realtime: {error}"
        )));
        let _ = audio_stream.stop();
        return;
    }

    let _ = event_tx.send(RuntimeEvent::Connected);

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            let _ = socket.close(None);
            break;
        }

        match audio_rx.recv_timeout(Duration::from_millis(SOCKET_TIMEOUT_MS)) {
            Ok(chunk) => {
                if let Err(error) = send_audio_chunk(&mut socket, &chunk) {
                    let _ = event_tx.send(RuntimeEvent::Error(error));
                    break;
                }
                telemetry.mark_audio_chunk_sent(chunk.len());
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                let _ = event_tx.send(RuntimeEvent::Error(String::from(
                    "A captura de audio do sistema foi interrompida.",
                )));
                break;
            }
        }

        match socket.read() {
            Ok(message) => handle_server_message(message, &event_tx, &mut telemetry),
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(tungstenite::Error::AlreadyClosed) | Err(tungstenite::Error::ConnectionClosed) => {
                break;
            }
            Err(error) => {
                let _ = event_tx.send(RuntimeEvent::Error(format!(
                    "Falha ao ler eventos do realtime: {error}"
                )));
                break;
            }
        }
    }

    let _ = audio_stream.stop();
    telemetry.finish();
    let _ = event_tx.send(RuntimeEvent::Stopped);
}

fn send_audio_chunk(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    chunk: &[u8],
) -> Result<(), String> {
    let payload = json!({
        "type": "input_audio_buffer.append",
        "audio": base64::engine::general_purpose::STANDARD.encode(chunk),
    });

    socket
        .send(Message::Text(payload.to_string()))
        .map_err(|error| format!("Falha ao enviar audio para o realtime: {error}"))
}

fn build_session_update(config: &LiveTranscriptionConfig) -> Value {
    let mut transcription = json!({ "model": config.model });

    if let Some(language) = config.language.as_deref() {
        transcription["language"] = Value::String(language.to_owned());
    }

    if let Some(prompt) = config.prompt.as_deref() {
        transcription["prompt"] = Value::String(prompt.to_owned());
    }

    let mut session = json!({
        "input_audio_format": "pcm16",
        "input_audio_transcription": transcription,
        "include": ["item.input_audio_transcription.logprobs"]
    });

    if let Some(noise_reduction) = build_noise_reduction(config) {
        session["input_audio_noise_reduction"] = noise_reduction;
    }

    session["turn_detection"] = build_turn_detection(config).unwrap_or(Value::Null);

    json!({
        "type": "transcription_session.update",
        "session": session
    })
}

fn build_noise_reduction(config: &LiveTranscriptionConfig) -> Option<Value> {
    match config.noise_reduction {
        Some(NoiseReductionMode::NearField) => Some(json!({ "type": "near_field" })),
        Some(NoiseReductionMode::FarField) => Some(json!({ "type": "far_field" })),
        None => None,
    }
}

fn build_turn_detection(config: &LiveTranscriptionConfig) -> Option<Value> {
    match &config.turn_detection {
        TurnDetectionMode::Disabled => None,
        TurnDetectionMode::ServerVad {
            threshold,
            prefix_padding_ms,
            silence_duration_ms,
        } => Some(json!({
            "type": "server_vad",
            "threshold": quantize_decimal(*threshold, 4),
            "prefix_padding_ms": prefix_padding_ms,
            "silence_duration_ms": silence_duration_ms,
        })),
        TurnDetectionMode::SemanticVad { .. } => Some(json!({
            "type": "server_vad",
            "threshold": 0.45,
            "prefix_padding_ms": 240,
            "silence_duration_ms": 320,
        })),
    }
}

fn quantize_decimal(value: f32, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    ((value as f64) * factor).round() / factor
}

fn handle_server_message(
    message: Message,
    event_tx: &Sender<RuntimeEvent>,
    telemetry: &mut RealtimeTelemetry,
) {
    let Message::Text(text) = message else {
        return;
    };

    let parsed: Value = match serde_json::from_str(&text) {
        Ok(parsed) => parsed,
        Err(error) => {
            let _ = event_tx.send(RuntimeEvent::Warning(format!(
                "Recebi um evento realtime invalido: {error}"
            )));
            return;
        }
    };

    let event_type = parsed
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match event_type {
        "transcription_session.updated" | "session.updated" => {}
        "input_audio_buffer.speech_started" => {
            let _ = event_tx.send(RuntimeEvent::Warning(String::from(
                "VAD detectou inicio de fala.",
            )));
        }
        "input_audio_buffer.speech_stopped" => {
            let _ = event_tx.send(RuntimeEvent::Warning(String::from(
                "VAD detectou fim de fala.",
            )));
        }
        "input_audio_buffer.committed" => {
            let _ = event_tx.send(RuntimeEvent::Warning(String::from(
                "Buffer de audio enviado para transcricao.",
            )));
        }
        "conversation.item.input_audio_transcription.delta" => {
            let item_id = parsed
                .get("item_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let delta = parsed
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();

            if !item_id.is_empty() && !delta.is_empty() {
                telemetry.mark_delta(delta.len());
                if should_log_realtime_deltas() {
                    eprintln!("[openvoice][realtime][delta] {delta}");
                }
                let _ = event_tx.send(RuntimeEvent::TranscriptDelta { item_id, delta });
            }
        }
        "conversation.item.input_audio_transcription.completed" => {
            let item_id = parsed
                .get("item_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let transcript = parsed
                .get("transcript")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();

            if !item_id.is_empty() {
                if !transcript.trim().is_empty() {
                    telemetry.mark_completed(transcript.len());
                }
                if should_log_realtime_transcripts() && !transcript.trim().is_empty() {
                    eprintln!("[openvoice][realtime][transcript] {transcript}");
                }

                let _ = event_tx.send(RuntimeEvent::TranscriptCompleted {
                    item_id,
                    transcript,
                });
            }
        }
        "error" => {
            let message = parsed
                .get("error")
                .and_then(Value::as_object)
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Falha desconhecida no OpenAI Realtime.")
                .to_owned();

            let _ = event_tx.send(RuntimeEvent::Error(message));
        }
        _ => {}
    }
}

fn should_log_realtime_transcripts() -> bool {
    flag_from_env("OPENVOICE_LOG_REALTIME_TRANSCRIPTS")
}

fn should_log_realtime_deltas() -> bool {
    flag_from_env("OPENVOICE_LOG_REALTIME_DELTAS")
}

fn flag_from_env(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .as_deref()
        .map(|value| matches!(value, "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

fn configure_stream_timeout(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> Result<(), String> {
    let stream = socket.get_mut();
    let timeout = Some(Duration::from_millis(SOCKET_TIMEOUT_MS));

    match stream {
        MaybeTlsStream::Plain(tcp) => tcp
            .set_read_timeout(timeout)
            .map_err(|error| format!("Falha ao configurar timeout do socket: {error}")),
        MaybeTlsStream::Rustls(tls) => tls
            .get_mut()
            .set_read_timeout(timeout)
            .map_err(|error| format!("Falha ao configurar timeout do socket TLS: {error}")),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::build_session_update;
    use crate::modules::live_transcription::domain::{
        LiveTranscriptionConfig, NoiseReductionMode, TurnDetectionMode,
    };

    fn base_config() -> LiveTranscriptionConfig {
        LiveTranscriptionConfig {
            bearer_token: String::from("token"),
            model: String::from("gpt-4o-transcribe"),
            prompt: None,
            language: None,
            noise_reduction: None,
            turn_detection: TurnDetectionMode::Disabled,
        }
    }

    #[test]
    fn builds_session_update_with_server_vad_and_prompt() {
        let mut config = base_config();
        config.prompt = Some(String::from("Meeting terms: Kubernetes, Grafana."));
        config.language = Some(String::from("pt"));
        config.noise_reduction = Some(NoiseReductionMode::NearField);
        config.turn_detection = TurnDetectionMode::ServerVad {
            threshold: 0.41,
            prefix_padding_ms: 180,
            silence_duration_ms: 260,
        };

        let payload = build_session_update(&config);

        assert_eq!(payload["type"], "transcription_session.update");
        assert_eq!(
            payload["session"]["input_audio_transcription"]["model"],
            "gpt-4o-transcribe"
        );
        assert_eq!(
            payload["session"]["input_audio_transcription"]["language"],
            "pt"
        );
        assert_eq!(
            payload["session"]["input_audio_transcription"]["prompt"],
            "Meeting terms: Kubernetes, Grafana."
        );
        assert_eq!(
            payload["session"]["input_audio_noise_reduction"]["type"],
            "near_field"
        );
        assert_eq!(payload["session"]["turn_detection"]["type"], "server_vad");
        assert_eq!(
            payload["session"]["turn_detection"]["silence_duration_ms"],
            260
        );
    }

    #[test]
    fn omits_turn_detection_and_noise_reduction_when_disabled() {
        let payload = build_session_update(&base_config());

        assert!(payload["session"]["turn_detection"].is_null());
        assert!(payload["session"]["input_audio_noise_reduction"].is_null());
    }

    #[test]
    fn builds_session_update_with_semantic_vad_fallback() {
        let mut config = base_config();
        config.turn_detection = TurnDetectionMode::SemanticVad {
            eagerness: String::from("high"),
        };
        config.noise_reduction = Some(NoiseReductionMode::FarField);

        let payload = build_session_update(&config);

        assert_eq!(payload["session"]["turn_detection"]["type"], "server_vad");
        assert_eq!(
            payload["session"]["input_audio_noise_reduction"]["type"],
            "far_field"
        );
    }
}
