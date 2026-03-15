use crate::platform::hyprland;
use iced::{Point, Size};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorGeometry {
    pub size: Size,
    pub position: Point,
}

pub fn focused_monitor_geometry() -> Option<MonitorGeometry> {
    hyprland::focused_monitor().map(|monitor| MonitorGeometry {
        size: Size::new(monitor.width, monitor.height),
        position: Point::new(monitor.x, monitor.y),
    })
}
