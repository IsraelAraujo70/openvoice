use iced::{Point, Size, keyboard, window};

use crate::modules::auth::domain::{OpenAiAuthSnapshot, PendingOpenAiOAuthFlow};
use crate::modules::dictation::domain::DictationOutput;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::live_transcription::infrastructure::db::SessionSummary;
use crate::modules::settings::domain::AppSettings;

#[derive(Debug, Clone)]
pub enum Message {
    // Window lifecycle
    WindowOpened(window::Id),
    WindowCloseRequested(window::Id),
    MonitorSizeLoaded(Option<Size>),
    // Input events
    KeyEvent(keyboard::Event),
    StartDrag,
    WindowMoved(Point),
    // Navigation
    OpenSettingsView,
    CloseSettingsView,
    // Settings form
    SettingsApiKeyChanged(String),
    SettingsOpenAiRealtimeApiKeyChanged(String),
    SettingsModelChanged(String),
    SettingsOpenAiRealtimeModelChanged(String),
    SettingsOpenAiRealtimeLanguageChanged(String),
    SaveSettings,
    SettingsSaved(Result<AppSettings, String>),
    // OpenAI OAuth
    StartOpenAiOAuthLogin,
    OpenAiOAuthStarted(Result<PendingOpenAiOAuthFlow, String>),
    OpenAiOAuthCallbackCaptured(Result<String, String>),
    OpenAiOAuthCallbackUrlChanged(String),
    SubmitOpenAiOAuthCallback,
    OpenAiOAuthFinished(Result<OpenAiAuthSnapshot, String>),
    LogoutOpenAi,
    OpenAiLogoutFinished(Result<(), String>),
    // Dictation (mic → OpenRouter)
    StartDictation,
    StopDictation,
    DictationFinished(Result<DictationOutput, String>),
    // Realtime transcription (system audio → OpenAI Realtime API)
    StartRealtimeTranscription,
    StopRealtimeTranscription,
    RealtimeEventReceived(Option<RuntimeEvent>),
    // Subtitle window
    SubtitleWindowOpened(window::Id),
    CloseSubtitleWindow,
    // Live transcription persistence
    LiveTranscriptionSaved(Result<i64, String>),
    // Sessions window
    OpenSessionsView,
    SessionsWindowOpened(window::Id),
    CloseSessionsView,
    SessionsLoaded(Result<Vec<SessionSummary>, String>),
    SessionSelected(i64),
    SessionDetailLoaded(Result<Vec<String>, String>),
    CopySessionTranscript,
    // Window behavior
    TogglePassthrough,
    Quit,
}
