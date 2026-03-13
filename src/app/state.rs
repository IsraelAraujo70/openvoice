use crate::app::message::Message;
use crate::modules::audio::infrastructure::microphone::Recorder;
use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::PendingOpenAiOAuthFlow;
use crate::modules::live_transcription::application::ActiveLiveTranscription;
use crate::modules::live_transcription::infrastructure::db::SessionSummary;
use crate::modules::settings::application as settings_application;
use crate::modules::settings::domain::{AppSettings, SettingsForm};
use crate::platform::window as platform_window;
use crate::platform::window::MonitorGeometry;
use iced::{window, Point, Task};
use std::collections::HashSet;

pub struct Overlay {
    // Window IDs
    pub main_window_id: Option<window::Id>,
    pub subtitle_window_id: Option<window::Id>,

    // HUD state
    pub passthrough_enabled: bool,
    pub main_view: MainView,
    pub home_tab: HomeTab,
    pub primary_monitor: Option<MonitorGeometry>,
    pub hud_position: Option<Point>,
    pub phase: OverlayPhase,
    pub hint: String,
    pub error: Option<String>,
    pub preview: Option<String>,

    // Settings
    pub settings: AppSettings,
    pub settings_form: SettingsForm,
    pub is_saving_settings: bool,
    pub settings_note: Option<String>,

    // Auth (OpenAI OAuth)
    pub is_openai_authenticating: bool,
    pub pending_openai_oauth: Option<PendingOpenAiOAuthFlow>,
    pub openai_callback_url_input: String,
    pub has_openai_credentials: bool,
    pub openai_account_label: Option<String>,

    // Dictation (mic recording)
    pub recorder: Option<Recorder>,

    // Live transcription (system audio streaming)
    pub live_transcription: Option<ActiveLiveTranscription>,
    pub live_session_started_at: Option<String>,
    pub live_session_db_id: Option<i64>,
    pub live_session_creating: bool,
    pub live_session_finalizing: bool,
    pub live_session_stopped_at: Option<String>,
    pub live_segments_persisting: bool,
    pub live_persisted_segment_count: usize,
    pub live_partial_item_id: Option<String>,
    pub live_partial_transcript: String,
    pub live_completed_segments: Vec<String>,
    pub subtitle_closing: bool,

    // Sessions view
    pub sessions_list: Vec<SessionSummary>,
    pub sessions_loading: bool,
    pub sessions_error: Option<String>,
    pub sessions_search_query: String,
    pub selected_session_id: Option<i64>,
    pub selected_session_segments: Vec<String>,
    pub selected_session_loading: bool,

    // Title generation circuit breaker: session IDs where generation already failed
    pub title_gen_failed_ids: HashSet<i64>,
}

impl Overlay {
    pub fn title(&self, _window: window::Id) -> String {
        String::from("OpenVoice")
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.phase, OverlayPhase::Recording)
    }

    pub fn is_dictation_recording(&self) -> bool {
        self.recorder.is_some()
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.phase, OverlayPhase::Processing)
    }

    pub fn is_live_transcribing(&self) -> bool {
        self.live_transcription.is_some()
    }

    pub fn can_start_dictation(&self) -> bool {
        self.settings.has_api_key()
            && !self.is_processing()
            && !self.is_saving_settings
            && !self.is_live_transcribing()
    }

    pub fn can_start_realtime_transcription(&self) -> bool {
        !self.is_recording()
            && !self.is_processing()
            && !self.is_saving_settings
            && !self.is_openai_authenticating
            && !self.is_live_transcribing()
            && self.settings.has_openai_realtime_api_key()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPhase {
    Idle,
    Recording,
    Processing,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainView {
    Hud,
    Home,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeTab {
    Home,
    Sessions,
    Settings,
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayConfig {
    pub start_with_passthrough: bool,
}

impl OverlayConfig {
    pub fn from_env() -> Self {
        let start_with_passthrough = std::env::var("OPENVOICE_MOUSE_PASSTHROUGH")
            .ok()
            .as_deref()
            .map(|value| matches!(value, "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false);

        Self {
            start_with_passthrough,
        }
    }
}

pub fn boot() -> (Overlay, Task<Message>) {
    let config = OverlayConfig::from_env();
    let primary_monitor = platform_window::detect_primary_monitor_geometry();
    let (settings, settings_error) = match settings_application::load_settings() {
        Ok(settings) => (settings, None),
        Err(error) => (AppSettings::default(), Some(error)),
    };
    let auth_snapshot = auth_application::load_auth_snapshot()
        .unwrap_or_else(|_| crate::modules::auth::domain::OpenAiAuthSnapshot::signed_out());
    let settings_form = SettingsForm::from(&settings);
    let missing_api_key = (!settings.has_api_key())
        .then(|| String::from("Cadastre sua OpenRouter API key no painel de settings abaixo."));

    let state = Overlay {
        main_window_id: None,
        subtitle_window_id: None,
        passthrough_enabled: config.start_with_passthrough,
        main_view: MainView::Hud,
        home_tab: HomeTab::Home,
        primary_monitor,
        hud_position: None,
        phase: OverlayPhase::Idle,
        hint: if config.start_with_passthrough {
            String::from("Passthrough ativo. Pressione P para interagir.")
        } else {
            String::new()
        },
        error: settings_error.or(missing_api_key),
        preview: None,
        settings,
        settings_form,
        is_saving_settings: false,
        is_openai_authenticating: false,
        pending_openai_oauth: None,
        openai_callback_url_input: String::new(),
        has_openai_credentials: auth_snapshot.is_authenticated,
        openai_account_label: auth_snapshot.account_label,
        settings_note: None,
        recorder: None,
        live_transcription: None,
        live_session_started_at: None,
        live_session_db_id: None,
        live_session_creating: false,
        live_session_finalizing: false,
        live_session_stopped_at: None,
        live_segments_persisting: false,
        live_persisted_segment_count: 0,
        live_partial_item_id: None,
        live_partial_transcript: String::new(),
        live_completed_segments: Vec::new(),
        subtitle_closing: false,
        sessions_list: Vec::new(),
        sessions_loading: false,
        sessions_error: None,
        sessions_search_query: String::new(),
        selected_session_id: None,
        selected_session_segments: Vec::new(),
        selected_session_loading: false,
        title_gen_failed_ids: HashSet::new(),
    };

    // With iced::daemon, we must open the initial window manually.
    let (_, open_hud) = window::open(platform_window::hud_settings());

    (state, open_hud.map(Message::WindowOpened))
}
