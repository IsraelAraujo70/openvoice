use crate::app::message::Message;
use crate::modules::audio::application::ActiveCaptureSession;
use crate::modules::settings::application as settings_application;
use crate::modules::settings::domain::{AppSettings, SettingsForm};
use crate::platform::window::MonitorGeometry;
use iced::{window, Point, Task};

pub struct Overlay {
    pub main_window_id: Option<window::Id>,
    pub passthrough_enabled: bool,
    pub scene: Scene,
    pub primary_monitor: Option<MonitorGeometry>,
    pub hud_position: Option<Point>,
    pub phase: OverlayPhase,
    pub hint: String,
    pub error: Option<String>,
    pub preview: Option<String>,
    pub settings: AppSettings,
    pub settings_form: SettingsForm,
    pub is_saving_settings: bool,
    pub settings_note: Option<String>,
    pub active_capture_session: Option<ActiveCaptureSession>,
}

impl Overlay {
    pub fn title(&self) -> String {
        if matches!(self.scene, Scene::Settings) {
            String::from("OpenVoice Settings")
        } else if self.passthrough_enabled {
            String::from("OpenVoice HUD [passthrough]")
        } else {
            String::from("OpenVoice HUD [interactive]")
        }
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.phase, OverlayPhase::Recording)
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.phase, OverlayPhase::Processing)
    }

    pub fn can_start_dictation(&self) -> bool {
        self.settings.has_api_key() && !self.is_processing() && !self.is_saving_settings
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPhase {
    Idle,
    Recording,
    Processing,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    Hud,
    Settings,
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
            .unwrap_or(false);

        Self {
            start_with_passthrough,
        }
    }
}

pub fn boot() -> (Overlay, Task<Message>) {
    let config = OverlayConfig::from_env();
    let primary_monitor = crate::platform::window::detect_primary_monitor_geometry();
    let (settings, settings_error) = match settings_application::load_settings() {
        Ok(settings) => (settings, None),
        Err(error) => (AppSettings::default(), Some(error)),
    };
    let settings_form = SettingsForm::from(&settings);
    let missing_api_key = (!settings.has_api_key())
        .then(|| String::from("Cadastre sua OpenRouter API key no painel de settings abaixo."));

    (
        Overlay {
            main_window_id: None,
            passthrough_enabled: config.start_with_passthrough,
            scene: Scene::Hud,
            primary_monitor,
            hud_position: None,
            phase: OverlayPhase::Idle,
            hint: if config.start_with_passthrough {
                String::from("Passthrough ativo. Pressione P para interagir.")
            } else {
                String::new()
            },
            error: settings_error.or(missing_api_key),
            preview: None,
            settings,
            settings_form,
            is_saving_settings: false,
            settings_note: None,
            active_capture_session: None,
        },
        Task::none(),
    )
}
