use crate::app::{Message, Overlay};
use crate::modules::copilot::domain::CopilotMode;
use crate::modules::settings::domain::{
    SUPPORTED_OPENAI_REALTIME_LANGUAGES, SUPPORTED_OPENAI_REALTIME_PROFILES,
};
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

pub fn tab_content(state: &Overlay) -> Element<'_, Message> {
    let save_settings = action_button(
        if state.is_saving_settings {
            "Saving..."
        } else {
            "Save Settings"
        },
        (!state.is_saving_settings).then_some(Message::SaveSettings),
    );

    let openai_auth_action = if state.has_openai_credentials {
        action_button(
            if state.is_openai_authenticating {
                "Working..."
            } else {
                "Log out"
            },
            (!state.is_openai_authenticating).then_some(Message::LogoutOpenAi),
        )
    } else if state.pending_openai_oauth.is_some() {
        action_button(
            if state.is_openai_authenticating {
                "Waiting for callback..."
            } else {
                "OAuth pending"
            },
            None,
        )
    } else {
        action_button(
            if state.is_openai_authenticating {
                "Signing in..."
            } else {
                "Sign in with ChatGPT"
            },
            (!state.is_openai_authenticating).then_some(Message::StartOpenAiOAuthLogin),
        )
    };

    let manual_callback_block = state.pending_openai_oauth.as_ref().map(|flow| {
        container(
            column![
                text(format!(
                    "Caso o navegador nao finalize sozinho, cole a callback URL de {}.",
                    flow.redirect_uri
                ))
                .size(12)
                .color(Color::from_rgba8(148, 163, 184, 0.88)),
                text_input(
                    "http://localhost:1455/auth/callback?...",
                    &state.openai_callback_url_input
                )
                .on_input(Message::OpenAiOAuthCallbackUrlChanged)
                .padding([12, 14]),
                action_button(
                    if state.is_openai_authenticating {
                        "Finishing..."
                    } else {
                        "Use callback URL"
                    },
                    (!state.is_openai_authenticating).then_some(Message::SubmitOpenAiOAuthCallback)
                ),
            ]
            .spacing(12),
        )
        .padding(14)
        .style(|_| card_style())
    });

    let content = column![
        container(
            column![
                section_title("OpenRouter"),
                text(
                    "OpenRouter e usado no fluxo de gravacao do microfone: grava, envia o audio final e copia a transcricao para o clipboard."
                )
                .size(12)
                .color(Color::from_rgba8(148, 163, 184, 0.88)),
                text_input(
                    "OpenRouter API key",
                    &state.settings_form.openrouter_api_key
                )
                .on_input(Message::SettingsApiKeyChanged)
                .secure(true)
                .padding([12, 14]),
                text_input("Model", &state.settings_form.openrouter_model)
                    .on_input(Message::SettingsModelChanged)
                    .padding([12, 14]),
                row![
                    save_settings,
                    state
                        .settings_note
                        .as_ref()
                        .map(|note| {
                            text(note)
                                .size(12)
                                .color(Color::from_rgba8(148, 163, 184, 0.88))
                        })
                        .unwrap_or_else(|| text("")),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(14),
        )
        .padding(18)
        .style(|_| card_style()),
        container(
            column![
                section_title("OpenAI Realtime"),
                text(
                    "OpenAI Realtime e usado na transcricao ao vivo do audio do sistema. Aqui entram a API key da Platform e o modelo de transcricao."
                )
                .size(12)
                .color(Color::from_rgba8(148, 163, 184, 0.88)),
                text_input("OpenAI API key", &state.settings_form.openai_realtime_api_key)
                    .on_input(Message::SettingsOpenAiRealtimeApiKeyChanged)
                    .secure(true)
                    .padding([12, 14]),
                text_input("Transcription model", &state.settings_form.openai_realtime_model)
                    .on_input(Message::SettingsOpenAiRealtimeModelChanged)
                    .padding([12, 14]),
                pick_list(
                    SUPPORTED_OPENAI_REALTIME_LANGUAGE_OPTIONS,
                    selected_language_option(&state.settings_form.openai_realtime_language),
                    |option| {
                        Message::SettingsOpenAiRealtimeLanguageChanged(
                            option.code().to_owned()
                        )
                    }
                )
                .placeholder("Language"),
                pick_list(
                    SUPPORTED_OPENAI_REALTIME_PROFILE_OPTIONS,
                    selected_profile_option(&state.settings_form.openai_realtime_profile),
                    |option| {
                        Message::SettingsOpenAiRealtimeProfileChanged(
                            option.code().to_owned()
                        )
                    }
                )
                .placeholder("Realtime profile"),
            ]
            .spacing(14),
        )
        .padding(18)
        .style(|_| card_style()),
        container(
            column![
                section_title("Copilot"),
                text(
                    "O copiloto usa a sessao OAuth do ChatGPT para responder com contexto de transcript, sessao salva e screenshot opcional."
                )
                .size(12)
                .color(Color::from_rgba8(148, 163, 184, 0.88)),
                text_input("Copilot model", &state.settings_form.copilot_model)
                    .on_input(Message::SettingsCopilotModelChanged)
                    .padding([12, 14]),
                pick_list(
                    SUPPORTED_COPILOT_MODE_OPTIONS,
                    selected_copilot_mode_option(&state.settings_form.copilot_default_mode),
                    |mode| Message::SettingsCopilotDefaultModeChanged(mode.code().to_owned())
                )
                .placeholder("Default copilot mode"),
                checkbox(state.settings_form.copilot_auto_include_transcript)
                    .label("Include transcript by default")
                    .on_toggle(Message::SettingsCopilotAutoIncludeTranscriptChanged)
                    .text_size(13),
                checkbox(state.settings_form.copilot_save_history)
                    .label("Save copilot history locally")
                    .on_toggle(Message::SettingsCopilotSaveHistoryChanged)
                    .text_size(13),
            ]
            .spacing(14),
        )
        .padding(18)
        .style(|_| card_style()),
        container(
            column![
                section_title("OpenAI OAuth"),
                openai_auth_action,
                text(
                    "Mantenha o login ChatGPT aqui para copiloto, title generation e futuros fluxos OAuth. Realtime nao depende mais desta sessao."
                )
                .size(12)
                .color(Color::from_rgba8(148, 163, 184, 0.88)),
                manual_callback_block
                    .map(Element::from)
                    .unwrap_or_else(|| Space::new().height(0).into()),
            ]
            .spacing(14),
        )
        .padding(18)
        .style(|_| card_style()),
        container(
            column![
                section_title("Runtime"),
                status_row("Clipboard", "microphone transcript copied after processing"),
                status_row("Storage", "settings in ~/.config/openvoice; system capture stays available for a future feature"),
                status_row("Audio", "dictation uses microphone only in the current HUD flow"),
                status_row(
                    "Realtime auth",
                    if state.settings.has_openai_realtime_api_key() {
                        "configured via API key"
                    } else {
                        "missing OpenAI API key"
                    },
                ),
                status_row(
                    "ChatGPT OAuth",
                    if state.has_openai_credentials {
                        "connected for copilot"
                    } else {
                        "not signed in"
                    },
                ),
                status_row("Copilot model", state.settings.copilot_model.clone()),
                status_row("Copilot default mode", state.settings.copilot_default_mode.clone()),
                status_row(
                    "Copilot transcript",
                    if state.settings.copilot_auto_include_transcript {
                        "included by default"
                    } else {
                        "manual only"
                    },
                ),
                status_row(
                    "OAuth account",
                    state.openai_account_label.as_deref().unwrap_or("unknown"),
                ),
            ]
            .spacing(12),
        )
        .padding(18)
        .style(|_| card_style()),
        state
            .error
            .as_ref()
            .map(|error| {
                container(
                    column![
                        text("Issue")
                            .size(13)
                            .color(Color::from_rgb8(251, 146, 60)),
                        text(error)
                            .size(16)
                            .color(Color::from_rgb8(255, 207, 164)),
                    ]
                    .spacing(8),
                )
                .padding(18)
                .style(|_| error_style())
            })
            .unwrap_or_else(|| {
                container(
                    text("This panel is generated by OpenVoice. Validate OpenRouter and OpenAI realtime before relying on it in production.")
                        .size(12)
                        .color(Color::from_rgba8(226, 232, 240, 0.66)),
                )
                .padding([0, 4])
            }),
    ]
    .spacing(18);

    scrollable(content).height(Length::Fill).into()
}

fn section_title(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(13)
        .color(Color::from_rgba8(148, 163, 184, 0.9))
        .into()
}

fn status_row(label: &'static str, value: impl Into<String>) -> Element<'static, Message> {
    let value = value.into();

    row![
        text(label)
            .size(13)
            .color(Color::from_rgba8(148, 163, 184, 0.82)),
        Space::new().width(Length::Fill),
        text(value).size(13),
    ]
    .into()
}

fn action_button<'a>(label: impl Into<String>, on_press: Option<Message>) -> Element<'a, Message> {
    button(text(label.into()).size(14))
        .padding([12, 16])
        .width(Length::Shrink)
        .on_press_maybe(on_press)
        .style(|_, status| match status {
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgba8(34, 211, 238, 0.12))),
                border: Border {
                    color: Color::from_rgba8(34, 211, 238, 0.18),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                text_color: Color::from_rgba8(226, 232, 240, 0.4),
                shadow: Shadow::default(),
                snap: false,
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Color::from_rgba8(34, 211, 238, 0.92))),
                border: Border {
                    color: Color::from_rgba8(103, 232, 249, 0.95),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                text_color: Color::from_rgb8(8, 14, 20),
                shadow: Shadow {
                    color: Color::from_rgba8(34, 211, 238, 0.18),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                snap: false,
            },
            _ => button::Style {
                background: Some(Background::Color(Color::from_rgba8(34, 211, 238, 0.82))),
                border: Border {
                    color: Color::from_rgba8(34, 211, 238, 0.24),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                text_color: Color::from_rgb8(8, 14, 20),
                shadow: Shadow {
                    color: Color::from_rgba8(34, 211, 238, 0.12),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 18.0,
                },
                snap: false,
            },
        })
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LanguageOption {
    label: &'static str,
    code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProfileOption {
    label: &'static str,
    code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CopilotModeOption {
    label: &'static str,
    code: &'static str,
}

impl ProfileOption {
    const fn new(label: &'static str, code: &'static str) -> Self {
        Self { label, code }
    }

    fn code(self) -> &'static str {
        self.code
    }
}

impl CopilotModeOption {
    const fn new(label: &'static str, code: &'static str) -> Self {
        Self { label, code }
    }

    fn code(self) -> &'static str {
        self.code
    }
}

impl LanguageOption {
    const fn new(label: &'static str, code: &'static str) -> Self {
        Self { label, code }
    }

    fn code(self) -> &'static str {
        self.code
    }
}

impl std::fmt::Display for LanguageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.label.fmt(f)
    }
}

