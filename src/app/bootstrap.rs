use crate::app::message::Message;
use crate::app::state::{Overlay, boot};
use crate::app::update::update;
use crate::platform::window;
use crate::ui;
use crate::ui::theme;

pub fn run() -> iced::Result {
    iced::application(boot, update, ui::view)
        .title(Overlay::title)
        .window(window::hud_settings())
        .theme(theme::app_theme)
        .style(theme::app_style)
        .subscription(subscription)
        .run()
}

fn subscription(_state: &Overlay) -> iced::Subscription<Message> {
    iced::Subscription::batch([
        iced::window::open_events().map(Message::WindowOpened),
        iced::window::close_requests().map(Message::WindowCloseRequested),
        iced::keyboard::listen().map(Message::KeyEvent),
    ])
}
