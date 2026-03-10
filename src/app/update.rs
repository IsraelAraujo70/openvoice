use crate::app::message::Message;
use crate::app::state::Overlay;
use iced::keyboard::{self, Key, key::Named};
use iced::{Point, Task, window};

pub fn update(state: &mut Overlay, message: Message) -> Task<Message> {
    match message {
        Message::WindowOpened(id) => {
            state.window_id = Some(id);

            let mut tasks = vec![window::set_level(id, window::Level::AlwaysOnTop)];

            if let Some(primary) = state.primary_monitor {
                tasks.push(window::resize(id, primary.size));
                tasks.push(window::move_to(id, primary.position));
                tasks.push(window::set_level(id, window::Level::AlwaysOnTop));
            } else {
                tasks.push(window::monitor_size(id).map(Message::MonitorSizeLoaded));
            }

            if state.passthrough_enabled {
                tasks.push(window::enable_mouse_passthrough(id));
            }

            Task::batch(tasks)
        }
        Message::MonitorSizeLoaded(Some(size)) => state
            .window_id
            .map(|window_id| {
                Task::batch([
                    window::resize(window_id, size),
                    window::move_to(window_id, Point::ORIGIN),
                    window::set_level(window_id, window::Level::AlwaysOnTop),
                ])
            })
            .unwrap_or_else(Task::none),
        Message::MonitorSizeLoaded(None) => Task::none(),
        Message::KeyEvent(event) => match event {
            keyboard::Event::KeyPressed {
                key, physical_key, ..
            } => match key.as_ref() {
                Key::Named(Named::Escape) => Task::done(Message::Quit),
                _ if matches!(key.to_latin(physical_key), Some('p')) => {
                    Task::done(Message::TogglePassthrough)
                }
                _ => Task::none(),
            },
            _ => Task::none(),
        },
        Message::TogglePassthrough => {
            state.passthrough_enabled = !state.passthrough_enabled;
            state.status = if state.passthrough_enabled {
                "Passthrough enabled. Clicks should reach the app behind."
            } else {
                "Passthrough disabled. The overlay can receive input again."
            };

            state.window_id.map_or_else(Task::none, |window_id| {
                let passthrough_task = if state.passthrough_enabled {
                    window::enable_mouse_passthrough(window_id)
                } else {
                    window::disable_mouse_passthrough(window_id)
                };

                Task::batch([
                    passthrough_task,
                    window::set_level(window_id, window::Level::AlwaysOnTop),
                ])
            })
        }
        Message::Quit => state
            .window_id
            .map(window::close)
            .unwrap_or_else(Task::none),
    }
}
