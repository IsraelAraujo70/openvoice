use crate::app::{Message, Overlay};
use crate::modules::copilot::application as copilot_application;
use crate::modules::copilot::domain::{CopilotChatMessage, CopilotMode, CopilotRole};
use iced::widget::{
    Space, button, checkbox, column, container, markdown, radio, row, scrollable, text,
    text_editor,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    match state.copilot_mode {
        CopilotMode::Meeting => meeting_overlay(state),
        CopilotMode::Interview | CopilotMode::General => chat_overlay(state),
    }
}

fn chat_overlay(state: &Overlay) -> Element<'_, Message> {
    let header = row![
        column![
            text("Copilot").size(24).color(Color::WHITE),
            text(context_label(state))
                .size(12)
                .color(Color::from_rgba8(226, 232, 240, 0.62)),
        ]
        .spacing(2),
        Space::new().width(Length::Fill),
        subtle_action("Fechar", Some(Message::CloseCopilotView)),
    ]
    .align_y(Alignment::Center);

    let shell = container(
        column![
            header,
            mode_bar(state),
            message_feed(state, true),
            composer(state, false),
        ]
        .spacing(16),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([22, 24])
    .style(|_| shell_style());

    container(shell)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .into()
}

fn meeting_overlay(state: &Overlay) -> Element<'_, Message> {
    let latest_user = state
        .copilot_messages
        .iter()
        .rev()
        .find(|message| message.role == CopilotRole::User);
    let latest_answer = state
        .copilot_messages
        .iter()
        .rev()
        .find(|message| message.role == CopilotRole::Assistant);

    let mut body = column![
        row![
            text("Meeting Copilot")
                .size(12)
                .color(Color::from_rgba8(148, 163, 184, 0.82)),
            Space::new().width(Length::Fill),
            subtle_action("Chat", Some(Message::CopilotModeChanged(CopilotMode::General))),
            subtle_action("Fechar", Some(Message::CloseCopilotView)),
        ]
        .align_y(Alignment::Center)
    ]
    .spacing(10);

    if let Some(question) = latest_user {
        body = body.push(
            text(format!("Pergunta: {}", question.content))
                .size(11)
                .color(Color::from_rgba8(226, 232, 240, 0.56)),
        );
    }

    if let Some(error) = &state.copilot_error {
        body = body.push(error_card(error));
    } else if let Some(answer) = latest_answer {
        body = body.push(
            container(markdown_message(answer))
                .padding([12, 14])
                .width(Length::Fill)
                .style(|_| meeting_answer_style()),
        );
    } else {
        body = body.push(
            text("Respostas curtas e discretas durante a reuniao.")
                .size(12)
                .color(Color::from_rgba8(226, 232, 240, 0.66)),
        );
    }

    body = body.push(composer(state, true));

    container(
        container(body)
            .padding([12, 14])
            .style(|_| meeting_shell_style()),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn mode_bar(state: &Overlay) -> Element<'_, Message> {
    let left = row![
        mode_radio("General", CopilotMode::General, state.copilot_mode),
        mode_radio("Interview", CopilotMode::Interview, state.copilot_mode),
        mode_radio("Meeting", CopilotMode::Meeting, state.copilot_mode),
    ]
    .spacing(14)
    .wrap();

    let right = checkbox(state.copilot_include_transcript)
        .label("Transcript automatico")
        .on_toggle(Message::CopilotIncludeTranscriptChanged)
        .text_size(12);

    row![left, Space::new().width(Length::Fill), right]
        .align_y(Alignment::Center)
        .into()
}

fn message_feed(state: &Overlay, show_history: bool) -> Element<'_, Message> {
    let mut col = column![].spacing(14);

    if state.copilot_messages.is_empty() && state.copilot_error.is_none() {
        col = col.push(
            container(
                text("Use o transcript atual, sessao salva e screenshot opcional para conversar com o copiloto.")
                    .size(12)
                    .color(Color::from_rgba8(148, 163, 184, 0.72)),
            )
            .padding([4, 2]),
        );
    }

    let iter: Box<dyn Iterator<Item = _>> = if show_history {
        Box::new(state.copilot_messages.iter())
    } else {
        Box::new(state.copilot_messages.iter().rev().take(2).collect::<Vec<_>>().into_iter().rev())
    };

    for message in iter {
        col = col.push(match message.role {
            CopilotRole::User => user_message(&message.content),
            CopilotRole::Assistant => assistant_message(message),
        });
    }

    if let Some(error) = &state.copilot_error {
        col = col.push(error_card(error));
    }

    if state.copilot_busy {
        col = col.push(typing_indicator());
    }

    scrollable(col).height(Length::Fill).into()
}

