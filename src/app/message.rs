use iced::widget::text_editor;
use iced::{Point, Size, keyboard, window};
use iced::widget::markdown;

use crate::modules::auth::domain::{OpenAiAuthSnapshot, PendingOpenAiOAuthFlow};
use crate::modules::copilot::domain::{CopilotAnswer, CopilotMode, ScreenshotAttachment};
use crate::modules::dictation::domain::DictationOutput;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::live_transcription::infrastructure::db::SessionSummary;
use crate::modules::settings::domain::AppSettings;

use crate::app::state::HomeTab;

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
    OpenHomeView,
    CloseHomeView,
    OpenCopilotView,
    CloseCopilotView,
    SwitchHomeTab(HomeTab),
    // Settings form
    SettingsApiKeyChanged(String),
    SettingsOpenAiRealtimeApiKeyChanged(String),
    SettingsModelChanged(String),
    SettingsOpenAiRealtimeModelChanged(String),
    SettingsOpenAiRealtimeLanguageChanged(String),
    SettingsOpenAiRealtimeProfileChanged(String),
    SettingsCopilotModelChanged(String),
    SettingsCopilotDefaultModeChanged(String),
    SettingsCopilotAutoIncludeTranscriptChanged(bool),
    SettingsCopilotSaveHistoryChanged(bool),
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
    LiveSessionCreated(Result<i64, String>),
    LiveSessionSegmentsPersisted(Result<usize, String>),
    LiveSessionFinalized(Result<(), String>),
    LiveSessionTitleGenerated(Result<(i64, String), String>),
    // Sessions data (loaded inside Home tab)
    SessionsLoaded(Result<Vec<SessionSummary>, String>),
    SessionsSearchChanged(String),
    SessionSelected(i64),
    SessionDetailLoaded(Result<Vec<String>, String>),
    CopySessionTranscript,
    // Copilot
    CopilotInputEdited(text_editor::Action),
    CopilotModeChanged(CopilotMode),
    CopilotIncludeTranscriptChanged(bool),
    CaptureCopilotScreenshot,
    CopilotScreenshotCaptured(Result<ScreenshotAttachment, String>),
    ClearCopilotScreenshot,
    SubmitCopilotRequest,
    CopilotAnswerReceived(Result<CopilotAnswer, String>),
    CopilotMarkdownLinkClicked(markdown::Uri),
    CopyCopilotAnswer,
    // Window behavior
    TogglePassthrough,
    Quit,
}
