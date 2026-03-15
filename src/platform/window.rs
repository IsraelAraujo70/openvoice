use crate::platform::monitors::{self, MonitorGeometry};
use iced::{Point, Size, window};
use std::env;

const HUD_WIDTH: f32 = 380.0;
const HUD_HEIGHT: f32 = 96.0;
const HOME_WIDTH: f32 = 700.0;
const HOME_HEIGHT: f32 = 800.0;
const COPILOT_OVERLAY_WIDTH: f32 = 860.0;
const COPILOT_OVERLAY_HEIGHT: f32 = 268.0;
const COPILOT_RESPONSE_WIDTH: f32 = 860.0;
const COPILOT_RESPONSE_HEIGHT: f32 = 360.0;
const SUBTITLE_WIDTH: f32 = 860.0;
const SUBTITLE_HEIGHT: f32 = 80.0;
const DEFAULT_APPLICATION_ID_PREFIX: &str = "openvoice";

pub fn hud_settings() -> window::Settings {
    let primary = monitors::focused_monitor_geometry();

    window::Settings {
        decorations: false,
        transparent: true,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        size: primary
            .map(hud_size)
            .unwrap_or_else(|| Size::new(HUD_WIDTH, HUD_HEIGHT)),
        position: primary
            .map(|monitor| window::Position::Specific(hud_position(monitor)))
            .unwrap_or(window::Position::Specific(Point::new(48.0, 48.0))),
        exit_on_close_request: false,
        platform_specific: platform_specific("main"),
        ..Default::default()
    }
}

pub fn home_window_settings() -> window::Settings {
    let primary = monitors::focused_monitor_geometry();

    window::Settings {
        decorations: false,
        transparent: true,
        resizable: true,
        level: window::Level::Normal,
        size: primary
            .map(home_size)
            .unwrap_or_else(|| Size::new(HOME_WIDTH, HOME_HEIGHT)),
        position: primary
            .map(|monitor| window::Position::Specific(home_position(monitor)))
            .unwrap_or(window::Position::Specific(Point::ORIGIN)),
        exit_on_close_request: false,
        platform_specific: platform_specific("main"),
        ..Default::default()
    }
}

pub fn subtitle_window_settings(primary: Option<MonitorGeometry>) -> window::Settings {
    window::Settings {
        decorations: false,
        transparent: true,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        size: Size::new(SUBTITLE_WIDTH, SUBTITLE_HEIGHT),
        position: primary
            .map(|m| window::Position::Specific(subtitle_position(m)))
            .unwrap_or(window::Position::Specific(Point::new(200.0, 900.0))),
        exit_on_close_request: false,
        platform_specific: platform_specific("subtitle"),
        ..Default::default()
    }
}

pub fn copilot_overlay_window_settings(primary: Option<MonitorGeometry>) -> window::Settings {
    window::Settings {
        decorations: false,
        transparent: true,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        size: primary
            .map(copilot_overlay_size)
            .unwrap_or_else(|| Size::new(COPILOT_OVERLAY_WIDTH, COPILOT_OVERLAY_HEIGHT)),
        position: primary
            .map(|monitor| window::Position::Specific(copilot_overlay_position(monitor)))
            .unwrap_or(window::Position::Specific(Point::new(140.0, 720.0))),
        exit_on_close_request: false,
        platform_specific: platform_specific("copilot-overlay"),
        ..Default::default()
    }
}

pub fn copilot_response_window_settings(primary: Option<MonitorGeometry>) -> window::Settings {
    window::Settings {
        decorations: false,
        transparent: true,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        size: primary
            .map(copilot_response_size)
            .unwrap_or_else(|| Size::new(COPILOT_RESPONSE_WIDTH, COPILOT_RESPONSE_HEIGHT)),
        position: primary
            .map(|monitor| window::Position::Specific(copilot_response_position(monitor)))
            .unwrap_or(window::Position::Specific(Point::new(140.0, 500.0))),
        exit_on_close_request: false,
        platform_specific: platform_specific("copilot-response"),
        ..Default::default()
    }
}

fn application_id_prefix() -> String {
    normalize_application_id_prefix(env::var("OPENVOICE_APPLICATION_ID_PREFIX").ok().as_deref())
}

