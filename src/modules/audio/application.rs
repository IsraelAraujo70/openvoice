use crate::modules::audio::domain::CaptureSession;
use crate::modules::audio::infrastructure::{microphone, storage, system};

pub struct ActiveCaptureSession {
    pub session_id: String,
    pub started_at_unix_ms: u128,
    microphone: microphone::Recorder,
    system: system::Recorder,
}

impl ActiveCaptureSession {
    pub fn session_label(&self) -> &str {
        &self.session_id
    }
}

pub fn start_capture_session() -> Result<ActiveCaptureSession, String> {
    let session_id = storage::generate_session_id();
    let started_at_unix_ms = storage::unix_timestamp_ms();
    let microphone = microphone::start_default_recording()?;

    let system = match system::start_default_recording() {
        Ok(system) => system,
        Err(error) => {
            drop(microphone);
            return Err(error);
        }
    };

    Ok(ActiveCaptureSession {
        session_id,
        started_at_unix_ms,
        microphone,
        system,
    })
}

pub fn finish_capture_session(session: ActiveCaptureSession) -> Result<CaptureSession, String> {
    let ActiveCaptureSession {
        session_id,
        started_at_unix_ms,
        microphone,
        system,
    } = session;
    let microphone_track = microphone.finish()?;
    let system_track = system.finish()?;
    let finished_at_unix_ms = storage::unix_timestamp_ms();

    storage::persist_session(
        session_id,
        started_at_unix_ms,
        finished_at_unix_ms,
        microphone_track,
        system_track,
    )
}
