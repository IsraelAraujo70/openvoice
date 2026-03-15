use crate::app::{HomeTab, Message, Overlay};
use crate::modules::copilot::application as copilot_application;
use crate::modules::copilot::domain::{CopilotChatMessage, CopilotMode, CopilotRole};
use iced::widget::{
    Space, button, column, container, markdown, row, scrollable, text, text_editor,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let latest_user = latest_message(state, CopilotRole::User);

    let header = row![
        mode_bar(state),
        Space::new().width(Length::Fill),
        subtle_action("Sessao", Some(Message::SwitchHomeTab(HomeTab::Copilot))),
        subtle_action("Fechar", Some(Message::CloseCopilotView)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let metadata = row![
        text(context_label(state))
            .size(11)
            .color(Color::from_rgba8(226, 232, 240, 0.58)),
        Space::new().width(Length::Fill),
        text(if state.copilot_include_transcript {
            "Transcript contextual ativo"
        } else {
            "Sem transcript automatico"
        })
        .size(11)
        .color(if state.copilot_include_transcript {
            Color::from_rgba8(34, 211, 238, 0.78)
        } else {
            Color::from_rgba8(148, 163, 184, 0.74)
        }),
    ]
    .align_y(Alignment::Center);

    let shell =
        container(column![header, metadata, composer_panel(state, latest_user),].spacing(12))
            .width(Length::Fill)
            .max_width(920)
            .padding([14, 18])
            .style(|_| overlay_shell_style());

    container(
        column![
            Space::new().height(Length::Fill),
            container(shell).width(Length::Fill).center_x(Length::Fill),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([0, 28])
    .into()
}

pub fn session_tab_content(state: &Overlay) -> Element<'_, Message> {
    let header = row![
        column![
            text("Copilot").size(22).color(Color::WHITE),
            text("Sessao persistente com markdown, transcript e screenshots")
                .size(12)
                .color(Color::from_rgba8(226, 232, 240, 0.60)),
        ]
        .spacing(2),
        Space::new().width(Length::Fill),
        subtle_action("Nova sessao", Some(Message::NewCopilotThread)),
        subtle_action("Overlay", Some(Message::OpenCopilotView)),
    ]
    .align_y(Alignment::Center);

    let shell = container(
        column![
            header,
            mode_bar(state),
            session_metadata(state),
            thread_selector(state),
            history_feed(state),
            full_composer(state),
        ]
        .spacing(16),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([22, 24])
    .style(|_| session_shell_style());

    container(shell)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn latest_message<'a>(state: &'a Overlay, role: CopilotRole) -> Option<&'a CopilotChatMessage> {
    state
        .copilot_messages
        .iter()
        .rev()
        .find(|message| message.role == role)
}

fn composer_panel<'a>(
    state: &'a Overlay,
    latest_user: Option<&'a CopilotChatMessage>,
) -> Element<'a, Message> {
    let screenshot_summary = state
        .copilot_screenshot
        .as_ref()
        .map(copilot_application::screenshot_summary)
        .unwrap_or_else(|| String::from("Sem screenshot"));

    let editor = container(
        text_editor(&state.copilot_input)
            .placeholder("Digite sua pergunta...")
            .on_action(Message::CopilotInputEdited)
            .height(Length::Fixed(60.0))
            .padding([10, 12]),
    )
    .width(Length::Fill)
    .style(|_| editor_style());

    let context_row = row![
        latest_user
            .map(|message| {
                text(format!(
                    "Ultima pergunta: {}",
                    truncate_inline(&message.content, 72)
                ))
                .size(10)
                .color(Color::from_rgba8(148, 163, 184, 0.68))
            })
            .unwrap_or_else(|| {
                text("Digite uma pergunta curta ou cole um pedido maior na sessao.")
                    .size(10)
                    .color(Color::from_rgba8(148, 163, 184, 0.68))
            }),
        Space::new().width(Length::Fill),
        text(screenshot_summary)
            .size(10)
            .color(Color::from_rgba8(148, 163, 184, 0.72)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let actions = row![
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
            if state.is_copilot_listening() {
                "Parar"
            } else {
                "Ouvir"
            },
            if state.copilot_busy {
                None
            } else if state.is_copilot_listening() {
                Some(Message::StopCopilotListen)
            } else {
                Some(Message::StartCopilotListen)
            },
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
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    container(column![context_row, editor, actions].spacing(10))
        .width(Length::Fill)
        .padding([12, 18])
        .style(|_| composer_shell_style())
        .into()
}

fn session_metadata(state: &Overlay) -> Element<'_, Message> {
    row![
        text(context_label(state))
            .size(11)
            .color(Color::from_rgba8(226, 232, 240, 0.62)),
        Space::new().width(Length::Fill),
        text(if state.copilot_include_transcript {
            "Transcript contextual ativo"
        } else {
            "Sem transcript automatico"
        })
        .size(12)
        .color(if state.copilot_include_transcript {
            Color::from_rgba8(34, 211, 238, 0.78)
        } else {
            Color::from_rgba8(148, 163, 184, 0.74)
        }),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn history_feed(state: &Overlay) -> Element<'_, Message> {
    let mut col = column![].spacing(14);

    if state.copilot_messages.is_empty() && state.copilot_error.is_none() {
        col = col.push(
            container(
                text("Use o overlay para perguntas rapidas ou continue a conversa aqui com historico completo.")
                    .size(12)
                    .color(Color::from_rgba8(148, 163, 184, 0.72)),
            )
            .padding([4, 2]),
        );
    }

    for message in &state.copilot_messages {
        col = col.push(match message.role {
            CopilotRole::User => user_message(&message.content),
            CopilotRole::Assistant => assistant_message(message),
        });
    }

    if let Some(error) = &state.copilot_error {
        col = col.push(error_card(error));
    }

    scrollable(col).height(Length::Fill).into()
}

fn thread_selector(state: &Overlay) -> Element<'_, Message> {
    if state.copilot_threads_loading {
        return text("Carregando sessoes do copiloto...")
            .size(12)
            .color(Color::from_rgba8(148, 163, 184, 0.72))
            .into();
    }

    if state.copilot_threads.is_empty() {
        return text("Nenhuma sessao salva ainda. A primeira resposta ja cria o historico.")
            .size(12)
            .color(Color::from_rgba8(148, 163, 184, 0.72))
            .into();
    }

    let mut items = column![].spacing(8);

    for thread in state.copilot_threads.iter().take(6) {
        let is_selected = state.selected_copilot_thread_id == Some(thread.id);
        let date_label = thread
            .created_at
            .split('T')
            .next()
            .unwrap_or(&thread.created_at);
        let session_label = thread
            .session_id
            .map(|id| format!("sessao #{id}"))
            .unwrap_or_else(|| String::from("sem transcript"));
        let subtitle = if thread.last_preview.trim().is_empty() {
            format!(
                "{} · {} turnos · {} · {}",
                thread.mode.label(),
                thread.turn_count,
                session_label,
                date_label
            )
        } else {
            format!(
                "{} · {} turnos · {} · {} · {}",
                thread.mode.label(),
                thread.turn_count,
                session_label,
                date_label,
                truncate_inline(&thread.last_preview, 52)
            )
        };

        let card = container(
            column![
                text(format!("Sessao #{}", thread.id))
                    .size(12)
                    .color(Color::WHITE),
                text(subtitle)
                    .size(11)
                    .color(Color::from_rgba8(148, 163, 184, 0.72)),
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .padding([10, 12])
        .style(move |_| thread_card_style(is_selected));

        items = items.push(
            button(card)
                .width(Length::Fill)
                .on_press(Message::CopilotThreadSelected(thread.id))
                .style(|_, status| transparent_button_style(status)),
        );
    }

    container(items).width(Length::Fill).padding([2, 0]).into()
}

fn full_composer(state: &Overlay) -> Element<'_, Message> {
    let screenshot_summary = state
        .copilot_screenshot
        .as_ref()
        .map(copilot_application::screenshot_summary)
        .unwrap_or_else(|| String::from("Sem screenshot"));

    let editor = container(
        text_editor(&state.copilot_input)
            .placeholder("Pergunte algo...")
            .on_action(Message::CopilotInputEdited)
            .height(Length::Fixed(120.0))
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
            if state.is_copilot_listening() {
                "Parar"
            } else {
                "Ouvir"
            },
            if state.copilot_busy {
                None
            } else if state.is_copilot_listening() {
                Some(Message::StopCopilotListen)
            } else {
                Some(Message::StartCopilotListen)
            },
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

fn mode_bar(state: &Overlay) -> Element<'_, Message> {
    row![
        mode_chip(CopilotMode::General, state.copilot_mode),
        mode_chip(CopilotMode::Interview, state.copilot_mode),
        mode_chip(CopilotMode::Meeting, state.copilot_mode),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn mode_chip(mode: CopilotMode, selected: CopilotMode) -> Element<'static, Message> {
    let is_active = mode == selected;

    button(text(mode.label()).size(12).color(if is_active {
        Color::WHITE
    } else {
        Color::from_rgba8(148, 163, 184, 0.78)
    }))
    .padding([7, 12])
    .on_press(Message::CopilotModeChanged(mode))
    .style(move |_, status| mode_chip_style(is_active, status))
    .into()
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
        .max_width(560)
        .style(|_| assistant_bubble_style())
        .into()
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

fn context_label(state: &Overlay) -> String {
    if state.is_live_transcribing() || !state.live_completed_segments.is_empty() {
        return String::from("Contexto: transcript ao vivo");
    }

    if let Some(session_id) = state.selected_session_id {
        return format!("Contexto: sessao salva #{session_id}");
    }

    String::from("Contexto: pergunta direta")
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

fn primary_action<'a>(label: &'static str, on_press: Option<Message>) -> Element<'a, Message> {
    button(text(label).size(13))
        .padding([9, 14])
        .on_press_maybe(on_press)
        .style(|_, status| action_button_style(status, false))
        .into()
}

fn subtle_action<'a>(label: &'static str, on_press: Option<Message>) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([8, 12])
        .on_press_maybe(on_press)
        .style(|_, status| action_button_style(status, true))
        .into()
}

fn action_button_style(status: button::Status, subtle: bool) -> button::Style {
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

fn mode_chip_style(active: bool, status: button::Status) -> button::Style {
    let base = if active {
        Color::from_rgba8(34, 211, 238, 0.24)
    } else {
        Color::from_rgba8(255, 255, 255, 0.04)
    };
    let border = if active {
        Color::from_rgba8(103, 232, 249, 0.32)
    } else {
        Color::from_rgba8(255, 255, 255, 0.08)
    };

    let background = match status {
        button::Status::Hovered => base.scale_alpha(1.15),
        button::Status::Disabled => base.scale_alpha(0.4),
        _ => base,
    };

    button::Style {
        background: Some(Background::Color(background)),
        border: Border {
            color: if matches!(status, button::Status::Disabled) {
                border.scale_alpha(0.4)
            } else {
                border
            },
            width: 1.0,
            radius: 999.0.into(),
        },
        text_color: if active {
            Color::WHITE
        } else {
            Color::from_rgba8(226, 232, 240, 0.82)
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn overlay_shell_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.76)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.10),
            width: 1.0,
            radius: 18.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.32),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        })
}

fn composer_shell_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(255, 255, 255, 0.03)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.08),
            width: 1.0,
            radius: 14.0.into(),
        })
}

fn thread_card_style(selected: bool) -> container::Style {
    container::Style::default()
        .background(Background::Color(if selected {
            Color::from_rgba8(34, 211, 238, 0.14)
        } else {
            Color::from_rgba8(255, 255, 255, 0.04)
        }))
        .border(Border {
            color: if selected {
                Color::from_rgba8(103, 232, 249, 0.30)
            } else {
                Color::from_rgba8(255, 255, 255, 0.08)
            },
            width: 1.0,
            radius: 12.0.into(),
        })
}

fn transparent_button_style(_status: button::Status) -> button::Style {
    button::Style {
        background: None,
        border: Border::default(),
        text_color: Color::WHITE,
        shadow: Shadow::default(),
        snap: false,
    }
}

fn session_shell_style() -> container::Style {
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
