use iced::{Point, Size, keyboard, window};

use crate::modules::auth::domain::{OpenAiAuthSnapshot, PendingOpenAiOAuthFlow};
use crate::modules::dictation::domain::DictationOutput;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::settings::domain::AppSettings;

#[derive(Debug, Clone)]
pub enum Message {
    WindowOpened(window::Id),
    WindowCloseRequested(window::Id),
    MonitorSizeLoaded(Option<Size>),
    KeyEvent(keyboard::Event),
    StartDrag,
    WindowMoved(Point),
    OpenSettingsView,
    CloseSettingsView,
    SettingsApiKeyChanged(String),
    SettingsModelChanged(String),
    SettingsOpenAiRealtimeModelChanged(String),
    SaveSettings,
    SettingsSaved(Result<AppSettings, String>),
    StartOpenAiOAuthLogin,
    OpenAiOAuthStarted(Result<PendingOpenAiOAuthFlow, String>),
    OpenAiOAuthCallbackCaptured(Result<String, String>),
    OpenAiOAuthCallbackUrlChanged(String),
    SubmitOpenAiOAuthCallback,
    OpenAiOAuthFinished(Result<OpenAiAuthSnapshot, String>),
    LogoutOpenAi,
    OpenAiLogoutFinished(Result<(), String>),
    StartDictation,
    StopDictation,
    DictationFinished(Result<DictationOutput, String>),
    StartRealtimeTranscription,
    StopRealtimeTranscription,
    RealtimeEventReceived(Option<RuntimeEvent>),
    TogglePassthrough,
    Quit,
}
