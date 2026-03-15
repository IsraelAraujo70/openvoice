use iced::{window, Point, Size};
use serde_json::Value;
use std::process::Command;

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

#[derive(Debug, Clone, Copy)]
pub struct MonitorGeometry {
    pub size: Size,
    pub position: Point,
}

pub fn hud_settings() -> window::Settings {
    let primary = detect_primary_monitor_geometry();

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
        ..Default::default()
    }
}

pub fn home_window_settings() -> window::Settings {
    let primary = detect_primary_monitor_geometry();

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
        ..Default::default()
    }
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

pub fn detect_primary_monitor_geometry() -> Option<MonitorGeometry> {
    read_xrandr("--listactivemonitors")
        .and_then(|stdout| parse_xrandr_listactivemonitors(&stdout))
        .or_else(|| read_xrandr("--query").and_then(|stdout| parse_xrandr_query_primary(&stdout)))
        .or_else(detect_hyprctl_monitor)
}

fn detect_hyprctl_monitor() -> Option<MonitorGeometry> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    parse_hyprctl_monitors(&stdout)
}

fn parse_hyprctl_monitors(stdout: &str) -> Option<MonitorGeometry> {
    let monitors: Vec<Value> = serde_json::from_str(stdout).ok()?;

    let focused = monitors
        .iter()
        .find(|m| m.get("focused").and_then(Value::as_bool).unwrap_or(false))
        .or_else(|| monitors.first())?;

    let width = focused.get("width")?.as_f64()? as f32;
    let height = focused.get("height")?.as_f64()? as f32;
    let x = focused.get("x")?.as_f64()? as f32;
    let y = focused.get("y")?.as_f64()? as f32;

    Some(MonitorGeometry {
        size: Size::new(width, height),
        position: Point::new(x, y),
    })
}

fn read_xrandr(arg: &str) -> Option<String> {
    let output = Command::new("xrandr").arg(arg).output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}

fn parse_xrandr_listactivemonitors(stdout: &str) -> Option<MonitorGeometry> {
    stdout
        .lines()
        .skip(1)
        .find(|line| line.contains("*"))
        .and_then(parse_listactivemonitors_line)
}

fn parse_listactivemonitors_line(line: &str) -> Option<MonitorGeometry> {
    line.split_whitespace()
        .find(|token| token.contains('+') && token.contains('x'))
        .and_then(parse_geometry_token)
}

fn parse_xrandr_query_primary(stdout: &str) -> Option<MonitorGeometry> {
    stdout
        .lines()
        .find(|line| line.contains(" connected primary "))
        .and_then(|line| {
            line.split_whitespace()
                .find(|token| token.contains('x') && token.contains('+'))
                .and_then(parse_geometry_token)
        })
}

fn parse_geometry_token(token: &str) -> Option<MonitorGeometry> {
    let (size, position) = token.split_once('+')?;
    let (width, height) = size.split_once('x')?;
    let (x, y) = position.split_once('+')?;

    Some(MonitorGeometry {
        size: Size::new(
            width.split('/').next()?.parse::<f32>().ok()?,
            height.split('/').next()?.parse::<f32>().ok()?,
        ),
        position: Point::new(x.parse::<f32>().ok()?, y.parse::<f32>().ok()?),
    })
}

#[allow(dead_code)]
pub fn clamp_hud_to_monitor(position: Point, monitor: MonitorGeometry) -> Point {
    let min_x = monitor.position.x;
    let min_y = monitor.position.y;
    let max_x = monitor.position.x + monitor.size.width - HUD_WIDTH;
    let max_y = monitor.position.y + monitor.size.height - HUD_HEIGHT;

    Point::new(
        position.x.clamp(min_x, max_x),
        position.y.clamp(min_y, max_y),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        parse_geometry_token, parse_hyprctl_monitors, parse_xrandr_listactivemonitors, Point, Size,
    };

    #[test]
    fn parses_listactivemonitors_primary_output() {
        let geometry = parse_xrandr_listactivemonitors(
            "Monitors: 3\n 0: +*DP-3 1920/520x1080/320+1360+0  DP-3\n 1: +DP-1 1920/530x1080/300+3280+0  DP-1\n",
        )
        .expect("primary monitor geometry");

        assert_eq!(geometry.size, Size::new(1920.0, 1080.0));
        assert_eq!(geometry.position, Point::new(1360.0, 0.0));
    }

    #[test]
    fn parses_raw_geometry_token() {
        let geometry = parse_geometry_token("1360x765+0+171").expect("geometry");

        assert_eq!(geometry.size, Size::new(1360.0, 765.0));
        assert_eq!(geometry.position, Point::new(0.0, 171.0));
    }

    #[test]
    fn parses_hyprctl_monitors_focused() {
        let json = r#"[
            {"width":1920,"height":1080,"x":0,"y":0,"focused":true},
            {"width":1920,"height":1080,"x":-1920,"y":0,"focused":false}
        ]"#;

        let geometry = parse_hyprctl_monitors(json).expect("focused monitor");

        assert_eq!(geometry.size, Size::new(1920.0, 1080.0));
        assert_eq!(geometry.position, Point::new(0.0, 0.0));
    }

    #[test]
    fn parses_hyprctl_monitors_fallback_to_first() {
        let json = r#"[
            {"width":2560,"height":1440,"x":0,"y":0,"focused":false}
        ]"#;

        let geometry = parse_hyprctl_monitors(json).expect("first monitor");

        assert_eq!(geometry.size, Size::new(2560.0, 1440.0));
        assert_eq!(geometry.position, Point::new(0.0, 0.0));
    }
}
