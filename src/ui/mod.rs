pub mod components;
pub mod overlay;
pub mod sessions;
pub mod settings;
pub mod subtitle;
pub mod theme;

use crate::app::{Message, Overlay};
use iced::{window, Element};

pub fn view(state: &Overlay, window_id: window::Id) -> Element<'_, Message> {
    if state.subtitle_window_id == Some(window_id) {
        subtitle::view(state)
    } else if state.sessions_window_id == Some(window_id) {
        sessions::view(state)
    } else if state.settings_open {
        settings::view(state)
    } else {
        overlay::view(state)
    }
}
