use iced::{Size, keyboard, window};

#[derive(Debug, Clone)]
pub enum Message {
    WindowOpened(window::Id),
    MonitorSizeLoaded(Option<Size>),
    KeyEvent(keyboard::Event),
    TogglePassthrough,
    Quit,
}
