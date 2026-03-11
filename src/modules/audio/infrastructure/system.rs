#![allow(dead_code)]

use crate::modules::audio::domain::{AudioSourceKind, CapturedAudio, CapturedTrack};
use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

const SYSTEM_SAMPLE_RATE: u32 = 48_000;
const SYSTEM_CHANNELS: u16 = 2;
const LIVE_TARGET_SAMPLE_RATE: u32 = 24_000;
const LIVE_CHUNK_BYTES: usize = 4_800;

type SharedSamples = Arc<Mutex<Vec<f32>>>;
type SharedError = Arc<Mutex<Option<String>>>;

pub struct Recorder {
    child: Child,
    samples: SharedSamples,
    last_error: SharedError,
    reader_thread: Option<JoinHandle<()>>,
    device_name: String,
}

pub struct LiveStream {
    child: Child,
    last_error: SharedError,
    worker: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn finish(mut self) -> Result<CapturedTrack, String> {
        let _ = self.child.kill();
        let _ = self.child.wait();

        if let Some(handle) = self.reader_thread.take() {
            handle.join().map_err(|_| {
                String::from("A leitura do audio do sistema terminou de forma inesperada.")
            })?;
        }

        if let Some(error) = self
            .last_error
            .lock()
            .map_err(|_| String::from("Nao foi possivel ler o estado do audio do sistema."))?
            .clone()
        {
            return Err(error);
        }

        let samples = self
            .samples
            .lock()
            .map_err(|_| String::from("Nao foi possivel finalizar a captura do audio do sistema."))?
            .clone();

        Ok(CapturedTrack {
            source: AudioSourceKind::SystemMonitor,
            device_name: self.device_name,
            audio: CapturedAudio {
                samples,
                sample_rate: SYSTEM_SAMPLE_RATE,
                channels: SYSTEM_CHANNELS,
            },
        })
    }
}

impl LiveStream {
    pub fn stop(mut self) -> Result<(), String> {
        let _ = self.child.kill();
        let _ = self.child.wait();

        if let Some(handle) = self.worker.take() {
            handle.join().map_err(|_| {
                String::from("A leitura do audio realtime terminou de forma inesperada.")
            })?;
        }

        if let Some(error) = self
            .last_error
            .lock()
            .map_err(|_| String::from("Nao foi possivel ler o estado do audio realtime."))?
            .clone()
        {
            return Err(error);
        }

        Ok(())
    }
}

pub fn start_default_recording() -> Result<Recorder, String> {
    let monitor_source = default_monitor_source_name()?;

    let mut child = spawn_parec_process(&monitor_source)?;

    let stdout = child.stdout.take().ok_or_else(|| {
        String::from("Falha ao abrir a saida do processo de captura do audio do sistema.")
    })?;
    let samples = Arc::new(Mutex::new(Vec::new()));
    let last_error = Arc::new(Mutex::new(None));
    let reader_thread = spawn_reader_thread(stdout, Arc::clone(&samples), Arc::clone(&last_error));

    Ok(Recorder {
        child,
        samples,
        last_error,
        reader_thread: Some(reader_thread),
        device_name: monitor_source,
    })
}

pub fn start_default_live_stream(chunk_sender: Sender<Vec<u8>>) -> Result<LiveStream, String> {
    let monitor_source = default_monitor_source_name()?;
    let mut child = spawn_parec_process(&monitor_source)?;
    let stdout = child.stdout.take().ok_or_else(|| {
        String::from("Falha ao abrir a saida do processo realtime do audio do sistema.")
    })?;
    let last_error = Arc::new(Mutex::new(None));
    let worker = spawn_live_reader_thread(stdout, Arc::clone(&last_error), chunk_sender);

    Ok(LiveStream {
        child,
        last_error,
        worker: Some(worker),
    })
}

fn spawn_parec_process(monitor_source: &str) -> Result<Child, String> {
    Command::new("parec")
        .args([
            "--device",
            monitor_source,
            "--format=s16le",
            "--rate=48000",
            "--channels=2",
            "--raw",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            format!("Falha ao iniciar a captura do audio do sistema com parec: {error}")
        })
}

fn default_monitor_source_name() -> Result<String, String> {
    if let Some(source) = monitor_source_override()? {
        eprintln!("[openvoice][audio] using monitor source override: {source}");
        return Ok(source);
    }

    let default_sink = default_sink_name()?;
    let preferred = format!("{default_sink}.monitor");
    let available = list_monitor_sources()?;

    if available.iter().any(|source| source.name == preferred) {
        eprintln!("[openvoice][audio] using default sink monitor: {preferred}");
        return Ok(preferred);
    }

    let fallback = available
        .iter()
        .find(|source| source.state == "RUNNING")
        .or_else(|| available.first())
        .map(|source| source.name.clone())
        .ok_or_else(|| {
            String::from("Nao encontrei nenhum monitor source do PulseAudio/PipeWire para capturar o audio do sistema.")
        })?;

    eprintln!(
        "[openvoice][audio] default sink monitor {preferred} not found, falling back to {fallback}"
    );
    Ok(fallback)
}

