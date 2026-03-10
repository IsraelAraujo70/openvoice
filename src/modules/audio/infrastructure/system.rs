use crate::modules::audio::domain::{AudioSourceKind, CapturedAudio, CapturedTrack};
use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

const SYSTEM_SAMPLE_RATE: u32 = 48_000;
const SYSTEM_CHANNELS: u16 = 2;

type SharedSamples = Arc<Mutex<Vec<f32>>>;
type SharedError = Arc<Mutex<Option<String>>>;

pub struct Recorder {
    child: Child,
    samples: SharedSamples,
    last_error: SharedError,
    reader_thread: Option<JoinHandle<()>>,
    device_name: String,
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

pub fn start_default_recording() -> Result<Recorder, String> {
    let default_sink = default_sink_name()?;
    let monitor_source = format!("{default_sink}.monitor");

    let mut child = Command::new("parec")
        .args([
            "--device",
            &monitor_source,
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
        })?;

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
    use super::parse_default_sink_from_info;

    #[test]
    fn parses_default_sink_from_pactl_info() {
        let info = "Server String: /run/user/1000/pulse/native\nDefault Sink: alsa_output.pci-0000_0c_00.4.analog-stereo\n";

        assert_eq!(
            parse_default_sink_from_info(info).as_deref(),
            Some("alsa_output.pci-0000_0c_00.4.analog-stereo")
        );
    }
}
