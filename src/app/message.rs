use iced::{keyboard, window, Point, Size};

use crate::modules::dictation::domain::DictationOutput;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::settings::domain::SaveSettingsResult;

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
    SettingsOpenAiApiKeyChanged(String),
    SettingsOpenAiRealtimeModelChanged(String),
    SaveSettings,
    SettingsSaved(Result<SaveSettingsResult, String>),
    StartDictation,
    StopDictation,
    DictationFinished(Result<DictationOutput, String>),
    StartRealtimeTranscription,
    StopRealtimeTranscription,
    RealtimeEventReceived(Option<RuntimeEvent>),
    TogglePassthrough,
    Quit,
}
