use iced::{keyboard, window, Point, Size};

use crate::modules::dictation::domain::DualTranscriptOutput;
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
    SaveSettings,
    SettingsSaved(Result<AppSettings, String>),
    StartDictation,
    StopDictation,
    DictationFinished(Result<DualTranscriptOutput, String>),
    TogglePassthrough,
    Quit,
}
