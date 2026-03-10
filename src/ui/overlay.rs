use crate::app::{Message, Overlay, OverlayPhase};
use crate::ui::components::chrome_button::{self, ButtonEmphasis};
use crate::ui::components::waveform;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

pub fn view(state: &Overlay) -> Element<'_, Message> {
    let accent = match state.phase {
        OverlayPhase::Idle => Color::from_rgb8(238, 236, 214),
        OverlayPhase::Recording => Color::from_rgb8(248, 113, 113),
        OverlayPhase::Processing => Color::from_rgb8(250, 204, 21),
        OverlayPhase::Success => Color::from_rgb8(74, 222, 128),
        OverlayPhase::Error => Color::from_rgb8(251, 146, 60),
    };
    let mic_action = if state.is_recording() {
        Some(Message::StopDictation)
    } else if state.can_start_dictation() {
        Some(Message::StartDictation)
    } else {
        None
    };

    let hud = container(
        column![
            row![
                chrome_button::view("✕", Some(Message::Quit), ButtonEmphasis::Neutral),
                Space::new().width(Length::Fill),
                chrome_button::view("●", mic_action, ButtonEmphasis::Primary),
            ]
            .width(Length::Fill)
            .align_y(Alignment::Start),
            container(waveform::view(state.status_title(), &state.hint, accent))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            row![
                chrome_button::view("⚙", Some(Message::OpenSettingsView), ButtonEmphasis::Accent),
                Space::new().width(Length::Fill),
                text(match state.phase {
                    OverlayPhase::Idle => "READY",
                    OverlayPhase::Recording => "REC",
                    OverlayPhase::Processing => "WAIT",
                    OverlayPhase::Success => "COPIED",
                    OverlayPhase::Error => "ERROR",
                })
                .size(12)
                .color(Color::from_rgba8(226, 232, 240, 0.82)),
            ]
            .width(Length::Fill)
            .align_y(Alignment::End),
            state
                .error
                .as_ref()
                .map(|error| {
                    container(text(error).size(12).color(Color::from_rgb8(255, 207, 164)))
                        .padding([8, 10])
                        .style(|_| warning_panel())
                })
                .unwrap_or_else(|| container(text(""))),
        ]
        .spacing(12),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(18)
    .style(move |_| {
        container::Style::default()
            .background(Background::Color(Color::from_rgba8(8, 10, 14, 0.94)))
            .border(Border {
                color: accent.scale_alpha(0.35),
                width: 1.0,
                radius: 28.0.into(),
            })
            .shadow(Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.34),
                offset: iced::Vector::new(0.0, 18.0),
                blur_radius: 34.0,
            })
            .color(Color::from_rgb8(248, 250, 252))
    });

    container(hud)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .into()
}

fn warning_panel() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(120, 53, 15, 0.42)))
        .border(Border {
            color: Color::from_rgba8(251, 146, 60, 0.25),
            width: 1.0,
            radius: 16.0.into(),
        })
        .color(Color::from_rgb8(255, 207, 164))
}
