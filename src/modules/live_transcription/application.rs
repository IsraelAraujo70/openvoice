#![allow(dead_code)]

use crate::modules::live_transcription::domain::{
    LiveTranscriptionConfig, RuntimeEvent, TurnDetectionMode,
};
use crate::modules::live_transcription::infrastructure::openai_realtime::{
    self, SessionHandle, SharedReceiver,
};
use crate::modules::settings::domain::AppSettings;

const CAPTION_VAD_THRESHOLD: f32 = 0.47;
const CAPTION_PREFIX_PADDING_MS: u32 = 320;
const CAPTION_SILENCE_DURATION_MS: u32 = 420;
const BALANCED_VAD_THRESHOLD: f32 = 0.45;
const BALANCED_PREFIX_PADDING_MS: u32 = 380;
const BALANCED_SILENCE_DURATION_MS: u32 = 540;
const ACCURACY_VAD_THRESHOLD: f32 = 0.42;
const ACCURACY_PREFIX_PADDING_MS: u32 = 480;
const ACCURACY_SILENCE_DURATION_MS: u32 = 700;

#[derive(Debug, Clone, Copy)]
enum RealtimeProfile {
    Caption,
    Balanced,
    Accuracy,
}

pub struct ActiveLiveTranscription {
    session: SessionHandle,
}

impl ActiveLiveTranscription {
    pub fn receiver(&self) -> SharedReceiver {
        self.session.receiver()
    }

    pub fn stop(self) {
        self.session.stop();
    }
}

pub fn start_live_transcription(settings: &AppSettings) -> Result<ActiveLiveTranscription, String> {
    let bearer_token = settings.openai_realtime_api_key.trim();
    if bearer_token.is_empty() {
        return Err(String::from(
            "Cadastre e salve uma OpenAI API key antes de iniciar a transcription realtime.",
        ));
    }

    let profile = realtime_profile_from_settings(settings);
    let language = normalize_language_hint(&settings.openai_realtime_language);
    let (threshold, prefix_padding_ms, silence_duration_ms) = profile_vad(profile);

    let config = LiveTranscriptionConfig {
        bearer_token: bearer_token.to_owned(),
        model: settings.openai_realtime_model.clone(),
        prompt: build_realtime_prompt(profile, language.as_deref()),
        language,
        noise_reduction: None,
        turn_detection: TurnDetectionMode::ServerVad {
            threshold,
            prefix_padding_ms,
            silence_duration_ms,
        },
    };

    let session = openai_realtime::start_session(config)?;
    Ok(ActiveLiveTranscription { session })
}

pub fn poll_next_event(receiver: SharedReceiver) -> Option<RuntimeEvent> {
    receiver.lock().ok()?.recv().ok()
}

fn realtime_profile_from_settings(settings: &AppSettings) -> RealtimeProfile {
    match settings.openai_realtime_profile.trim() {
        "caption" => RealtimeProfile::Caption,
        "accuracy" => RealtimeProfile::Accuracy,
        "balanced" => RealtimeProfile::Balanced,
        _ => realtime_profile_from_env(),
    }
}

fn realtime_profile_from_env() -> RealtimeProfile {
    match std::env::var("OPENVOICE_REALTIME_PROFILE")
        .ok()
        .as_deref()
        .map(str::trim)
    {
        Some("caption") => RealtimeProfile::Caption,
        Some("accuracy") => RealtimeProfile::Accuracy,
        _ => RealtimeProfile::Balanced,
    }
}

fn normalize_language_hint(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn profile_vad(profile: RealtimeProfile) -> (f32, u32, u32) {
    match profile {
        RealtimeProfile::Caption => (
            CAPTION_VAD_THRESHOLD,
            CAPTION_PREFIX_PADDING_MS,
            CAPTION_SILENCE_DURATION_MS,
        ),
        RealtimeProfile::Balanced => (
            BALANCED_VAD_THRESHOLD,
            BALANCED_PREFIX_PADDING_MS,
            BALANCED_SILENCE_DURATION_MS,
        ),
        RealtimeProfile::Accuracy => (
            ACCURACY_VAD_THRESHOLD,
            ACCURACY_PREFIX_PADDING_MS,
            ACCURACY_SILENCE_DURATION_MS,
        ),
    }
}

fn build_realtime_prompt(profile: RealtimeProfile, language: Option<&str>) -> Option<String> {
    let style = match profile {
        RealtimeProfile::Caption => "Return fast live captions with short readable phrases.",
        RealtimeProfile::Balanced => "Return fluent live captions with readable phrasing.",
        RealtimeProfile::Accuracy => "Prefer accurate full phrases over premature segmentation.",
    };

    let language_hint = match language {
        Some("pt") => "Transcribe only Portuguese speech. Preserve names and technical terms.",
        Some("en") => "Transcribe only English speech. Preserve names and technical terms.",
        Some(code) => {
            return Some(format!(
                "{style} Transcribe only spoken language '{code}'. Preserve names and technical terms. Do not invent missing words."
            ));
        }
        None => "Transcribe the spoken language you hear. Preserve names and technical terms.",
    };

    Some(format!(
        "{style} {language_hint} Do not invent missing words. Prefer natural punctuation."
    ))
}

#[cfg(test)]
mod tests {
    use super::{RealtimeProfile, build_realtime_prompt, normalize_language_hint, profile_vad};

    #[test]
    fn builds_language_specific_prompt() {
        let prompt = build_realtime_prompt(RealtimeProfile::Balanced, Some("en")).expect("prompt");

        assert!(prompt.contains("English"));
        assert!(prompt.contains("Preserve names and technical terms"));
    }

    #[test]
    fn normalizes_empty_language_to_none() {
        assert_eq!(normalize_language_hint("   "), None);
        assert_eq!(normalize_language_hint("pt").as_deref(), Some("pt"));
    }

    #[test]
    fn accuracy_profile_waits_longer_than_caption_profile() {
        let (_, caption_prefix, caption_silence) = profile_vad(RealtimeProfile::Caption);
        let (_, accuracy_prefix, accuracy_silence) = profile_vad(RealtimeProfile::Accuracy);

        assert!(accuracy_prefix > caption_prefix);
        assert!(accuracy_silence > caption_silence);
    }
}
