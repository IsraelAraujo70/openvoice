pub mod components;
pub mod overlay;
pub mod settings;
pub mod theme;

use crate::app::{Message, Overlay, Scene};
use iced::Element;

pub fn view(state: &Overlay) -> Element<'_, Message> {
    if matches!(state.scene, Scene::Settings) {
        settings::view(state)
    } else {
        overlay::view(state)
    }
}