fn normalize_application_id_prefix(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_APPLICATION_ID_PREFIX)
        .to_owned()
}

pub fn application_id_for_role(role: &str) -> String {
    compose_application_id(&application_id_prefix(), role)
}

fn compose_application_id(prefix: &str, role: &str) -> String {
    if role.trim().is_empty() {
        prefix.to_owned()
    } else {
        format!("{prefix}-{role}")
    }
}

#[cfg(target_os = "linux")]
fn platform_specific(role: &str) -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: application_id_for_role(role),
        override_redirect: false,
    }
}

#[cfg(not(target_os = "linux"))]
fn platform_specific(_role: &str) -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}

fn hud_size(monitor: MonitorGeometry) -> Size {
    Size::new(
        HUD_WIDTH.min(monitor.size.width.max(HUD_WIDTH)),
        HUD_HEIGHT.min(monitor.size.height.max(HUD_HEIGHT)),
    )
}

fn hud_position(monitor: MonitorGeometry) -> Point {
    Point::new(
        monitor.position.x + (monitor.size.width - HUD_WIDTH).max(32.0) - 32.0,
        monitor.position.y + 28.0,
    )
}

fn home_size(monitor: MonitorGeometry) -> Size {
    Size::new(
        HOME_WIDTH.min((monitor.size.width - 96.0).max(480.0)),
        HOME_HEIGHT.min((monitor.size.height - 96.0).max(520.0)),
    )
}

fn home_position(monitor: MonitorGeometry) -> Point {
    let size = home_size(monitor);

    Point::new(
        monitor.position.x + ((monitor.size.width - size.width) / 2.0).max(32.0),
        monitor.position.y + ((monitor.size.height - size.height) / 2.0).max(32.0),
    )
}

fn subtitle_position(monitor: MonitorGeometry) -> Point {
    // Bottom-center, with a margin from the bottom edge
    Point::new(
        monitor.position.x + ((monitor.size.width - SUBTITLE_WIDTH) / 2.0).max(0.0),
        monitor.position.y + monitor.size.height - SUBTITLE_HEIGHT - 96.0,
    )
}

fn copilot_overlay_size(monitor: MonitorGeometry) -> Size {
    Size::new(
        COPILOT_OVERLAY_WIDTH.min((monitor.size.width - 96.0).max(560.0)),
        COPILOT_OVERLAY_HEIGHT.min((monitor.size.height - 140.0).max(220.0)),
    )
}

fn copilot_overlay_position(monitor: MonitorGeometry) -> Point {
    let size = copilot_overlay_size(monitor);

    Point::new(
        monitor.position.x + ((monitor.size.width - size.width) / 2.0).max(24.0),
        monitor.position.y + monitor.size.height - size.height - 112.0,
    )
}

fn copilot_response_size(monitor: MonitorGeometry) -> Size {
    Size::new(
        COPILOT_RESPONSE_WIDTH.min((monitor.size.width - 96.0).max(560.0)),
        COPILOT_RESPONSE_HEIGHT.min((monitor.size.height - 220.0).max(240.0)),
    )
}

fn copilot_response_position(monitor: MonitorGeometry) -> Point {
    let response = copilot_response_size(monitor);
    let overlay_pos = copilot_overlay_position(monitor);

    Point::new(
        monitor.position.x + ((monitor.size.width - response.width) / 2.0).max(24.0),
        (overlay_pos.y - response.height - 16.0).max(monitor.position.y + 48.0),
    )
}

#[cfg(test)]
mod tests {
    use super::{compose_application_id, normalize_application_id_prefix};

    #[test]
    fn falls_back_to_default_prefix_when_env_is_missing() {
        assert_eq!(normalize_application_id_prefix(None), "openvoice");
    }

    #[test]
    fn trims_custom_prefix() {
        assert_eq!(
            normalize_application_id_prefix(Some("  openvoice-dev  ")),
            "openvoice-dev"
        );
    }

    #[test]
    fn builds_suffix_based_window_ids() {
        assert_eq!(
            compose_application_id("openvoice", "subtitle"),
            "openvoice-subtitle"
        );
    }
}
