use crate::modules::copilot::domain::ScreenshotAttachment;
use crate::platform::hyprland;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct ScreenshotCommand<'a> {
    program: &'a str,
    args: Vec<String>,
}

pub fn capture_primary_display() -> Result<ScreenshotAttachment, String> {
    if !hyprland::is_hyprland_session() {
        return Err(String::from(
            "Captura de tela do OpenVoice agora suporta apenas sessoes Hyprland.",
        ));
    }

    let path = temp_png_path();
    let commands = hyprland_screenshot_commands(&path);

    let mut errors = Vec::new();

    for command in commands {
        match run_capture_command(&command, &path) {
            Ok(bytes) => {
                let _ = fs::remove_file(&path);
                return Ok(ScreenshotAttachment {
                    bytes,
                    mime_type: String::from("image/png"),
                });
            }
            Err(error) => errors.push(error),
        }
    }

    let joined = errors.join(" | ");
    Err(format!("Nao consegui capturar a tela. {joined}"))
}

fn run_capture_command(command: &ScreenshotCommand<'_>, path: &PathBuf) -> Result<Vec<u8>, String> {
    let output = Command::new(command.program)
        .args(&command.args)
        .output()
        .map_err(|error| format!("{} indisponivel: {error}", command.program))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{} retornou status {}: {}",
            command.program,
            output.status,
            stderr.trim()
        ));
    }

    let bytes = fs::read(path).map_err(|error| {
        format!(
            "{} executou, mas nao consegui ler screenshot temporario: {error}",
            command.program
        )
    })?;

    if bytes.is_empty() {
        return Err(format!("{} gerou screenshot vazio.", command.program));
    }

    Ok(bytes)
}

fn temp_png_path() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    std::env::temp_dir().join(format!("openvoice-copilot-{stamp}.png"))
}

fn hyprland_screenshot_commands(path: &PathBuf) -> Vec<ScreenshotCommand<'static>> {
    let path_string = path.display().to_string();

    vec![
        ScreenshotCommand {
            program: "grim",
            args: vec![path_string.clone()],
        },
        ScreenshotCommand {
            program: "wayshot",
            args: vec!["-f".into(), path_string],
        },
    ]
}
