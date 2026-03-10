use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element};

pub fn view<'a>(
    state_label: &'a str,
    hint: &'a str,
    accent: Color,
) -> Element<'a, crate::app::Message> {
    let bars = [20.0, 38.0, 62.0, 84.0, 62.0, 38.0, 20.0]
        .into_iter()
        .map(|height| {
            container("")
                .width(8)
                .height(height)
                .style(move |_| {
                    container::Style::default()
                        .background(Background::Color(accent))
                        .border(Border {
                            color: accent,
                            width: 0.0,
                            radius: 999.0.into(),
                        })
                })
                .into()
        })
        .collect::<Vec<Element<'a, crate::app::Message>>>();

    column![
        row(bars).spacing(8).align_y(Alignment::Center),
        text(state_label).size(18),
        text(hint)
            .size(12)
            .color(Color::from_rgba8(226, 232, 240, 0.72)),
    ]
    .spacing(12)
    .align_x(Alignment::Center)
    .into()
}
