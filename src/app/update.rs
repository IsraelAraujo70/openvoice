use crate::app::message::Message;
use crate::app::state::{Overlay, OverlayPhase, Scene};
use crate::modules::audio::infrastructure::microphone;
use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::CredentialStoreStrategy;
use crate::modules::dictation::application as dictation_application;
use crate::modules::dictation::domain::DictationConfig;
use crate::modules::live_transcription::application as live_transcription_application;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::settings::application as settings_application;
use crate::modules::settings::domain::SettingsForm;
use crate::platform::window as app_window;
use iced::keyboard::{self, Key, key::Named};
use iced::{Point, Task, window};

pub fn update(state: &mut Overlay, message: Message) -> Task<Message> {
    match message {
        Message::WindowOpened(id) => {
            if state.main_window_id.is_none() {
                state.main_window_id = Some(id);

                let mut tasks = vec![window::set_level(id, window::Level::AlwaysOnTop)];

                if let Some(primary) = state.primary_monitor {
                    let hud = app_window::hud_settings();
                    let position = match hud.position {
                        window::Position::Specific(point) => point,
                        _ => primary.position,
                    };

                    state.hud_position = Some(position);

                    tasks.push(window::resize(id, hud.size));
                    tasks.push(window::move_to(id, position));
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
        Message::StartDrag => state.main_window_id.map_or_else(Task::none, window::drag),
        Message::WindowMoved(position) => {
            if !matches!(state.scene, Scene::Hud) {
                return Task::none();
            }

            if let Some(monitor) = state.primary_monitor {
                let clamped = app_window::clamp_hud_to_monitor(position, monitor);
                state.hud_position = Some(clamped);

                if clamped != position {
                    return state
                        .main_window_id
                        .map_or_else(Task::none, |id| window::move_to(id, clamped));
                }
            } else {
                state.hud_position = Some(position);
            }

            Task::none()
        }
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
                let default_position = match hud.position {
                    window::Position::Specific(point) => point,
                    _ => Point::ORIGIN,
                };
                let position = state.hud_position.unwrap_or(default_position);
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
        Message::SettingsOpenAiRealtimeModelChanged(value) => {
            state.settings_form.openai_realtime_model = value;
            Task::none()
        }
        Message::SaveSettings => {
            state.is_saving_settings = true;
            state.settings_note = Some(String::from("Salvando settings..."));
            state.error = None;

            let openrouter_api_key = state.settings_form.openrouter_api_key.clone();
            let openrouter_model = state.settings_form.openrouter_model.clone();
            let openai_realtime_model = state.settings_form.openai_realtime_model.clone();

            Task::perform(
                async move {
                    settings_application::save_settings(
                        openrouter_api_key,
                        openrouter_model,
                        openai_realtime_model,
                    )
                },
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
                        state.hint = String::from(
                            "Settings prontas. Clique no microfone ou use RT para transcricao ao vivo.",
                        );
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
        Message::StartOpenAiOAuthLogin => {
            state.is_openai_authenticating = true;
            state.settings_note = Some(String::from(
                "Abrindo navegador para login ChatGPT em localhost:1455...",
            ));
            state.error = None;
            state.pending_openai_oauth = None;
            state.openai_callback_url_input.clear();

            Task::perform(
                async move { auth_application::start_login(CredentialStoreStrategy::Auto) },
                Message::OpenAiOAuthStarted,
            )
        }
        Message::OpenAiOAuthStarted(result) => match result {
            Ok(flow) => {
                state.pending_openai_oauth = Some(flow.clone());
                state.settings_note = Some(String::from(
                    "Navegador aberto. Se o app nao concluir sozinho, cole a callback URL abaixo.",
                ));
                state.error = None;

                Task::perform(
                    async move { auth_application::wait_for_callback(flow.flow_id) },
                    Message::OpenAiOAuthCallbackCaptured,
                )
            }
            Err(error) => {
                state.is_openai_authenticating = false;
                state.settings_note = None;
                state.error = Some(error);
                Task::none()
            }
        },
        Message::OpenAiOAuthCallbackCaptured(result) => match result {
            Ok(callback_url) => {
                state.openai_callback_url_input = callback_url.clone();

                if let Some(flow) = state.pending_openai_oauth.as_ref() {
                    let flow_id = flow.flow_id.clone();
                    return Task::perform(
                        async move { auth_application::complete_login(flow_id, callback_url) },
                        Message::OpenAiOAuthFinished,
                    );
                }

                Task::none()
            }
            Err(error) => {
                state.is_openai_authenticating = false;
                state.settings_note = Some(String::from(
                    "Callback automatico nao concluiu. Cole a callback URL manualmente.",
                ));
                state.error = Some(error);
                Task::none()
            }
        },
        Message::OpenAiOAuthCallbackUrlChanged(value) => {
            state.openai_callback_url_input = value;
            Task::none()
        }
        Message::SubmitOpenAiOAuthCallback => {
            let Some(flow) = state.pending_openai_oauth.as_ref() else {
                state.error = Some(String::from(
                    "Nao existe login OAuth pendente para concluir manualmente.",
                ));
                return Task::none();
            };

            let callback_url = state.openai_callback_url_input.trim().to_owned();
            if callback_url.is_empty() {
                state.error = Some(String::from("Cole a callback URL antes de continuar."));
                return Task::none();
            }

            state.is_openai_authenticating = true;
            state.settings_note = Some(String::from(
                "Finalizando login ChatGPT a partir da callback URL...",
            ));
            state.error = None;

            let flow_id = flow.flow_id.clone();
            Task::perform(
                async move { auth_application::complete_login(flow_id, callback_url) },
                Message::OpenAiOAuthFinished,
            )
        }
        Message::OpenAiOAuthFinished(result) => {
            state.is_openai_authenticating = false;

            match result {
                Ok(snapshot) => {
                    state.pending_openai_oauth = None;
                    state.openai_callback_url_input.clear();
                    state.has_openai_credentials = snapshot.is_authenticated;
                    state.openai_account_label = snapshot.account_label;
                    state.settings_note = Some(String::from(
                        "Login ChatGPT concluido. Realtime transcription pronta para uso.",
                    ));
                    state.error = None;
                    Task::none()
                }
                Err(error) => {
                    state.settings_note = None;
                    state.error = Some(error);
                    Task::none()
                }
            }
        }
        Message::LogoutOpenAi => {
            state.is_openai_authenticating = true;
            state.pending_openai_oauth = None;
            state.openai_callback_url_input.clear();
            state.settings_note = Some(String::from("Removendo sessao ChatGPT do OpenVoice..."));
            state.error = None;

            Task::perform(
                async move { auth_application::logout(CredentialStoreStrategy::Auto) },
                Message::OpenAiLogoutFinished,
            )
        }
        Message::OpenAiLogoutFinished(result) => {
            state.is_openai_authenticating = false;

            match result {
                Ok(()) => {
                    state.has_openai_credentials = false;
                    state.openai_account_label = None;
                    state.settings_note =
                        Some(String::from("Sessao ChatGPT removida do OpenVoice."));
                    state.error = None;
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

            match microphone::start_default_recording() {
                Ok(recorder) => {
                    let device_name = recorder
                        .device_name()
                        .unwrap_or("microfone padrao")
                        .to_owned();

                    state.recorder = Some(recorder);
                    state.phase = OverlayPhase::Recording;
                    state.hint =
                        format!("REC MIC ativo em {device_name}. Clique no microfone para parar.");
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
                    state.hint = String::from("Nao consegui iniciar a captura do microfone.");
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
                Ok(capture_track) => {
                    let Ok(config) = DictationConfig::from_settings(&state.settings) else {
                        state.phase = OverlayPhase::Error;
                        state.hint = String::from("OpenRouter nao configurado.");
                        state.error = Some(String::from(
                            "Cadastre e salve a OpenRouter API key antes de gravar.",
                        ));
                        return Task::none();
                    };

                    state.phase = OverlayPhase::Processing;
                    state.hint = String::from("Enviando audio do microfone para o OpenRouter...");
                    state.error = None;

                    Task::perform(
                        async move {
                            dictation_application::transcribe_capture(config, capture_track.audio)
                        },
                        Message::DictationFinished,
                    )
                }
                Err(error) => {
                    state.phase = OverlayPhase::Error;
                    state.hint =
                        String::from("A captura do microfone foi interrompida antes do envio.");
                    state.error = Some(error);
                    Task::none()
                }
            }
        }
        Message::DictationFinished(result) => match result {
            Ok(output) => {
                state.phase = OverlayPhase::Success;
                state.hint = format!(
                    "{:.1}s de audio do microfone transcritos e enviados para o clipboard.",
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
        Message::StartRealtimeTranscription => {
            if !state.can_start_realtime_transcription() {
                state.phase = OverlayPhase::Error;
                state.error = Some(if !state.has_openai_credentials {
                    String::from(
                        "Entre com ChatGPT nas settings antes de iniciar a transcription realtime.",
                    )
                } else {
                    String::from("Finalize a acao atual antes de iniciar a transcription realtime.")
                });
                return Task::none();
            }

            match live_transcription_application::start_live_transcription(&state.settings) {
                Ok(session) => {
                    let receiver = session.receiver();
                    state.live_transcription = Some(session);
                    state.phase = OverlayPhase::Recording;
                    state.hint = String::from(
                        "Realtime transcription conectada ao system audio. Clique novamente para parar.",
                    );
                    state.error = None;
                    state.preview = None;
                    state.live_partial_item_id = None;
                    state.live_partial_transcript.clear();
                    state.live_completed_segments.clear();

                    Task::perform(
                        async move { live_transcription_application::poll_next_event(receiver) },
                        Message::RealtimeEventReceived,
                    )
                }
                Err(error) => {
                    state.phase = OverlayPhase::Error;
                    state.hint = String::from("Nao consegui iniciar a transcription realtime.");
                    state.error = Some(error);
                    Task::none()
                }
            }
        }
        Message::StopRealtimeTranscription => {
            if let Some(session) = state.live_transcription.take() {
                session.stop();
            }

            state.phase = OverlayPhase::Idle;
            state.hint = String::from("Realtime transcription finalizada.");
            state.error = None;
            state.live_partial_item_id = None;
            state.live_partial_transcript.clear();
            state.live_completed_segments.clear();
            Task::none()
        }
        Message::RealtimeEventReceived(event) => {
            let Some(event) = event else {
                state.live_transcription = None;
                state.phase = OverlayPhase::Idle;
                state.hint = String::from("Realtime transcription encerrada.");
                state.live_partial_item_id = None;
                state.live_partial_transcript.clear();
                state.live_completed_segments.clear();
                return Task::none();
            };

            let mut continue_polling = state.live_transcription.is_some();

            match event {
                RuntimeEvent::Connected => {
                    state.phase = OverlayPhase::Recording;
                    state.hint =
                        String::from("System audio em streaming para transcricao realtime.");
                    state.error = None;
                }
                RuntimeEvent::TranscriptDelta { item_id, delta } => {
                    if !delta.trim().is_empty() {
                        if state.live_partial_item_id.as_deref() != Some(item_id.as_str()) {
                            state.live_partial_item_id = Some(item_id);
                            state.live_partial_transcript.clear();
                        }

                        state.live_partial_transcript.push_str(&delta);
                        state.preview = Some(state.live_partial_transcript.clone());
                    }
                }
                RuntimeEvent::TranscriptCompleted {
                    item_id,
                    transcript,
                } => {
                    if !transcript.trim().is_empty() {
                        state.live_completed_segments.push(transcript.clone());
                        state.preview = Some(transcript);
                    }
                    state.live_partial_item_id = Some(item_id);
                    state.live_partial_transcript.clear();
                    state.error = None;
                }
                RuntimeEvent::Warning(warning) => {
                    state.hint = warning;
                }
                RuntimeEvent::Error(error) => {
                    state.error = Some(error);
                    state.phase = OverlayPhase::Error;
                    state.live_partial_item_id = None;
                    state.live_partial_transcript.clear();
                    state.live_completed_segments.clear();
                    continue_polling = false;

                    if let Some(session) = state.live_transcription.take() {
                        session.stop();
                    }
                }
                RuntimeEvent::Stopped => {
                    state.live_transcription = None;
                    state.phase = OverlayPhase::Idle;
                    state.hint = String::from("Realtime transcription parada.");
                    state.error = None;
                    state.live_partial_item_id = None;
                    state.live_partial_transcript.clear();
                    state.live_completed_segments.clear();
                    continue_polling = false;
                }
            }

            if continue_polling {
                if let Some(session) = state.live_transcription.as_ref() {
                    let receiver = session.receiver();
                    return Task::perform(
                        async move { live_transcription_application::poll_next_event(receiver) },
                        Message::RealtimeEventReceived,
                    );
                }
            }

            Task::none()
        }
        Message::TogglePassthrough => {
            if !state.passthrough_enabled
                && (state.is_recording() || state.is_processing() || state.is_live_transcribing())
            {
                state.error = Some(String::from(
                    "Finalize a captura atual antes de habilitar passthrough.",
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

            if let Some(session) = state.live_transcription.take() {
                session.stop();
            }

            if let Some(window_id) = state.main_window_id.take() {
                tasks.push(window::close(window_id));
            }

            Task::batch(tasks)
        }
    }
}
