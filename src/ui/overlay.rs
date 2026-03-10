use crate::app::{Message, Overlay, OverlayPhase};
use crate::ui::components::chrome_button::{self, ButtonKind};
use crate::ui::components::status_indicator;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let accent = phase_color(state.phase);

    let mic_action = if state.is_recording() {
        Some(Message::StopDictation)
    } else if state.can_start_dictation() {
        Some(Message::StartDictation)
    } else {
        None
    };

    let status_label = match state.phase {
        OverlayPhase::Idle => "READY",
        OverlayPhase::Recording => "REC",
        OverlayPhase::Processing => "WAIT",
        OverlayPhase::Success => "COPIED",
        OverlayPhase::Error => "ERROR",
    };

    let info_text = state.error.as_deref().unwrap_or(&state.hint);
    let info_color = if state.error.is_some() {
        Color::from_rgba8(249, 115, 22, 0.72)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.28)
    };

    let hud = container(
        column![
            row![
                status_indicator::view(status_label, accent),
                Space::new().width(Length::Fill),
                chrome_button::view("", mic_action, ButtonKind::Mic(accent)),
                chrome_button::view("⚙", Some(Message::OpenSettingsView), ButtonKind::Ghost),
                chrome_button::view("✕", Some(Message::Quit), ButtonKind::Ghost),
            ]
            .spacing(8)
            .width(Length::Fill)
            .align_y(Alignment::Center),
            text(info_text).size(11).color(info_color),
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([14, 18])
    .style(move |_| hud_style(accent));

    container(hud)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn phase_color(phase: OverlayPhase) -> Color {
    match phase {
        OverlayPhase::Idle => Color::from_rgba(1.0, 1.0, 1.0, 0.4),
        OverlayPhase::Recording => Color::from_rgb8(239, 68, 68),
        OverlayPhase::Processing => Color::from_rgb8(234, 179, 8),
        OverlayPhase::Success => Color::from_rgb8(34, 197, 94),
        OverlayPhase::Error => Color::from_rgb8(249, 115, 22),
    }
}

fn hud_style(accent: Color) -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.78)))
        .border(Border {
            color: accent.scale_alpha(0.12),
            width: 1.0,
            radius: 16.0.into(),
        })
        .shadow(Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 20.0,
        })
        .color(Color::WHITE)
}
