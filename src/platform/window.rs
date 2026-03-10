use iced::{window, Point, Size};
use std::process::Command;

const HUD_WIDTH: f32 = 380.0;
const HUD_HEIGHT: f32 = 96.0;
const SETTINGS_WIDTH: f32 = 540.0;
const SETTINGS_HEIGHT: f32 = 760.0;

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

pub fn settings_window_settings() -> window::Settings {
    let primary = detect_primary_monitor_geometry();

    window::Settings {
        decorations: false,
        transparent: true,
        resizable: true,
        level: window::Level::Normal,
        size: primary
            .map(settings_size)
            .unwrap_or_else(|| Size::new(SETTINGS_WIDTH, SETTINGS_HEIGHT)),
        position: primary
            .map(|monitor| window::Position::Specific(settings_position(monitor)))
            .unwrap_or(window::Position::Specific(Point::ORIGIN)),
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

fn settings_size(monitor: MonitorGeometry) -> Size {
    Size::new(
        SETTINGS_WIDTH.min((monitor.size.width - 96.0).max(420.0)),
        SETTINGS_HEIGHT.min((monitor.size.height - 96.0).max(520.0)),
    )
}

fn settings_position(monitor: MonitorGeometry) -> Point {
    let size = settings_size(monitor);

    Point::new(
        monitor.position.x + ((monitor.size.width - size.width) / 2.0).max(32.0),
        monitor.position.y + ((monitor.size.height - size.height) / 2.0).max(32.0),
    )
}

pub fn detect_primary_monitor_geometry() -> Option<MonitorGeometry> {
    read_xrandr("--listactivemonitors")
        .and_then(|stdout| parse_xrandr_listactivemonitors(&stdout))
        .or_else(|| read_xrandr("--query").and_then(|stdout| parse_xrandr_query_primary(&stdout)))
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

/// Clamp a HUD position so it stays within the given monitor bounds.
/// The HUD must remain fully visible — no edge can escape the monitor.
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
    use super::{parse_geometry_token, parse_xrandr_listactivemonitors, Point, Size};

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
}
