use crate::app::{HomeTab, Message, Overlay};
use crate::modules::live_transcription::infrastructure::db::format_iso_for_display;
use crate::ui::{sessions, settings};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let header = row![
        column![
            text("OpenVoice").size(26).color(Color::WHITE),
            text("Copiloto pessoal Linux-native")
                .size(13)
                .color(Color::from_rgba8(226, 232, 240, 0.60)),
        ]
        .spacing(2),
        Space::new().width(Length::Fill),
        close_btn(),
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center);

    let tabs = tab_bar(state.home_tab);

    let content: Element<'_, Message> = match state.home_tab {
        HomeTab::Home => scrollable(home_content(state)).height(Length::Fill).into(),
        HomeTab::Sessions => sessions::tab_content(state),
        HomeTab::Settings => settings::tab_content(state),
    };

    let shell = container(column![header, tabs, content].spacing(18))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([24, 28])
        .style(|_| shell_style());

    container(shell)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .into()
}

fn home_content(state: &Overlay) -> Element<'_, Message> {
    let listen_action = if state.is_live_transcribing() {
        Some(Message::StopRealtimeTranscription)
    } else if state.can_start_realtime_transcription() {
        Some(Message::StartRealtimeTranscription)
    } else {
        None
    };

    let dictation_action = if state.is_dictation_recording() {
        Some(Message::StopDictation)
    } else if state.can_start_dictation() {
        Some(Message::StartDictation)
    } else {
        None
    };

    let listen_label = if state.is_live_transcribing() {
        "Parar Escuta"
    } else {
        "Ouvir Desktop"
    };

    let dictation_label = if state.is_dictation_recording() {
        "Parar Ditado"
    } else {
        "Ditar"
    };

    let cards = column![
        action_card(
            listen_label,
            "Transcricao realtime do audio do sistema",
            "Ctrl+Shift+L",
            listen_action,
            false,
        ),
        action_card(
            dictation_label,
            "Gravar microfone e transcrever via OpenRouter",
            "Ctrl+Shift+D",
            dictation_action,
            false,
        ),
        action_card(
            "Perguntar Algo",
            "Chat contextual com LLM",
            "em breve",
            None,
            true,
        ),
    ]
    .spacing(12);

    let mut content = column![cards].spacing(16);

    // Status hints
    let status = status_hints(state);
    content = content.push(status);

    // Recent sessions (up to 3)
    if !state.sessions_list.is_empty() {
        content = content.push(recent_sessions(state));
    }

    if let Some(err) = &state.error {
        content = content.push(
            container(
                column![
                    text("Issue").size(13).color(Color::from_rgb8(251, 146, 60)),
                    text(err).size(14).color(Color::from_rgb8(255, 207, 164)),
                ]
                .spacing(6),
            )
            .padding(14)
            .style(|_| error_card_style()),
        );
    }

    content.into()
}

fn status_hints(state: &Overlay) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();

    // Realtime transcription status
    if state.is_live_transcribing() {
        let seg_count = state.live_completed_segments.len() + state.live_persisted_segment_count;
        let detail = if seg_count > 0 {
            format!("Realtime ativo \u{2022} {seg_count} segmento(s)")
        } else {
            String::from("Realtime ativo \u{2022} aguardando audio...")
        };
        items.push(status_pill(&detail, Color::from_rgb8(34, 211, 238)));
    }

    // Dictation status
    if state.is_dictation_recording() {
        items.push(status_pill(
            "Ditado gravando...",
            Color::from_rgb8(251, 146, 60),
        ));
    } else if state.is_processing() {
        items.push(status_pill(
            "Processando ditado...",
            Color::from_rgb8(251, 146, 60),
        ));
    }

    // Provider status
    if !state.settings.has_api_key() {
        items.push(status_pill(
            "OpenRouter API key nao configurada",
            Color::from_rgb8(248, 113, 113),
        ));
    }
    if !state.settings.has_openai_realtime_api_key() {
        items.push(status_pill(
            "OpenAI Realtime API key nao configurada",
            Color::from_rgb8(248, 113, 113),
        ));
    }

    column(items).spacing(6).into()
}

fn status_pill<'a>(label: &str, accent: Color) -> Element<'a, Message> {
    let dot = text("\u{25CF} ").size(10).color(accent);
    let lbl = text(label.to_string())
        .size(12)
        .color(Color::from_rgba8(226, 232, 240, 0.75));

    container(row![dot, lbl].align_y(Alignment::Center))
        .padding([6, 12])
        .style(move |_| status_pill_style())
        .into()
}

