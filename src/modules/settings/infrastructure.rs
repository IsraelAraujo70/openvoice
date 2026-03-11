use crate::modules::settings::domain::AppSettings;
use std::fs;
use std::path::PathBuf;

pub fn load_settings() -> Result<AppSettings, String> {
    let path = settings_path()?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let contents =
        fs::read_to_string(&path).map_err(|error| format!("Falha ao ler settings: {error}"))?;

    serde_json::from_str::<AppSettings>(&contents)
        .map(AppSettings::normalized)
        .map_err(|error| {
            format!(
                "Falha ao interpretar settings em {}: {error}",
                path.display()
            )
        })
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar pasta de settings: {error}"))?;
    }

    let contents = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("Falha ao serializar settings: {error}"))?;

    fs::write(&path, contents)
        .map_err(|error| format!("Falha ao salvar settings em {}: {error}", path.display()))
}

fn settings_path() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| {
            String::from("Nao consegui descobrir a pasta de configuracao do usuario.")
        })?;

    Ok(base.join("openvoice").join("settings.json"))
}
