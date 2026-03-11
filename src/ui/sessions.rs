use crate::app::{Message, Overlay};
use crate::modules::live_transcription::infrastructure::db::{
    SessionSummary, format_iso_for_display,
};
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let header = row![
        text("Sessoes gravadas").size(16).color(Color::WHITE),
        Space::new().width(Length::Fill),
        close_btn(),
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    let body: Element<'_, Message> = if state.sessions_loading {
        text("Carregando sessoes...").size(13).color(muted()).into()
    } else if let Some(err) = &state.sessions_error {
        text(format!("Erro: {err}"))
            .size(13)
            .color(Color::from_rgb8(249, 115, 22))
            .into()
    } else if state.sessions_list.is_empty() {
        text("Nenhuma sessao gravada ainda. Inicie a transcricao realtime para comecar.")
            .size(13)
            .color(muted())
            .into()
    } else {
        sessions_list(state)
    };

    let content = column![header, body].spacing(20);

    container(
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([20, 24])
            .style(|_| panel_style()),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn sessions_list(state: &Overlay) -> Element<'_, Message> {
    let mut col = column![].spacing(8);

    for session in &state.sessions_list {
        let is_selected = state.selected_session_id == Some(session.id);
        col = col.push(session_card(state, session, is_selected));
    }

    scrollable(col).height(Length::Fill).into()
}

fn session_card<'a>(
    state: &'a Overlay,
    session: &'a SessionSummary,
    is_selected: bool,
) -> Element<'a, Message> {
    let date_label = format_iso_for_display(&session.started_at);
    let stopped_label = session
        .stopped_at
        .as_deref()
        .map(format_iso_for_display)
        .unwrap_or_else(|| String::from("em andamento"));
    let model_label = session.model.as_deref().unwrap_or("modelo desconhecido");
    let lang_label = session.language.as_deref().unwrap_or("idioma desconhecido");
    let seg_label = format!(
        "{} segmento{}",
        session.segment_count,
        if session.segment_count == 1 { "" } else { "s" }
    );

    let summary_row = row![
        column![
            text(date_label).size(13).color(Color::WHITE),
            text(format!("{lang_label} · {model_label} · {seg_label}"))
                .size(11)
                .color(muted()),
            text(format!("Finalizada: {stopped_label}"))
                .size(11)
                .color(muted()),
        ]
        .spacing(3),
        Space::new().width(Length::Fill),
        expand_btn(session.id, is_selected),
    ]
    .align_y(Alignment::Center)
    .spacing(8);

    let card: Element<'_, Message> = if is_selected {
        let detail = session_detail(state, session);
        column![summary_row, detail].spacing(12).into()
    } else {
        summary_row.into()
    };

    container(card)
        .width(Length::Fill)
        .padding([12, 16])
        .style(move |_| card_style(is_selected))
        .into()
}

fn session_detail<'a>(state: &'a Overlay, _session: &'a SessionSummary) -> Element<'a, Message> {
    if state.selected_session_loading {
        return text("Carregando transcricao...")
            .size(12)
            .color(muted())
            .into();
    }

    if state.selected_session_segments.is_empty() {
        return text("Nenhum segmento encontrado.")
            .size(12)
            .color(muted())
            .into();
    }

    let transcript_text = state.selected_session_segments.join(" ");

    let actions = row![
        action_btn("Copiar", Message::CopySessionTranscript),
        action_btn("Exportar para Obsidian", Message::CopySessionTranscript), // placeholder
    ]
    .spacing(8);

    column![
        text(_session.preview.clone())
            .size(12)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.62)),
        container(
            scrollable(
                text(transcript_text)
                    .size(12)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.85))
            )
            .height(180)
        )
        .width(Length::Fill)
        .padding([10, 14])
        .style(|_| transcript_box_style()),
        actions,
    ]
    .spacing(10)
    .into()
}

// ---------------------------------------------------------------------------
// Small widget helpers
// ---------------------------------------------------------------------------

fn close_btn<'a>() -> Element<'a, Message> {
    button(
        text("✕")
            .size(14)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
    )
    .on_press(Message::CloseSessionsView)
    .style(|_, _| ghost_btn_style())
    .padding([4, 8])
    .into()
}

fn expand_btn<'a>(session_id: i64, is_selected: bool) -> Element<'a, Message> {
    let label = if is_selected { "▲" } else { "▼" };
    button(
        text(label)
            .size(12)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.55)),
    )
    .on_press(if is_selected {
        Message::SessionSelected(0) // collapse — use id 0 as sentinel for "deselect"
    } else {
        Message::SessionSelected(session_id)
    })
    .style(|_, _| ghost_btn_style())
    .padding([4, 8])
    .into()
}

fn action_btn<'a>(label: &'static str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(12).color(Color::WHITE))
        .on_press(msg)
        .style(|_, _| action_btn_style())
        .padding([6, 14])
        .into()
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

fn muted() -> Color {
    Color::from_rgba(1.0, 1.0, 1.0, 0.38)
}

fn panel_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba(0.07, 0.07, 0.10, 0.96)))
        .border(Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 16.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 28.0,
        })
        .color(Color::WHITE)
}

fn card_style(selected: bool) -> container::Style {
    let bg_alpha = if selected { 0.12 } else { 0.06 };
    container::Style::default()
        .background(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, bg_alpha)))
        .border(Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, if selected { 0.18 } else { 0.07 }),
            width: 1.0,
            radius: 10.0.into(),
        })
}

fn transcript_box_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.28)))
        .border(Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.07),
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

fn action_btn_style() -> button::Style {
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
}
