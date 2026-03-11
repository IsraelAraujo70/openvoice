use crate::app::Message;
use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Element, Length, Shadow};

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind {
    Ghost,
    Caption(Color),
    Mic(Color),
}

pub fn view(
    label: &'static str,
    on_press: Option<Message>,
    kind: ButtonKind,
) -> Element<'static, Message> {
    match kind {
        ButtonKind::Ghost => ghost_button(label, on_press),
        ButtonKind::Caption(accent) => caption_button(label, on_press, accent),
        ButtonKind::Mic(accent) => mic_button(on_press, accent),
    }
}

fn ghost_button(label: &'static str, on_press: Option<Message>) -> Element<'static, Message> {
    button(
        container(text(label).size(14))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(0)
    .width(26)
    .height(26)
    .on_press_maybe(on_press)
    .style(|_, status| {
        let (bg_alpha, text_alpha) = match status {
            button::Status::Active => (0.0, 0.4),
            button::Status::Hovered => (0.06, 0.82),
            button::Status::Pressed => (0.04, 0.6),
            button::Status::Disabled => (0.0, 0.12),
        };

        button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, bg_alpha))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 999.0.into(),
            },
            text_color: Color::from_rgba(1.0, 1.0, 1.0, text_alpha),
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            snap: false,
        }
    })
    .into()
}

fn caption_button(
    label: &'static str,
    on_press: Option<Message>,
    accent: Color,
) -> Element<'static, Message> {
    button(
        container(text(label).size(11))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(0)
    .width(32)
    .height(22)
    .on_press_maybe(on_press)
    .style(move |_, status| {
        let (bg_alpha, border_alpha, text_alpha) = match status {
            button::Status::Active => (0.08, 0.28, 0.88),
            button::Status::Hovered => (0.14, 0.5, 1.0),
            button::Status::Pressed => (0.1, 0.34, 0.94),
            button::Status::Disabled => (0.02, 0.1, 0.2),
        };

        button::Style {
            background: Some(Background::Color(accent.scale_alpha(bg_alpha))),
            border: Border {
                color: accent.scale_alpha(border_alpha),
                width: 1.0,
                radius: 7.0.into(),
            },
            text_color: Color::from_rgba(1.0, 1.0, 1.0, text_alpha),
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            snap: false,
        }
    })
    .into()
}

fn mic_button(on_press: Option<Message>, accent: Color) -> Element<'static, Message> {
    let inner_dot = container("").width(10).height(10).style(move |_| {
        container::Style::default()
            .background(Background::Color(accent.scale_alpha(0.82)))
            .border(Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 999.0.into(),
            })
    });

    button(
        container(inner_dot)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(0)
    .width(34)
    .height(34)
    .on_press_maybe(on_press)
    .style(move |_, status| {
        let scale = match status {
            button::Status::Active => 1.0,
            button::Status::Hovered => 1.3,
            button::Status::Pressed => 0.8,
            button::Status::Disabled => 0.3,
        };

        button::Style {
            background: Some(Background::Color(accent.scale_alpha(0.08 * scale))),
            border: Border {
                color: accent.scale_alpha(0.3 * scale),
                width: 1.5,
                radius: 999.0.into(),
            },
            text_color: Color::TRANSPARENT,
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            snap: false,
        }
    })
    .into()
}