fn spawn_reader_thread(
    mut stdout: ChildStdout,
    samples: SharedSamples,
    last_error: SharedError,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];

        loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    if let Ok(mut slot) = samples.lock() {
                        for chunk in buffer[..read].chunks_exact(2) {
                            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                            slot.push(sample as f32 / i16::MAX as f32);
                        }
                    } else {
                        write_error(
                            &last_error,
                            "Nao foi possivel armazenar samples do audio do sistema.",
                        );
                        break;
                    }
                }
                Err(error) => {
                    write_error(
                        &last_error,
                        &format!("Falha ao ler o stream do audio do sistema: {error}"),
                    );
                    break;
                }
            }
        }
    })
}

fn spawn_live_reader_thread(
    mut stdout: ChildStdout,
    last_error: SharedError,
    chunk_sender: Sender<Vec<u8>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut read_buffer = [0_u8; 4096];
        let mut pending = Vec::new();
        let mut output_chunk = Vec::with_capacity(LIVE_CHUNK_BYTES);
        let mut pending_mono_sample = None;

        loop {
            match stdout.read(&mut read_buffer) {
                Ok(0) => break,
                Ok(read) => {
                    pending.extend_from_slice(&read_buffer[..read]);
                    process_live_audio_bytes(
                        &mut pending,
                        &mut output_chunk,
                        &mut pending_mono_sample,
                        &chunk_sender,
                    );
                }
                Err(error) => {
                    write_error(
                        &last_error,
                        &format!("Falha ao ler o stream realtime do audio do sistema: {error}"),
                    );
                    break;
                }
            }
        }

        if !output_chunk.is_empty() {
            let _ = chunk_sender.send(output_chunk);
        }
    })
}

fn process_live_audio_bytes(
    pending: &mut Vec<u8>,
    output_chunk: &mut Vec<u8>,
    pending_mono_sample: &mut Option<i16>,
    chunk_sender: &Sender<Vec<u8>>,
) {
    let mut offset = 0;
    while pending.len().saturating_sub(offset) >= 4 {
        let left = i16::from_le_bytes([pending[offset], pending[offset + 1]]) as i32;
        let right = i16::from_le_bytes([pending[offset + 2], pending[offset + 3]]) as i32;
        let mono = ((left + right) / 2) as i16;

        if let Some(previous) = pending_mono_sample.take() {
            // Average each pair of 48 kHz mono samples before decimating to 24 kHz.
            // This is a small low-pass step that preserves clarity better than
            // dropping every other sample outright.
            let filtered = ((previous as i32 + mono as i32) / 2) as i16;
            output_chunk.extend_from_slice(&filtered.to_le_bytes());

            if output_chunk.len() >= LIVE_CHUNK_BYTES
                && chunk_sender.send(std::mem::take(output_chunk)).is_err()
            {
                return;
            }
        } else {
            *pending_mono_sample = Some(mono);
        }
        offset += 4;
    }

    if offset > 0 {
        pending.drain(..offset);
    }
}

fn default_sink_name() -> Result<String, String> {
    if let Some(sink) = read_command_output("pactl", &["get-default-sink"])? {
        return Ok(sink);
    }

    let info = read_command_output("pactl", &["info"])?.ok_or_else(|| {
        String::from("O comando pactl nao retornou informacoes do servidor PulseAudio/PipeWire.")
    })?;

    parse_default_sink_from_info(&info).ok_or_else(|| {
        String::from(
            "Nao consegui descobrir o sink padrao do desktop. Verifique PulseAudio/pipewire-pulse.",
        )
    })
}

fn monitor_source_override() -> Result<Option<String>, String> {
    let Some(raw) = std::env::var("OPENVOICE_MONITOR_SOURCE").ok() else {
        return Ok(None);
    };

    let source = raw.trim();
    if source.is_empty() {
        return Ok(None);
    }

    let available = list_monitor_sources()?;
    if available.iter().any(|candidate| candidate.name == source) {
        Ok(Some(source.to_owned()))
    } else {
        Err(format!(
            "O monitor source definido em OPENVOICE_MONITOR_SOURCE nao existe: {source}"
        ))
    }
}

