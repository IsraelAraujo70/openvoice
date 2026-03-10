use iced::{Point, Size, window};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub struct MonitorGeometry {
    pub size: Size,
    pub position: Point,
}

pub fn initial_settings() -> window::Settings {
    let primary = detect_primary_monitor_geometry();

    window::Settings {
        decorations: false,
        transparent: true,
        resizable: false,
        level: window::Level::AlwaysOnTop,
        size: primary
            .map(|monitor| monitor.size)
            .unwrap_or_else(|| Size::new(1280.0, 720.0)),
        position: primary
            .map(|monitor| window::Position::Specific(monitor.position))
            .unwrap_or(window::Position::Specific(Point::ORIGIN)),
        exit_on_close_request: true,
        ..Default::default()
    }
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

#[cfg(test)]
mod tests {
    use super::{Point, Size, parse_geometry_token, parse_xrandr_listactivemonitors};

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
