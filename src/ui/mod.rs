pub mod components;
pub mod copilot;
pub mod home;
pub mod overlay;
pub mod sessions;
pub mod settings;
pub mod subtitle;
pub mod theme;

use crate::app::{MainView, Message, Overlay};
use iced::{Element, window};

pub fn view(state: &Overlay, window_id: window::Id) -> Element<'_, Message> {
    if state.subtitle_window_id == Some(window_id) {
        subtitle::view(state)
    } else if state.main_view == MainView::Copilot {
        copilot::view(state)
    } else if state.main_view == MainView::Home {
        home::view(state)
    } else {
        overlay::view(state)
    }
}