fn list_monitor_sources() -> Result<Vec<MonitorSourceInfo>, String> {
    let output = read_command_output("pactl", &["list", "short", "sources"])?.ok_or_else(|| {
        String::from("Nao consegui listar os monitor sources do PulseAudio/PipeWire.")
    })?;

    let sources = output
        .lines()
        .filter_map(parse_monitor_source_line)
        .collect::<Vec<_>>();

    Ok(sources)
}

#[derive(Debug, Clone)]
struct MonitorSourceInfo {
    name: String,
    state: String,
}

fn parse_monitor_source_line(line: &str) -> Option<MonitorSourceInfo> {
    let mut parts = line.split('\t');
    let _index = parts.next()?;
    let name = parts.next()?.trim();
    let _driver = parts.next()?;
    let _format = parts.next()?;
    let state = parts.next()?.trim();

    name.ends_with(".monitor").then(|| MonitorSourceInfo {
        name: name.to_owned(),
        state: state.to_owned(),
    })
}

fn read_command_output(command: &str, args: &[&str]) -> Result<Option<String>, String> {
    let output = match Command::new(command).args(args).output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "Preciso do comando {command} instalado para capturar o audio do sistema."
            ));
        }
        Err(error) => return Err(format!("Falha ao executar {command}: {error}")),
    };

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("Falha ao ler saida de {command}: {error}"))?;
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_owned()))
    }
}

fn parse_default_sink_from_info(info: &str) -> Option<String> {
    info.lines()
        .find_map(|line| line.strip_prefix("Default Sink:"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn write_error(last_error: &SharedError, error: &str) {
    if let Ok(mut slot) = last_error.lock() {
        *slot = Some(error.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LIVE_CHUNK_BYTES, parse_default_sink_from_info, parse_monitor_source_line,
        process_live_audio_bytes,
    };
    use std::sync::mpsc;

    #[test]
    fn parses_default_sink_from_pactl_info() {
        let info = "Server String: /run/user/1000/pulse/native\nDefault Sink: alsa_output.pci-0000_0c_00.4.analog-stereo\n";

        assert_eq!(
            parse_default_sink_from_info(info).as_deref(),
            Some("alsa_output.pci-0000_0c_00.4.analog-stereo")
        );
    }

    #[test]
    fn parses_monitor_source_lines() {
        let line = "60\talsa_output.usb-Astro_Gaming_Astro_MixAmp_Pro-00.stereo-chat.monitor\tPipeWire\ts16le 2ch 48000Hz\tRUNNING";
        let source = parse_monitor_source_line(line).expect("monitor source");

        assert_eq!(
            source.name,
            "alsa_output.usb-Astro_Gaming_Astro_MixAmp_Pro-00.stereo-chat.monitor"
        );
        assert_eq!(source.state, "RUNNING");
    }

    #[test]
    fn converts_stereo_48khz_to_mono_24khz() {
        let (tx, rx) = mpsc::channel();
        let mut pending = Vec::new();
        let mut output_chunk = Vec::new();
        let mut pending_mono_sample = None;

        for sample in [
            100_i16, 300_i16, 200_i16, 400_i16, 500_i16, 700_i16, 600_i16, 800_i16,
        ] {
            pending.extend_from_slice(&sample.to_le_bytes());
        }

        process_live_audio_bytes(
            &mut pending,
            &mut output_chunk,
            &mut pending_mono_sample,
            &tx,
        );

        assert!(rx.try_recv().is_err());
        assert!(pending.is_empty());
        assert!(pending_mono_sample.is_none());

        let samples = output_chunk
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();

        assert_eq!(samples, vec![250_i16, 650_i16]);
    }

    #[test]
    fn flushes_live_audio_chunks_with_lower_latency_budget() {
        let (tx, rx) = mpsc::channel();
        let mut pending = Vec::new();
        let mut output_chunk = Vec::new();
        let mut pending_mono_sample = None;
        let mut flushed = None;

        for _ in 0..64 {
            for _ in 0..256 {
                pending.extend_from_slice(&2000_i16.to_le_bytes());
                pending.extend_from_slice(&2000_i16.to_le_bytes());
            }
            process_live_audio_bytes(
                &mut pending,
                &mut output_chunk,
                &mut pending_mono_sample,
                &tx,
            );

            if let Ok(chunk) = rx.try_recv() {
                flushed = Some(chunk);
                break;
            }
        }

        let flushed = flushed.expect("chunk should flush once threshold is reached");
        assert_eq!(flushed.len(), LIVE_CHUNK_BYTES);
        assert!(output_chunk.len() < LIVE_CHUNK_BYTES);
    }
}
