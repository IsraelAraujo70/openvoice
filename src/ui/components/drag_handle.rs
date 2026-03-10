use crate::app::Message;
use iced::widget::{container, mouse_area, text};
use iced::{mouse, Color, Element, Length};

/// A subtle drag handle that initiates window drag on press.
/// Uses vertical ellipsis "⋮" as a grip icon at low opacity.
pub fn view<'a>() -> Element<'a, Message> {
    let grip = container(
        text("⋮")
            .size(16)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.22)),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(18)
    .height(Length::Fill);

    mouse_area(grip)
        .on_press(Message::StartDrag)
        .interaction(mouse::Interaction::Grab)
        .into()
}
