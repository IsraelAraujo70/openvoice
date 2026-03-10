use crate::modules::audio::domain::{
    CaptureSession, CapturedTrack, SessionArtifacts, SessionMetadata, TrackArtifact,
};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_session_id() -> String {
    format!("session-{}", unix_timestamp_ms())
}

pub fn unix_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub fn persist_session(
    session_id: String,
    started_at_unix_ms: u128,
    finished_at_unix_ms: u128,
    microphone: CapturedTrack,
    system: CapturedTrack,
) -> Result<CaptureSession, String> {
    let session_dir = session_dir(&session_id)?;
    fs::create_dir_all(&session_dir)
        .map_err(|error| format!("Falha ao criar pasta da sessao {}: {error}", session_id))?;

    let microphone_wav = session_dir.join("mic.wav");
    let system_wav = session_dir.join("system.wav");
    write_track_wav(&microphone, &microphone_wav)?;
    write_track_wav(&system, &system_wav)?;

    let artifacts = SessionArtifacts {
        session_dir: session_dir.clone(),
        metadata_path: session_dir.join("metadata.json"),
        microphone_wav: microphone_wav.clone(),
        system_wav: system_wav.clone(),
    };
    let microphone_artifact = TrackArtifact {
        source: microphone.source,
        device_name: microphone.device_name.clone(),
        wav_path: microphone_wav,
        format: microphone.audio.format(),
        duration_seconds: microphone.duration_seconds(),
    };
    let system_artifact = TrackArtifact {
        source: system.source,
        device_name: system.device_name.clone(),
        wav_path: system_wav,
        format: system.audio.format(),
        duration_seconds: system.duration_seconds(),
    };

    let session = CaptureSession {
        session_id,
        started_at_unix_ms,
        finished_at_unix_ms,
        microphone,
        system,
        microphone_artifact,
        system_artifact,
        artifacts,
    };
    let metadata = SessionMetadata::from_session(&session);
    let contents = serde_json::to_string_pretty(&metadata)
        .map_err(|error| format!("Falha ao serializar metadata da sessao: {error}"))?;

    fs::write(&session.artifacts.metadata_path, contents).map_err(|error| {
        format!(
            "Falha ao salvar metadata da sessao em {}: {error}",
            session.artifacts.metadata_path.display()
        )
    })?;

    Ok(session)
}

pub fn session_dir(session_id: &str) -> Result<PathBuf, String> {
    Ok(data_dir()?.join("sessions").join(session_id))
}

pub fn data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .ok_or_else(|| String::from("Nao consegui descobrir a pasta de dados do usuario."))?;

    Ok(base.join("openvoice"))
}

pub fn write_track_wav(track: &CapturedTrack, path: &PathBuf) -> Result<(), String> {
    let spec = WavSpec {
        channels: track.audio.channels,
        sample_rate: track.audio.sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)
        .map_err(|error| format!("Falha ao criar WAV em {}: {error}", path.display()))?;

    for sample in &track.audio.samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let encoded = (clamped * i16::MAX as f32) as i16;
        writer
            .write_sample(encoded)
            .map_err(|error| format!("Falha ao escrever WAV em {}: {error}", path.display()))?;
    }

    writer
        .finalize()
        .map_err(|error| format!("Falha ao finalizar WAV em {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{generate_session_id, session_dir};

    #[test]
    fn generates_session_ids_with_prefix() {
        assert!(generate_session_id().starts_with("session-"));
    }

    #[test]
    fn builds_session_dir_path() {
        let path = session_dir("session-123").expect("path");

        assert!(path.ends_with("openvoice/sessions/session-123"));
    }
}