fn composer(state: &Overlay, compact: bool) -> Element<'_, Message> {
    let screenshot_summary = state
        .copilot_screenshot
        .as_ref()
        .map(copilot_application::screenshot_summary)
        .unwrap_or_else(|| String::from("Sem screenshot"));

    let editor_height = if compact { 72.0 } else { 120.0 };

    let editor = container(
        text_editor(&state.copilot_input)
            .placeholder("Pergunte algo...")
            .on_action(Message::CopilotInputEdited)
            .height(Length::Fixed(editor_height))
            .padding(12),
    )
    .width(Length::Fill)
    .style(|_| editor_style());

    let actions = row![
        text(screenshot_summary)
            .size(11)
            .color(Color::from_rgba8(148, 163, 184, 0.72)),
        Space::new().width(Length::Fill),
        subtle_action(
            if state.copilot_busy {
                "Capturando..."
            } else {
                "Tela"
            },
            (!state.copilot_busy).then_some(Message::CaptureCopilotScreenshot),
        ),
        subtle_action(
            "Limpar",
            state
                .copilot_screenshot
                .as_ref()
                .map(|_| Message::ClearCopilotScreenshot),
        ),
        primary_action(
            if state.copilot_busy {
                "Respondendo..."
            } else {
                "Perguntar"
            },
            (!state.copilot_busy).then_some(Message::SubmitCopilotRequest),
        ),
        subtle_action(
            "Copiar",
            state
                .copilot_messages
                .iter()
                .any(|message| message.role == CopilotRole::Assistant)
                .then_some(Message::CopyCopilotAnswer),
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![editor, actions].spacing(10).into()
}

fn user_message(content: &str) -> Element<'_, Message> {
    row![
        Space::new().width(Length::Fill),
        container(text(content.to_owned()).size(13).color(Color::WHITE))
            .padding([12, 14])
            .max_width(420)
            .style(|_| user_bubble_style()),
    ]
    .align_y(Alignment::Start)
    .into()
}

fn assistant_message(message: &CopilotChatMessage) -> Element<'_, Message> {
    container(markdown_message(message))
        .padding([12, 14])
        .max_width(520)
        .style(|_| assistant_bubble_style())
        .into()
}

fn markdown_message(message: &CopilotChatMessage) -> Element<'_, Message> {
    markdown::view(message.markdown_items.iter(), Theme::TokyoNightStorm)
        .map(Message::CopilotMarkdownLinkClicked)
}

fn error_card(error: &str) -> Element<'_, Message> {
    container(
        column![
            text("Issue").size(12).color(Color::from_rgb8(251, 146, 60)),
            text(error.to_owned())
                .size(13)
                .color(Color::from_rgb8(255, 207, 164)),
        ]
        .spacing(6),
    )
    .padding([12, 14])
    .style(|_| error_style())
    .into()
}

fn typing_indicator() -> Element<'static, Message> {
    container(
        text("Pensando...")
            .size(12)
            .color(Color::from_rgba8(148, 163, 184, 0.82)),
    )
    .padding([8, 12])
    .style(|_| assistant_bubble_style())
    .into()
}

fn context_label(state: &Overlay) -> String {
    if state.is_live_transcribing() || !state.live_completed_segments.is_empty() {
        return String::from("Contexto: transcript ao vivo");
    }

    if let Some(session_id) = state.selected_session_id {
        return format!("Contexto: sessao salva #{session_id}");
    }

    String::from("Contexto: pergunta direta")
}

fn mode_radio<'a>(
    label: &'static str,
    value: CopilotMode,
    selected: CopilotMode,
) -> Element<'a, Message> {
    radio(label, value, Some(selected), Message::CopilotModeChanged)
        .size(14)
        .spacing(8)
        .into()
}

fn primary_action<'a>(label: &'static str, on_press: Option<Message>) -> Element<'a, Message> {
    button(text(label).size(13))
        .padding([10, 14])
        .on_press_maybe(on_press)
        .style(|_, status| button_style(status, false))
        .into()
}

fn subtle_action<'a>(label: &'static str, on_press: Option<Message>) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([8, 12])
        .on_press_maybe(on_press)
        .style(|_, status| button_style(status, true))
        .into()
}

fn button_style(status: button::Status, subtle: bool) -> button::Style {
    let (bg, border, text_color) = if subtle {
        (
            Color::from_rgba8(255, 255, 255, 0.06),
            Color::from_rgba8(255, 255, 255, 0.12),
            Color::WHITE,
        )
    } else {
        (
            Color::from_rgba8(34, 211, 238, 0.82),
            Color::from_rgba8(103, 232, 249, 0.28),
            Color::from_rgb8(8, 14, 20),
        )
    };

    match status {
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(bg.scale_alpha(0.4))),
            border: Border {
                color: border.scale_alpha(0.4),
                width: 1.0,
                radius: 10.0.into(),
            },
            text_color: text_color.scale_alpha(0.45),
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(bg.scale_alpha(1.08))),
            border: Border {
                color: border.scale_alpha(1.1),
                width: 1.0,
                radius: 10.0.into(),
            },
            text_color,
            shadow: Shadow {
                color: bg.scale_alpha(0.18),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border,
                width: 1.0,
                radius: 10.0.into(),
            },
            text_color,
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

fn shell_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(8, 12, 18, 0.96)))
        .border(Border {
            color: Color::from_rgba8(34, 211, 238, 0.16),
            width: 1.0,
            radius: 18.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.22),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        })
}

fn meeting_shell_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(4, 8, 13, 0.78)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.08),
            width: 1.0,
            radius: 16.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.28),
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        })
}

fn user_bubble_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(34, 211, 238, 0.26)))
        .border(Border {
            color: Color::from_rgba8(103, 232, 249, 0.22),
            width: 1.0,
            radius: 16.0.into(),
        })
}

fn assistant_bubble_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(255, 255, 255, 0.05)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.08),
            width: 1.0,
            radius: 16.0.into(),
        })
}

fn meeting_answer_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(0, 0, 0, 0.26)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.06),
            width: 1.0,
            radius: 12.0.into(),
        })
}

fn editor_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(2, 6, 11, 0.72)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.08),
            width: 1.0,
            radius: 14.0.into(),
        })
}

fn error_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(120, 53, 15, 0.26)))
        .border(Border {
            color: Color::from_rgba8(249, 115, 22, 0.22),
            width: 1.0,
            radius: 12.0.into(),
        })
}