fn recent_sessions(state: &Overlay) -> Element<'_, Message> {
    let header_row = row![
        text("Sessoes recentes")
            .size(14)
            .color(Color::from_rgba8(226, 232, 240, 0.80)),
        Space::new().width(Length::Fill),
        button(
            text("Ver todas")
                .size(11)
                .color(Color::from_rgb8(34, 211, 238)),
        )
        .on_press(Message::SwitchHomeTab(HomeTab::Sessions))
        .style(|_, _| ghost_btn_style())
        .padding([2, 6]),
    ]
    .align_y(Alignment::Center);

    let mut col = column![].spacing(6);
    for session in state.sessions_list.iter().take(3) {
        let date = format_iso_for_display(&session.started_at);
        let lang = session.language.as_deref().unwrap_or("?");
        let segs = session.segment_count;
        let preview = if session.preview.len() > 80 {
            format!("{}...", &session.preview[..80])
        } else {
            session.preview.clone()
        };

        let card = container(
            column![
                row![
                    text(date)
                        .size(12)
                        .color(Color::from_rgba8(226, 232, 240, 0.85)),
                    Space::new().width(Length::Fill),
                    text(format!("{lang} \u{00B7} {segs} seg"))
                        .size(10)
                        .color(Color::from_rgba8(148, 163, 184, 0.55)),
                ]
                .align_y(Alignment::Center),
                text(preview)
                    .size(11)
                    .color(Color::from_rgba8(148, 163, 184, 0.65)),
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .padding([10, 14])
        .style(|_| recent_card_style());

        col = col.push(
            button(card)
                .width(Length::Fill)
                .on_press(Message::SwitchHomeTab(HomeTab::Sessions))
                .style(|_, _| transparent_btn_style()),
        );
    }

    column![header_row, col].spacing(10).into()
}

fn action_card<'a>(
    title: &'a str,
    description: &'a str,
    badge: &'a str,
    on_press: Option<Message>,
    is_coming_soon: bool,
) -> Element<'a, Message> {
    let badge_el: Element<'a, Message> = if is_coming_soon {
        container(
            text(badge)
                .size(10)
                .color(Color::from_rgba8(148, 163, 184, 0.7)),
        )
        .padding([3, 8])
        .style(|_| badge_soon_style())
        .into()
    } else {
        container(
            text(badge)
                .size(10)
                .color(Color::from_rgba8(34, 211, 238, 0.85)),
        )
        .padding([3, 8])
        .style(|_| badge_shortcut_style())
        .into()
    };

    let card_content = row![
        column![
            text(title).size(15).color(if is_coming_soon {
                Color::from_rgba8(226, 232, 240, 0.4)
            } else {
                Color::WHITE
            }),
            text(description).size(12).color(if is_coming_soon {
                Color::from_rgba8(148, 163, 184, 0.35)
            } else {
                Color::from_rgba8(148, 163, 184, 0.80)
            }),
        ]
        .spacing(4),
        Space::new().width(Length::Fill),
        badge_el,
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    let btn = button(
        container(card_content)
            .width(Length::Fill)
            .padding([16, 20]),
    )
    .width(Length::Fill)
    .on_press_maybe(if is_coming_soon { None } else { on_press })
    .style(move |_, status| action_card_btn_style(is_coming_soon, status));

    btn.into()
}

fn tab_bar(active: HomeTab) -> Element<'static, Message> {
    row![
        tab_button("Inicio", HomeTab::Home, active),
        tab_button("Sessoes", HomeTab::Sessions, active),
        tab_button("Configuracoes", HomeTab::Settings, active),
    ]
    .spacing(4)
    .into()
}

fn tab_button(label: &'static str, tab: HomeTab, active: HomeTab) -> Element<'static, Message> {
    let is_active = tab == active;

    button(text(label).size(13).color(if is_active {
        Color::WHITE
    } else {
        Color::from_rgba8(148, 163, 184, 0.65)
    }))
    .on_press(Message::SwitchHomeTab(tab))
    .padding([8, 16])
    .style(move |_, _| tab_btn_style(is_active))
    .into()
}

fn close_btn<'a>() -> Element<'a, Message> {
    button(
        text("\u{2715}")
            .size(14)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
    )
    .on_press(Message::CloseHomeView)
    .style(|_, _| ghost_btn_style())
    .padding([4, 8])
    .into()
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

fn shell_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(4, 8, 14, 0.96)))
        .border(Border {
            color: Color::from_rgba8(94, 234, 212, 0.16),
            width: 1.0,
            radius: 24.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.34),
            offset: iced::Vector::new(0.0, 24.0),
            blur_radius: 42.0,
        })
        .color(Color::from_rgb8(248, 250, 252))
}

fn tab_btn_style(is_active: bool) -> button::Style {
    button::Style {
        background: if is_active {
            Some(Background::Color(Color::from_rgba8(34, 211, 238, 0.12)))
        } else {
            None
        },
        border: Border {
            color: if is_active {
                Color::from_rgba8(34, 211, 238, 0.24)
            } else {
                Color::TRANSPARENT
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Color::WHITE,
        snap: false,
    }
}

fn action_card_btn_style(is_disabled: bool, status: button::Status) -> button::Style {
    let (bg_alpha, border_alpha) = match (is_disabled, status) {
        (true, _) => (0.03, 0.06),
        (_, button::Status::Hovered) => (0.12, 0.18),
        _ => (0.06, 0.10),
    };

    button::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, bg_alpha))),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, border_alpha),
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Color::WHITE,
        snap: false,
    }
}

fn badge_shortcut_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(34, 211, 238, 0.10)))
        .border(Border {
            color: Color::from_rgba8(34, 211, 238, 0.20),
            width: 1.0,
            radius: 6.0.into(),
        })
}

fn badge_soon_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(148, 163, 184, 0.08)))
        .border(Border {
            color: Color::from_rgba8(148, 163, 184, 0.14),
            width: 1.0,
            radius: 6.0.into(),
        })
}

fn error_card_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(120, 18, 24, 0.28)))
        .border(Border {
            color: Color::from_rgba8(248, 113, 113, 0.24),
            width: 1.0,
            radius: 14.0.into(),
        })
        .color(Color::from_rgb8(255, 207, 164))
}

fn status_pill_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(255, 255, 255, 0.04)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.07),
            width: 1.0,
            radius: 8.0.into(),
        })
}

fn ghost_btn_style() -> button::Style {
    button::Style {
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Color::WHITE,
        snap: false,
    }
}

fn recent_card_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(255, 255, 255, 0.04)))
        .border(Border {
            color: Color::from_rgba8(255, 255, 255, 0.07),
            width: 1.0,
            radius: 10.0.into(),
        })
}

fn transparent_btn_style() -> button::Style {
    button::Style {
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Color::WHITE,
        snap: false,
    }
}
