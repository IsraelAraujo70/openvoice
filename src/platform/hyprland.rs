use serde::Deserialize;
use std::{env, process::Command};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorGeometry {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Deserialize)]
struct HyprlandMonitor {
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    #[serde(default)]
    focused: bool,
}

pub fn is_hyprland_session() -> bool {
    env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        || env::var("XDG_CURRENT_DESKTOP")
            .ok()
            .map(|desktop| desktop.to_ascii_lowercase().contains("hyprland"))
            .unwrap_or(false)
}

pub fn is_wayland_session() -> bool {
    env::var("XDG_SESSION_TYPE")
        .ok()
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
        || env::var_os("WAYLAND_DISPLAY").is_some()
}

pub fn focused_monitor_geometry() -> Option<MonitorGeometry> {
    if !is_hyprland_session() {
        return None;
    }

    let stdout = run_hyprctl(&["monitors", "-j"])?;
    parse_monitors(&stdout)
}

fn run_hyprctl(args: &[&str]) -> Option<String> {
    let output = Command::new("hyprctl").args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}

fn parse_monitors(stdout: &str) -> Option<MonitorGeometry> {
    let monitors: Vec<HyprlandMonitor> = serde_json::from_str(stdout).ok()?;

    let focused = monitors
        .iter()
        .find(|monitor| monitor.focused)
        .or_else(|| monitors.first())?;

    Some(MonitorGeometry {
        width: focused.width,
        height: focused.height,
        x: focused.x,
        y: focused.y,
    })
}

#[cfg(test)]
mod tests {
    use super::{MonitorGeometry, parse_monitors};

    #[test]
    fn parses_focused_monitor() {
        let json = r#"[
            {"width":1920,"height":1080,"x":0,"y":0,"focused":true},
            {"width":1920,"height":1080,"x":-1920,"y":0,"focused":false}
        ]"#;

        assert_eq!(
            parse_monitors(json),
            Some(MonitorGeometry {
                width: 1920.0,
                height: 1080.0,
                x: 0.0,
                y: 0.0,
            })
        );
    }

    #[test]
    fn falls_back_to_first_monitor() {
        let json = r#"[
            {"width":2560,"height":1440,"x":0,"y":0}
        ]"#;

        assert_eq!(
            parse_monitors(json),
            Some(MonitorGeometry {
                width: 2560.0,
                height: 1440.0,
                x: 0.0,
                y: 0.0,
            })
        );
    }
}
