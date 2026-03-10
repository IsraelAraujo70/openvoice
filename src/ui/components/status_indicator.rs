use crate::app::Message;
use iced::widget::{container, row, text};
use iced::{Alignment, Background, Border, Color, Element};

pub fn view<'a>(label: &'a str, accent: Color) -> Element<'a, Message> {
    let dot = container("").width(8).height(8).style(move |_| {
        container::Style::default()
            .background(Background::Color(accent))
            .border(Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 999.0.into(),
            })
    });

    row![
        dot,
        text(label)
            .size(11)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.55)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
