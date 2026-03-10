use crate::app::message::Message;
use crate::platform::window::MonitorGeometry;
use iced::{Task, window};

#[derive(Debug, Clone)]
pub struct Overlay {
    pub window_id: Option<window::Id>,
    pub passthrough_enabled: bool,
    pub status: &'static str,
    pub primary_monitor: Option<MonitorGeometry>,
}

impl Overlay {
    pub fn title(&self) -> String {
        if self.passthrough_enabled {
            String::from("OpenVoice Overlay [passthrough]")
        } else {
            String::from("OpenVoice Overlay [interactive]")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayConfig {
    pub start_with_passthrough: bool,
}

impl OverlayConfig {
    pub fn from_env() -> Self {
        let start_with_passthrough = std::env::var("OPENVOICE_MOUSE_PASSTHROUGH")
            .ok()
            .as_deref()
            .map(|value| matches!(value, "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(true);

        Self {
            start_with_passthrough,
        }
    }
}

pub fn boot() -> (Overlay, Task<Message>) {
    let config = OverlayConfig::from_env();
    let primary_monitor = crate::platform::window::detect_primary_monitor_geometry();

    (
        Overlay {
            window_id: None,
            passthrough_enabled: config.start_with_passthrough,
            status: if config.start_with_passthrough {
                "Mouse passthrough requested at startup."
            } else {
                "Interactive mode. Press P to enable passthrough."
            },
            primary_monitor,
        },
        Task::none(),
    )
}
