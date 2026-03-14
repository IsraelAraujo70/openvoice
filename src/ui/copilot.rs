use crate::app::{Message, Overlay};
use crate::modules::copilot::application as copilot_application;
use crate::modules::copilot::domain::CopilotMode;
use iced::widget::{
    Space, button, checkbox, column, container, radio, row, scrollable, text, text_editor,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let header = row![
        column![
            text("Copilot").size(24).color(Color::WHITE),
            text(context_label(state))
                .size(12)
                .color(Color::from_rgba8(226, 232, 240, 0.62)),
        ]
        .spacing(2),
        Space::new().width(Length::Fill),
        action_button("Fechar", Some(Message::CloseCopilotView), true),
    ]
    .align_y(Alignment::Center);

    let modes = row![
        mode_radio("General", CopilotMode::General, state.copilot_mode),
        mode_radio("Interview", CopilotMode::Interview, state.copilot_mode),
        mode_radio("Meeting", CopilotMode::Meeting, state.copilot_mode),
    ]
    .spacing(14)
    .wrap();

    let editor = container(
        text_editor(&state.copilot_input)
            .placeholder("Pergunte com contexto da sessao, entrevista ou agenda...")
            .on_action(Message::CopilotInputEdited)
            .height(Length::Fixed(180.0))
            .padding(14),
    )
    .width(Length::Fill)
    .style(|_| editor_style());

    let screenshot_summary = state
        .copilot_screenshot
        .as_ref()
        .map(copilot_application::screenshot_summary)
        .unwrap_or_else(|| String::from("Sem screenshot anexado"));

    let screenshot_row = row![
        text(screenshot_summary)
            .size(12)
            .color(Color::from_rgba8(148, 163, 184, 0.82)),
        Space::new().width(Length::Fill),
        action_button(
            if state.copilot_busy {
                "Capturando..."
            } else {
                "Capturar tela"
            },
            (!state.copilot_busy).then_some(Message::CaptureCopilotScreenshot),
            false,
        ),
        action_button(
            "Remover",
            state
                .copilot_screenshot
                .as_ref()
                .map(|_| Message::ClearCopilotScreenshot),
            true,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let toggles = checkbox(state.copilot_include_transcript)
        .label("Incluir transcript atual automaticamente")
        .on_toggle(Message::CopilotIncludeTranscriptChanged)
        .text_size(13);

    let submit_row = row![
        action_button(
            if state.copilot_busy {
                "Respondendo..."
            } else {
                "Perguntar"
            },
            (!state.copilot_busy).then_some(Message::SubmitCopilotRequest),
            false,
        ),
        action_button(
            "Copiar resposta",
            state
                .copilot_answer
                .as_ref()
                .map(|_| Message::CopyCopilotAnswer),
            true,
        ),
    ]
    .spacing(10);

    let body = column![
        modes,
        editor,
        screenshot_row,
        toggles,
        submit_row,
        answer_block(state),
    ]
    .spacing(14);

    let shell = container(column![header, scrollable(body).height(Length::Fill)].spacing(18))
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

fn answer_block(state: &Overlay) -> Element<'_, Message> {
    if let Some(error) = &state.copilot_error {
        return container(
            column![
                text("Issue").size(12).color(Color::from_rgb8(251, 146, 60)),
                text(error).size(13).color(Color::from_rgb8(255, 207, 164)),
            ]
            .spacing(6),
        )
        .padding(14)
        .style(|_| error_style())
        .into();
    }

    if let Some(answer) = &state.copilot_answer {
        return container(
            scrollable(
                text(answer.clone())
                    .size(13)
                    .color(Color::from_rgba8(226, 232, 240, 0.9)),
            )
            .height(Length::Fill),
        )
        .padding(16)
        .height(Length::Fill)
        .style(|_| answer_style())
        .into();
    }

    container(
        text("O copiloto usa transcript, sessao salva e screenshot opcional para responder.")
            .size(12)
            .color(Color::from_rgba8(148, 163, 184, 0.72)),
    )
    .padding([8, 2])
    .into()
}

fn action_button<'a>(
    label: &'static str,
    on_press: Option<Message>,
    subtle: bool,
) -> Element<'a, Message> {
    button(text(label).size(13))
        .padding([10, 14])
        .on_press_maybe(on_press)
        .style(move |_, status| button_style(status, subtle))
        .into()
}

fn button_style(status: button::Status, subtle: bool) -> button::Style {
    let (bg, border, text_color) = if subtle {
        (
            Color::from_rgba8(255, 255, 255, 0.08),
            Color::from_rgba8(255, 255, 255, 0.14),
            Color::WHITE,
        )
    } else {
        (
            Color::from_rgba8(34, 211, 238, 0.84),
            Color::from_rgba8(103, 232, 249, 0.24),
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
                color: border.scale_alpha(1.15),
                width: 1.0,
                radius: 10.0.into(),
            },
            text_color,
            shadow: Shadow {
                color: bg.scale_alpha(0.16),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 18.0,
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
            color: Color::from_rgba8(34, 211, 238, 0.18),
            width: 1.0,
            radius: 18.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.22),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 26.0,
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

fn answer_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(255, 255, 255, 0.05)))
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