impl std::fmt::Display for ProfileOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.label.fmt(f)
    }
}

impl std::fmt::Display for CopilotModeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.label.fmt(f)
    }
}

const SUPPORTED_OPENAI_REALTIME_LANGUAGE_OPTIONS: [LanguageOption; 8] = [
    LanguageOption::new("Auto", ""),
    LanguageOption::new("Portuguese", "pt"),
    LanguageOption::new("English", "en"),
    LanguageOption::new("German", "de"),
    LanguageOption::new("Spanish", "es"),
    LanguageOption::new("French", "fr"),
    LanguageOption::new("Italian", "it"),
    LanguageOption::new("Japanese", "ja"),
];

const SUPPORTED_OPENAI_REALTIME_PROFILE_OPTIONS: [ProfileOption; 3] = [
    ProfileOption::new("Caption", "caption"),
    ProfileOption::new("Balanced", "balanced"),
    ProfileOption::new("Accuracy", "accuracy"),
];

const SUPPORTED_COPILOT_MODE_OPTIONS: [CopilotModeOption; 3] = [
    CopilotModeOption::new("General", "general"),
    CopilotModeOption::new("Interview", "interview"),
    CopilotModeOption::new("Meeting", "meeting"),
];

fn selected_language_option(language: &str) -> Option<LanguageOption> {
    let normalized = if SUPPORTED_OPENAI_REALTIME_LANGUAGES.contains(&language) {
        language
    } else {
        ""
    };

    SUPPORTED_OPENAI_REALTIME_LANGUAGE_OPTIONS
        .iter()
        .copied()
        .find(|option| option.code == normalized)
}

fn selected_profile_option(profile: &str) -> Option<ProfileOption> {
    let normalized = if SUPPORTED_OPENAI_REALTIME_PROFILES.contains(&profile) {
        profile
    } else {
        "balanced"
    };

    SUPPORTED_OPENAI_REALTIME_PROFILE_OPTIONS
        .iter()
        .copied()
        .find(|option| option.code == normalized)
}

fn selected_copilot_mode_option(mode: &str) -> Option<CopilotModeOption> {
    let normalized = CopilotMode::from_code(mode).code();

    SUPPORTED_COPILOT_MODE_OPTIONS
        .iter()
        .copied()
        .find(|option| option.code == normalized)
}

fn card_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(8, 14, 20, 0.74)))
        .border(Border {
            color: Color::from_rgba8(94, 234, 212, 0.12),
            width: 1.0,
            radius: 18.0.into(),
        })
        .color(Color::from_rgb8(248, 250, 252))
}

fn error_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::from_rgba8(120, 18, 24, 0.28)))
        .border(Border {
            color: Color::from_rgba8(248, 113, 113, 0.24),
            width: 1.0,
            radius: 18.0.into(),
        })
        .color(Color::from_rgb8(255, 207, 164))
}
