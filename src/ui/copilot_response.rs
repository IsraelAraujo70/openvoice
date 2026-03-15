use crate::app::{Message, Overlay};
use crate::modules::copilot::domain::{CopilotChatMessage, CopilotRole};
use iced::widget::{button, column, container, markdown, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let latest_user = latest_message(state, CopilotRole::User);
    let latest_answer = latest_message(state, CopilotRole::Assistant);

    if latest_answer.is_none() && state.copilot_error.is_none() && !state.copilot_busy {
        return container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let has_answer = latest_answer.is_some();

    let content: Element<'_, Message> = if let Some(error) = &state.copilot_error {
        column![
            latest_user.map(question_hint).unwrap_or_else(empty_line),
            text(error.to_owned())
                .size(15)
                .color(Color::from_rgb8(255, 207, 164))
                .width(Length::Fill),
        ]
        .spacing(8)
        .into()
    } else if let Some(answer) = latest_answer {
        column![
            latest_user.map(question_hint).unwrap_or_else(empty_line),
            scrollable(container(markdown_message(answer)).width(Length::Fill))
                .height(Length::Fill),
        ]
        .height(Length::Fill)
        .spacing(8)
        .into()
    } else {
        text("Pensando...")
            .size(15)
            .color(Color::from_rgba8(226, 232, 240, 0.82))
            .into()
    };

    let footer = row![Space::new().width(Length::Fill), copy_btn(has_answer),]
        .align_y(Alignment::Center)
        .spacing(8);

    container(
        container(column![content, footer].spacing(8))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([12, 18])
            .style(|_| response_style()),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .padding([0, 28])
    .into()
}

fn latest_message<'a>(state: &'a Overlay, role: CopilotRole) -> Option<&'a CopilotChatMessage> {
    state
        .copilot_messages
        .iter()
        .rev()
        .find(|message| message.role == role)
}

fn markdown_message(message: &CopilotChatMessage) -> Element<'_, Message> {
    if message.content.trim().is_empty() && message.is_streaming {
        return text("Pensando...")
            .size(12)
            .color(Color::from_rgba8(148, 163, 184, 0.82))
            .into();
    }

    markdown::view(message.markdown_items.iter(), Theme::TokyoNightStorm)
        .map(Message::CopilotMarkdownLinkClicked)
}

fn question_hint(message: &CopilotChatMessage) -> Element<'_, Message> {
    text(format!("Voce: {}", truncate_inline(&message.content, 120)))
        .size(11)
        .color(Color::from_rgba8(226, 232, 240, 0.58))
        .into()
}

fn empty_line<'a>() -> Element<'a, Message> {
    text("").size(1).into()
}

fn copy_btn(has_answer: bool) -> Element<'static, Message> {
    let btn = button(text("Copiar").size(12).color(if has_answer {
        Color::WHITE
    } else {
        Color::from_rgba8(148, 163, 184, 0.5)
    }))
    .padding([6, 14])
    .on_press_maybe(has_answer.then_some(Message::CopyCopilotAnswer))
    .style(move |_, status| copy_btn_style(has_answer, status));

    btn.into()
}

fn copy_btn_style(enabled: bool, _status: button::Status) -> button::Style {
    if enabled {
        button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.10))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.14),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            text_color: Color::WHITE,
            snap: false,
        }
    } else {
        button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.07),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            text_color: Color::from_rgba8(148, 163, 184, 0.5),
            snap: false,
        }
    }
}

fn response_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.72)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.08),
            width: 1.0,
            radius: 16.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.32),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        })
}

fn truncate_inline(content: &str, max_chars: usize) -> String {
    let trimmed = content.trim().replace('\n', " ");

    if trimmed.chars().count() <= max_chars {
        return trimmed;
    }

    let mut shortened = trimmed
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    shortened.push('…');
    shortened
}
