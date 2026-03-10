use crate::app::{Message, Overlay};
use iced::widget::{column, container, text};
use iced::{Background, Border, Color, Element, Length, Shadow};

pub fn view(_state: &Overlay) -> Element<'_, Message> {
    let badge = container(column![text("OPENVOICE").size(14), text("WIP").size(32),].spacing(8))
        .padding([14, 16])
        .style(|_| {
            container::Style::default()
                .background(Background::Color(Color::from_rgba8(11, 15, 24, 0.78)))
                .border(Border {
                    color: Color::from_rgba8(120, 140, 190, 0.35),
                    width: 1.0,
                    radius: 18.0.into(),
                })
                .shadow(Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.32),
                    offset: iced::Vector::new(0.0, 14.0),
                    blur_radius: 32.0,
                })
                .color(Color::from_rgb8(242, 245, 252))
        });

    container(badge)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}
