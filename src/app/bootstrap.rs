use crate::app::message::Message;
use crate::app::state::{Overlay, boot};
use crate::app::update::update;
use crate::platform::window;
use crate::ui::overlay;
use crate::ui::theme;

pub fn run() -> iced::Result {
    iced::application(boot, update, overlay::view)
        .title(Overlay::title)
        .window(window::initial_settings())
        .theme(theme::app_theme)
        .style(theme::app_style)
        .subscription(subscription)
        .run()
}

fn subscription(_state: &Overlay) -> iced::Subscription<Message> {
    iced::Subscription::batch([
        iced::window::open_events().map(Message::WindowOpened),
        iced::keyboard::listen().map(Message::KeyEvent),
    ])
}
