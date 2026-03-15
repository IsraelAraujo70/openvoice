use crate::app::{Message, Overlay};
use iced::widget::{column, container, text};
use iced::{Background, Border, Color, Element, Length, Shadow};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    // During the 3-second fade-out (subtitle_closing), show only completed segments.
    // While actively transcribing, also show the in-flight partial.
    let completed = &state.live_completed_segments;
    let partial = if state.subtitle_closing {
        ""
    } else {
        state.live_partial_transcript.trim()
    };

    // Nothing to show — render a zero-size transparent container.
    if completed.is_empty() && partial.is_empty() {
        return container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    // Rolling window: last 2 completed segments plus the in-flight partial.
    let visible_lines: Vec<&str> = completed
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(|s| s.as_str())
        .collect();

    let mut col = column![].spacing(4);

    for line in &visible_lines {
        col = col.push(subtitle_line(line, false));
    }

    if !partial.is_empty() {
        let display = format!("{partial}▌");
        col = col.push(subtitle_line(&display, true));
    }

    container(
        container(col)
            .padding([10, 20])
            .style(|_| subtitle_pill_style()),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn subtitle_line(content: &str, is_partial: bool) -> iced::widget::Text<'static> {
    let alpha = if is_partial { 0.88 } else { 0.95 };
    let size = if is_partial { 17 } else { 16 };
    text(content.to_owned())
        .size(size)
        .color(Color::from_rgba(1.0, 1.0, 1.0, alpha))
}

fn subtitle_pill_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.72)))
        .border(Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 12.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        })
}
