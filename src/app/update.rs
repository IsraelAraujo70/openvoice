use crate::app::message::Message;
use crate::app::state::{Overlay, OverlayPhase, Scene};
use crate::modules::dictation::application;
use crate::modules::dictation::domain::DictationConfig;
use crate::modules::settings::application as settings_application;
use crate::modules::settings::domain::SettingsForm;
use crate::platform::audio;
use crate::platform::window as app_window;
use iced::keyboard::{self, Key, key::Named};
use iced::{Task, window};

pub fn update(state: &mut Overlay, message: Message) -> Task<Message> {
    match message {
        Message::WindowOpened(id) => {
            if state.main_window_id.is_none() {
                state.main_window_id = Some(id);

                let mut tasks = vec![window::set_level(id, window::Level::AlwaysOnTop)];

                if let Some(primary) = state.primary_monitor {
                    let hud = app_window::hud_settings();

                    tasks.push(window::resize(id, hud.size));
                    tasks.push(window::move_to(
                        id,
                        match hud.position {
                            window::Position::Specific(point) => point,
                            _ => primary.position,
                        },
                    ));
                    tasks.push(window::set_level(id, window::Level::AlwaysOnTop));
                } else {
                    tasks.push(window::monitor_size(id).map(Message::MonitorSizeLoaded));
                }

                if state.passthrough_enabled {
                    tasks.push(window::enable_mouse_passthrough(id));
                }

                return Task::batch(tasks);
            }

            Task::none()
        }
        Message::WindowCloseRequested(_id) => Task::done(Message::Quit),
        Message::MonitorSizeLoaded(Some(_size)) => Task::none(),
        Message::MonitorSizeLoaded(None) => Task::none(),
        Message::KeyEvent(event) => match event {
            keyboard::Event::KeyPressed {
                key, physical_key, ..
            } => match key.as_ref() {
                Key::Named(Named::Escape) if matches!(state.scene, Scene::Settings) => {
                    Task::done(Message::CloseSettingsView)
                }
                Key::Named(Named::Escape) => Task::done(Message::Quit),
                _ if matches!(key.to_latin(physical_key), Some('p')) => {
                    Task::done(Message::TogglePassthrough)
                }
                _ => Task::none(),
            },
            _ => Task::none(),
        },
        Message::OpenSettingsView => {
            if state.is_recording() || state.is_processing() {
                state.error = Some(String::from(
                    "Finalize o ditado antes de abrir a view de settings.",
                ));
                return Task::none();
            }

            state.scene = Scene::Settings;
            state.error = None;

            state.main_window_id.map_or_else(Task::none, |window_id| {
                let settings = app_window::settings_window_settings();
                let position = match settings.position {
                    window::Position::Specific(point) => point,
                    _ => iced::Point::ORIGIN,
                };

                Task::batch([
                    window::disable_mouse_passthrough(window_id),
                    window::resize(window_id, settings.size),
                    window::move_to(window_id, position),
                    window::set_level(window_id, window::Level::Normal),
                ])
            })
        }
        Message::CloseSettingsView => {
            state.scene = Scene::Hud;
            state.error = None;

            state.main_window_id.map_or_else(Task::none, |window_id| {
                let hud = app_window::hud_settings();
                let position = match hud.position {
                    window::Position::Specific(point) => point,
                    _ => iced::Point::ORIGIN,
                };
                let passthrough_task = if state.passthrough_enabled {
                    window::enable_mouse_passthrough(window_id)
                } else {
                    window::disable_mouse_passthrough(window_id)
                };

                Task::batch([
                    passthrough_task,
                    window::resize(window_id, hud.size),
                    window::move_to(window_id, position),
                    window::set_level(window_id, window::Level::AlwaysOnTop),
                ])
            })
        }
        Message::SettingsApiKeyChanged(value) => {
            state.settings_form.openrouter_api_key = value;
            Task::none()
        }
        Message::SettingsModelChanged(value) => {
            state.settings_form.openrouter_model = value;
            Task::none()
        }
        Message::SaveSettings => {
            state.is_saving_settings = true;
            state.settings_note = Some(String::from("Salvando settings..."));
            state.error = None;

            let api_key = state.settings_form.openrouter_api_key.clone();
            let model = state.settings_form.openrouter_model.clone();

            Task::perform(
                async move { settings_application::save_settings(api_key, model) },
                Message::SettingsSaved,
            )
        }
        Message::SettingsSaved(result) => {
            state.is_saving_settings = false;

            match result {
                Ok(settings) => {
                    state.settings = settings;
                    state.settings_form = SettingsForm::from(&state.settings);
                    state.settings_note = Some(String::from("Settings salvas em disco."));
                    state.error = None;

                    if !state.is_recording() && !state.is_processing() {
                        state.phase = OverlayPhase::Idle;
                        state.hint =
                            String::from("Settings prontas. Clique no microfone para ditar.");
                    }

                    Task::none()
                }
                Err(error) => {
                    state.settings_note = None;
                    state.error = Some(error);
                    Task::none()
                }
            }
        }
        Message::StartDictation => {
            if !state.can_start_dictation() {
                state.phase = OverlayPhase::Error;
                state.error = Some(String::from(
                    "Cadastre e salve sua OpenRouter API key antes de gravar.",
                ));
                return Task::none();
            }

            match audio::start_default_recording() {
                Ok(recorder) => {
                    let device_name = recorder.device_name().to_owned();

                    state.recorder = Some(recorder);
                    state.phase = OverlayPhase::Recording;
                    state.hint =
                        format!("Capturando no {device_name}. Clique no microfone para parar.");
                    state.error = None;
                    state.preview = None;

                    if state.passthrough_enabled {
                        state.passthrough_enabled = false;

                        return state.main_window_id.map_or_else(Task::none, |window_id| {
                            Task::batch([
                                window::disable_mouse_passthrough(window_id),
                                window::set_level(window_id, window::Level::AlwaysOnTop),
                            ])
                        });
                    }

                    Task::none()
                }
                Err(error) => {
                    state.phase = OverlayPhase::Error;
                    state.hint = String::from("Nao consegui abrir o microfone padrao.");
                    state.error = Some(error);
                    Task::none()
                }
            }
        }
        Message::StopDictation => {
            let Some(recorder) = state.recorder.take() else {
                return Task::none();
            };

            match recorder.finish() {
                Ok(capture) => {
                    let Ok(config) = DictationConfig::from_settings(&state.settings) else {
                        state.phase = OverlayPhase::Error;
                        state.hint = String::from("OpenRouter nao configurado.");
                        state.error = Some(String::from(
                            "Cadastre e salve a OpenRouter API key antes de gravar.",
                        ));
                        return Task::none();
                    };

                    state.phase = OverlayPhase::Processing;
                    state.hint = String::from(
                        "Enviando audio para o OpenRouter e aguardando transcricao...",
                    );
                    state.error = None;

                    Task::perform(
                        async move { application::transcribe_capture(config, capture) },
                        Message::DictationFinished,
                    )
                }
                Err(error) => {
                    state.phase = OverlayPhase::Error;
                    state.hint = String::from("A captura foi interrompida antes do envio.");
                    state.error = Some(error);
                    Task::none()
                }
            }
        }
        Message::DictationFinished(result) => match result {
            Ok(output) => {
                state.phase = OverlayPhase::Success;
                state.hint = format!(
                    "{:.1}s de audio transcritos e enviados para o clipboard.",
                    output.duration_seconds
                );
                state.error = None;
                state.preview = Some(output.preview());

                Task::batch([
                    iced::clipboard::write(output.transcript.clone()),
                    iced::clipboard::write_primary(output.transcript),
                ])
            }
            Err(error) => {
                state.phase = OverlayPhase::Error;
                state.hint = String::from("A transcricao via OpenRouter falhou.");
                state.error = Some(error);
                Task::none()
            }
        },
        Message::TogglePassthrough => {
            if !state.passthrough_enabled && (state.is_recording() || state.is_processing()) {
                state.error = Some(String::from(
                    "Finalize o ditado antes de habilitar passthrough.",
                ));
                return Task::none();
            }

            state.passthrough_enabled = !state.passthrough_enabled;
            state.hint = if state.passthrough_enabled {
                String::from("Passthrough ativo. Pressione P para voltar ao modo interativo.")
            } else {
                String::from("Modo interativo ativo. O HUD pode receber cliques novamente.")
            };

            state.main_window_id.map_or_else(Task::none, |window_id| {
                let passthrough_task = if state.passthrough_enabled {
                    window::enable_mouse_passthrough(window_id)
                } else {
                    window::disable_mouse_passthrough(window_id)
                };

                Task::batch([
                    passthrough_task,
                    window::set_level(window_id, window::Level::AlwaysOnTop),
                ])
            })
        }
        Message::Quit => {
            let mut tasks = Vec::new();

            if let Some(window_id) = state.main_window_id.take() {
                tasks.push(window::close(window_id));
            }

            Task::batch(tasks)
        }
    }
}
