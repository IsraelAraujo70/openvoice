use crate::modules::audio::infrastructure::system;
use crate::modules::live_transcription::domain::{LiveTranscriptionConfig, RuntimeEvent};
use base64::Engine;
use serde_json::{json, Value};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

pub type SharedReceiver = Arc<Mutex<Receiver<RuntimeEvent>>>;

pub struct SessionHandle {
    receiver: SharedReceiver,
    stop_flag: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
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

        match audio_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(chunk) => {
                let payload = json!({
                    "type": "input_audio_buffer.append",
                    "audio": base64::engine::general_purpose::STANDARD.encode(&chunk),
                });

                if let Err(error) = socket.send(Message::Text(payload.to_string())) {
                    let _ = event_tx.send(RuntimeEvent::Error(format!(
                        "Falha ao enviar audio para o realtime: {error}"
                    )));
                    break;
                }
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
            Ok(message) => handle_server_message(message, &event_tx),
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
    let _ = event_tx.send(RuntimeEvent::Stopped);
}

fn build_session_update(config: &LiveTranscriptionConfig) -> Value {
    let mut transcription = json!({ "model": config.model });

    if let Some(language) = config.language.as_deref() {
        transcription["language"] = Value::String(language.to_owned());
    }

    if let Some(prompt) = config.prompt.as_deref() {
        transcription["prompt"] = Value::String(prompt.to_owned());
    }

    json!({
        "type": "transcription_session.update",
        "session": {
            "input_audio_format": "pcm16",
            "input_audio_noise_reduction": {
                "type": "far_field",
            },
            "input_audio_transcription": transcription,
            "turn_detection": {
                "type": "server_vad",
                "threshold": 0.5,
                "prefix_padding_ms": 300,
                "silence_duration_ms": 500,
            },
            "include": [
                "item.input_audio_transcription.logprobs"
            ]
        }
    })
}

fn handle_server_message(message: Message, event_tx: &Sender<RuntimeEvent>) {
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
        "transcription_session.updated" | "session.updated" => {
            let _ = event_tx.send(RuntimeEvent::Warning(String::from(
                "Sessao realtime atualizada pelo servidor.",
            )));
        }
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
    std::env::var("OPENVOICE_LOG_REALTIME_TRANSCRIPTS")
        .ok()
        .as_deref()
        .map(|value| matches!(value, "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

fn configure_stream_timeout(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> Result<(), String> {
    let stream = socket.get_mut();
    let timeout = Some(Duration::from_millis(50));

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
