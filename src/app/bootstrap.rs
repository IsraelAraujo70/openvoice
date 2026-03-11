use crate::app::message::Message;
use crate::app::state::{Overlay, boot};
use crate::app::update::update;
use crate::ui;
use crate::ui::theme;

pub fn run() -> iced::Result {
    iced::daemon(boot, update, ui::view)
        .title(Overlay::title)
        .theme(|state: &Overlay, _window| theme::app_theme(state))
        .style(theme::app_style)
        .subscription(subscription)
        .run()
}

fn subscription(_state: &Overlay) -> iced::Subscription<Message> {
    iced::Subscription::batch([
        iced::window::close_requests().map(Message::WindowCloseRequested),
        iced::keyboard::listen().map(Message::KeyEvent),
        iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(iced::window::Event::Moved(point)) => {
                Some(Message::WindowMoved(point))
            }
            _ => None,
        }),
    ])
}
