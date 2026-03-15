use iced::widget::markdown;
use iced::widget::text_editor;
use iced::{keyboard, window, Point, Size};

use crate::modules::auth::domain::{OpenAiAuthSnapshot, PendingOpenAiOAuthFlow};
use crate::modules::copilot::application::{
    ActiveCopilotStream, LoadedCopilotThread, RuntimeEvent as CopilotRuntimeEvent,
};
use crate::modules::copilot::domain::{CopilotMode, CopilotThreadSummary, ScreenshotAttachment};
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
    // Copilot window
    CopilotWindowOpened(window::Id),
    CopilotResponseWindowOpened(window::Id),
    // Live transcription persistence
    LiveSessionCreated(Result<i64, String>),
    LiveSessionSegmentsPersisted(Result<usize, String>),
    LiveSessionFinalized(Result<(), String>),
    LiveSessionTitleGenerated(Result<(i64, String), String>),
    // Sessions data (loaded inside Home tab)
    SessionsLoaded(Result<Vec<SessionSummary>, String>),
    SessionsSearchChanged(String),
    SessionSelected(i64),
    OpenSessionDetail(i64),
    SessionDetailLoaded(Result<Vec<String>, String>),
    CopySessionTranscript,
    DeleteSession(i64),
    SessionDeleted(Result<i64, String>),
    // Copilot
    CopilotInputEdited(text_editor::Action),
    CopilotModeChanged(CopilotMode),
    StartCopilotListen,
    StopCopilotListen,
    CopilotListenTranscribed(Result<String, String>),
    CaptureCopilotScreenshot,
    CopilotScreenshotCaptured(Result<ScreenshotAttachment, String>),
    ClearCopilotScreenshot,
    SubmitCopilotRequest,
    CopilotStreamStarted(Result<ActiveCopilotStream, String>),
    CopilotStreamEventReceived(Option<CopilotRuntimeEvent>),
    CopilotThreadsLoaded(Result<Vec<CopilotThreadSummary>, String>),
    CopilotThreadSelected(i64),
    CopilotThreadLoaded(Result<LoadedCopilotThread, String>),
    OpenCopilotThreadInOverlay(i64),
    NewCopilotThread,
    CopilotMarkdownLinkClicked(markdown::Uri),
    CopyCopilotAnswer,
    DeleteCopilotThread(i64),
    CopilotThreadDeleted(Result<i64, String>),
    // Window behavior
    TogglePassthrough,
    Quit,
}
