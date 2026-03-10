use crate::app::Message;
use iced::widget::{button, text};
use iced::{Background, Border, Color, Element, Shadow};

pub fn view(
    label: &'static str,
    on_press: Option<Message>,
    emphasis: ButtonEmphasis,
) -> Element<'static, Message> {
    let (background, border, foreground) = match emphasis {
        ButtonEmphasis::Neutral => (
            Color::from_rgba8(255, 255, 255, 0.08),
            Color::from_rgba8(255, 255, 255, 0.12),
            Color::from_rgb8(244, 244, 232),
        ),
        ButtonEmphasis::Primary => (
            Color::from_rgb8(238, 236, 214),
            Color::from_rgb8(238, 236, 214),
            Color::from_rgb8(18, 18, 14),
        ),
        ButtonEmphasis::Accent => (
            Color::from_rgba8(94, 234, 212, 0.18),
            Color::from_rgba8(94, 234, 212, 0.26),
            Color::from_rgb8(222, 250, 244),
        ),
    };

    button(text(label).size(18))
        .padding(0)
        .width(42)
        .height(42)
        .on_press_maybe(on_press)
        .style(move |_, status| {
            let alpha = match status {
                button::Status::Active => 1.0,
                button::Status::Hovered => 1.08,
                button::Status::Pressed => 0.92,
                button::Status::Disabled => 0.45,
            };

            button::Style {
                background: Some(Background::Color(background.scale_alpha(alpha))),
                border: Border {
                    color: border.scale_alpha(alpha),
                    width: 1.0,
                    radius: 999.0.into(),
                },
                text_color: foreground,
                shadow: Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.18),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 18.0,
                },
                snap: false,
            }
        })
        .into()
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonEmphasis {
    Neutral,
    Primary,
    Accent,
}
