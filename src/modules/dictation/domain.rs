pub const TARGET_SAMPLE_RATE: u32 = 16_000;
const DEFAULT_REFERER: &str = "https://github.com/IsraelAraujo70/openvoice";
const DEFAULT_APP_TITLE: &str = "OpenVoice";

#[derive(Debug, Clone)]
pub struct CapturedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl CapturedAudio {
    pub fn duration_seconds(&self) -> f32 {
        let frames = self.samples.len() as f32 / self.channels.max(1) as f32;
        frames / self.sample_rate.max(1) as f32
    }
}

#[derive(Debug, Clone)]
pub struct PreparedAudio {
    pub wav_base64: String,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone)]
pub struct DictationConfig {
    pub api_key: String,
    pub model: String,
    pub referer: String,
    pub app_title: String,
    pub prompt: String,
}

impl DictationConfig {
    pub fn from_settings(
        settings: &crate::modules::settings::domain::AppSettings,
    ) -> Result<Self, String> {
        if !settings.has_api_key() {
            return Err(String::from(
                "Cadastre uma OpenRouter API key antes de tentar gravar.",
            ));
        }

        Ok(Self {
            api_key: settings.openrouter_api_key.clone(),
            model: settings.openrouter_model.clone(),
            referer: String::from(DEFAULT_REFERER),
            app_title: String::from(DEFAULT_APP_TITLE),
            prompt: String::from(
                "Transcribe this audio exactly as spoken. Output only the transcription, nothing else. Preserve the original language and do not add formatting or commentary.",
            ),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DictationOutput {
    pub transcript: String,
    pub duration_seconds: f32,
}

impl DictationOutput {
    pub fn preview(&self) -> String {
        let preview = self.transcript.trim();

        if preview.chars().count() <= 160 {
            return preview.to_owned();
        }

        let mut shortened = preview.chars().take(157).collect::<String>();
        shortened.push_str("...");
        shortened
    }
}
