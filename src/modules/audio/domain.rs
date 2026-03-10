use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSourceKind {
    Microphone,
    SystemMonitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone)]
pub struct CapturedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl CapturedAudio {
    pub fn duration_seconds(&self) -> f32 {
        let frames = self.samples.len() as f32 / self.channels.max(1) as f32;
        frames / self.sample_rate.max(1) as f32
    }

    pub fn format(&self) -> CaptureFormat {
        CaptureFormat {
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapturedTrack {
    pub source: AudioSourceKind,
    pub device_name: String,
    pub audio: CapturedAudio,
}

impl CapturedTrack {
    pub fn duration_seconds(&self) -> f32 {
        self.audio.duration_seconds()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackArtifact {
    pub source: AudioSourceKind,
    pub device_name: String,
    pub wav_path: PathBuf,
    pub format: CaptureFormat,
    pub frame_count: usize,
    pub status: String,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifacts {
    pub session_dir: PathBuf,
    pub metadata_path: PathBuf,
    pub microphone_wav: PathBuf,
    pub system_wav: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CaptureSession {
    pub session_id: String,
    pub started_at_unix_ms: u128,
    pub finished_at_unix_ms: u128,
    pub microphone: CapturedTrack,
    pub system: CapturedTrack,
    pub microphone_artifact: TrackArtifact,
    pub system_artifact: TrackArtifact,
    pub artifacts: SessionArtifacts,
}

impl CaptureSession {
    pub fn duration_seconds(&self) -> f32 {
        self.microphone
            .duration_seconds()
            .max(self.system.duration_seconds())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub source: AudioSourceKind,
    pub device_name: String,
    pub wav_path: PathBuf,
    pub format: CaptureFormat,
    pub frame_count: usize,
    pub status: String,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub started_at_unix_ms: u128,
    pub finished_at_unix_ms: u128,
    pub tracks: Vec<TrackMetadata>,
}

impl SessionMetadata {
    pub fn from_session(session: &CaptureSession) -> Self {
        Self {
            session_id: session.session_id.clone(),
            started_at_unix_ms: session.started_at_unix_ms,
            finished_at_unix_ms: session.finished_at_unix_ms,
            tracks: vec![
                TrackMetadata {
                    source: session.microphone_artifact.source,
                    device_name: session.microphone_artifact.device_name.clone(),
                    wav_path: session.microphone_artifact.wav_path.clone(),
                    format: session.microphone_artifact.format,
                    frame_count: session.microphone_artifact.frame_count,
                    status: session.microphone_artifact.status.clone(),
                    duration_seconds: session.microphone_artifact.duration_seconds,
                },
                TrackMetadata {
                    source: session.system_artifact.source,
                    device_name: session.system_artifact.device_name.clone(),
                    wav_path: session.system_artifact.wav_path.clone(),
                    format: session.system_artifact.format,
                    frame_count: session.system_artifact.frame_count,
                    status: session.system_artifact.status.clone(),
                    duration_seconds: session.system_artifact.duration_seconds,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioSourceKind, CaptureFormat, CapturedAudio, TrackArtifact};
    use std::path::PathBuf;

    #[test]
    fn computes_audio_duration() {
        let capture = CapturedAudio {
            samples: vec![0.0; 48_000],
            sample_rate: 48_000,
            channels: 1,
        };

        assert_eq!(capture.duration_seconds(), 1.0);
    }

    #[test]
    fn preserves_track_artifact_frame_count() {
        let artifact = TrackArtifact {
            source: AudioSourceKind::Microphone,
            device_name: String::from("default"),
            wav_path: PathBuf::from("/tmp/mic.wav"),
            format: CaptureFormat {
                sample_rate: 48_000,
                channels: 2,
            },
            frame_count: 1_024,
            status: String::from("captured"),
            duration_seconds: 2.0,
        };

        assert_eq!(artifact.frame_count, 1_024);
        assert_eq!(artifact.status, "captured");
    }
}
