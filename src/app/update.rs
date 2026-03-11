use crate::app::message::Message;
use crate::app::state::{Overlay, OverlayPhase};
use crate::modules::audio::infrastructure::microphone;
use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::CredentialStoreStrategy;
use crate::modules::dictation::application as dictation_application;
use crate::modules::dictation::domain::DictationConfig;
use crate::modules::live_transcription::application as live_transcription_application;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::live_transcription::infrastructure::db;
use crate::modules::settings::application as settings_application;
use crate::modules::settings::domain::SettingsForm;
use crate::platform::window as app_window;
use iced::keyboard::{self, Key, key::Named};
use iced::{Point, Task, window};

pub fn update(state: &mut Overlay, message: Message) -> Task<Message> {
    match message {
        // ------------------------------------------------------------------ //
        // Window lifecycle
        // ------------------------------------------------------------------ //
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

        Message::WindowCloseRequested(id) => {
            // Only quit if the main HUD window requests close.
            // Secondary windows (subtitle, sessions) just close themselves.
            if state.main_window_id == Some(id) {
                Task::done(Message::Quit)
            } else if state.sessions_window_id == Some(id) {
                Task::done(Message::CloseSessionsView)
            } else {
                // subtitle has no decorations so this shouldn't fire,
                // but handle gracefully anyway.
                Task::none()
            }
        }

        Message::MonitorSizeLoaded(Some(_size)) => Task::none(),
        Message::MonitorSizeLoaded(None) => Task::none(),

        Message::StartDrag => state.main_window_id.map_or_else(Task::none, window::drag),

        Message::WindowMoved(position) => {
            if state.settings_open {
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

        // ------------------------------------------------------------------ //
        // Input events
        // ------------------------------------------------------------------ //
        Message::KeyEvent(event) => match event {
            keyboard::Event::KeyPressed {
                key, physical_key, ..
            } => match key.as_ref() {
                Key::Named(Named::Escape) if state.settings_open => {
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

        // ------------------------------------------------------------------ //
        // Settings navigation
        // ------------------------------------------------------------------ //
        Message::OpenSettingsView => {
            if state.is_recording() || state.is_processing() {
                state.error = Some(String::from(
                    "Finalize o ditado antes de abrir a view de settings.",
                ));
                return Task::none();
            }

            state.settings_open = true;
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
            state.settings_open = false;
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

        // ------------------------------------------------------------------ //
        // Settings form
        // ------------------------------------------------------------------ //
        Message::SettingsApiKeyChanged(value) => {
            state.settings_form.openrouter_api_key = value;
            Task::none()
        }
        Message::SettingsOpenAiRealtimeApiKeyChanged(value) => {
            state.settings_form.openai_realtime_api_key = value;
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
        Message::SettingsOpenAiRealtimeLanguageChanged(value) => {
            state.settings_form.openai_realtime_language = value;
            Task::none()
        }
        Message::SaveSettings => {
            state.is_saving_settings = true;
            state.settings_note = Some(String::from("Salvando settings..."));
            state.error = None;

            let openrouter_api_key = state.settings_form.openrouter_api_key.clone();
            let openai_realtime_api_key = state.settings_form.openai_realtime_api_key.clone();
            let openrouter_model = state.settings_form.openrouter_model.clone();
            let openai_realtime_model = state.settings_form.openai_realtime_model.clone();
            let openai_realtime_language = state.settings_form.openai_realtime_language.clone();

            Task::perform(
                async move {
                    settings_application::save_settings(
                        openrouter_api_key,
                        openai_realtime_api_key,
                        openrouter_model,
                        openai_realtime_model,
                        openai_realtime_language,
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

        // ------------------------------------------------------------------ //
        // OpenAI OAuth
        // ------------------------------------------------------------------ //
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
                        "Login ChatGPT concluido para modelos OAuth futuros.",
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

        // ------------------------------------------------------------------ //
        // Dictation (mic → OpenRouter)
        // ------------------------------------------------------------------ //
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

        // ------------------------------------------------------------------ //
        // Realtime transcription (system audio → OpenAI Realtime API)
        // ------------------------------------------------------------------ //
        Message::StartRealtimeTranscription => {
            if !state.can_start_realtime_transcription() {
                state.phase = OverlayPhase::Error;
                state.error = Some(if !state.settings.has_openai_realtime_api_key() {
                    String::from(
                        "Cadastre e salve uma OpenAI API key nas settings antes de iniciar a transcription realtime.",
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
                    state.subtitle_closing = false;
                    state.live_session_started_at = Some(db::now_iso());

                    // Open the subtitle window
                    let subtitle_settings =
                        app_window::subtitle_window_settings(state.primary_monitor);
                    let (_, open_subtitle) = window::open(subtitle_settings);

                    Task::batch([
                        open_subtitle.map(Message::SubtitleWindowOpened),
                        Task::perform(
                            async move { live_transcription_application::poll_next_event(receiver) },
                            Message::RealtimeEventReceived,
                        ),
                    ])
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
            // Do NOT clear live_completed_segments yet — subtitle stays visible for 3s.
            state.subtitle_closing = true;

            // Save session to DB
            let segments = state.live_completed_segments.clone();
            let started_at = state
                .live_session_started_at
                .clone()
                .unwrap_or_else(db::now_iso);
            let stopped_at = db::now_iso();
            let language = Some(state.settings.openai_realtime_language.clone());
            let model = Some(state.settings.openai_realtime_model.clone());

            let save_task = Task::perform(
                async move { db::save_session(segments, started_at, stopped_at, language, model) },
                Message::LiveTranscriptionSaved,
            );

            // Close subtitle after 3 seconds
            let close_task = Task::perform(
                async {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                },
                |_| Message::CloseSubtitleWindow,
            );

            Task::batch([save_task, close_task])
        }

        // ------------------------------------------------------------------ //
        // Subtitle window
        // ------------------------------------------------------------------ //
        Message::SubtitleWindowOpened(id) => {
            state.subtitle_window_id = Some(id);
            Task::batch([
                window::set_level(id, window::Level::AlwaysOnTop),
                window::enable_mouse_passthrough(id),
            ])
        }

        Message::CloseSubtitleWindow => {
            state.subtitle_closing = false;
            state.live_completed_segments.clear();
            state.live_partial_transcript.clear();

            if let Some(id) = state.subtitle_window_id.take() {
                window::close(id)
            } else {
                Task::none()
            }
        }

        // ------------------------------------------------------------------ //
        // Realtime events
        // ------------------------------------------------------------------ //
        Message::RealtimeEventReceived(event) => {
            let Some(event) = event else {
                state.live_transcription = None;
                state.phase = OverlayPhase::Idle;
                state.hint = String::from("Realtime transcription encerrada.");
                state.live_partial_item_id = None;
                state.live_partial_transcript.clear();
                // Keep segments for subtitle fade-out
                state.subtitle_closing = true;

                let close_task = Task::perform(
                    async {
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    },
                    |_| Message::CloseSubtitleWindow,
                );
                return close_task;
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
                    }
                }
                RuntimeEvent::TranscriptCompleted {
                    item_id,
                    transcript,
                } => {
                    if !transcript.trim().is_empty() {
                        state.live_completed_segments.push(transcript.clone());
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
                    state.subtitle_closing = true;
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
                    // Keep segments for subtitle fade-out
                    state.subtitle_closing = true;
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

        // ------------------------------------------------------------------ //
        // Live transcription persistence
        // ------------------------------------------------------------------ //
        Message::LiveTranscriptionSaved(result) => {
            match result {
                Ok(_session_id) => {
                    state.hint = String::from("Sessao salva.");
                }
                Err(err) => {
                    state.error = Some(format!("Erro ao salvar sessao: {err}"));
                }
            }
            Task::none()
        }

        // ------------------------------------------------------------------ //
        // Sessions window
        // ------------------------------------------------------------------ //
        Message::OpenSessionsView => {
            if state.sessions_window_id.is_some() {
                // Already open — focus it
                return Task::none();
            }

            state.sessions_loading = true;
            state.sessions_error = None;

            let sessions_settings = app_window::sessions_window_settings(state.primary_monitor);
            let (_, open_sessions) = window::open(sessions_settings);

            Task::batch([
                open_sessions.map(Message::SessionsWindowOpened),
                Task::perform(async { db::list_sessions() }, Message::SessionsLoaded),
            ])
        }

        Message::SessionsWindowOpened(id) => {
            state.sessions_window_id = Some(id);
            window::set_level(id, window::Level::Normal)
        }

        Message::CloseSessionsView => {
            state.sessions_list.clear();
            state.sessions_error = None;
            state.sessions_loading = false;
            state.selected_session_id = None;
            state.selected_session_segments.clear();
            state.selected_session_loading = false;

            if let Some(id) = state.sessions_window_id.take() {
                window::close(id)
            } else {
                Task::none()
            }
        }

        Message::SessionsLoaded(result) => {
            state.sessions_loading = false;
            match result {
                Ok(sessions) => {
                    state.sessions_list = sessions;
                    state.sessions_error = None;
                }
                Err(err) => {
                    state.sessions_error = Some(err);
                }
            }
            Task::none()
        }

        Message::SessionSelected(id) => {
            // id == 0 is a sentinel for "deselect"
            if id == 0 || state.selected_session_id == Some(id) {
                state.selected_session_id = None;
                state.selected_session_segments.clear();
                return Task::none();
            }

            state.selected_session_id = Some(id);
            state.selected_session_loading = true;
            state.selected_session_segments.clear();

            Task::perform(
                async move { db::get_session_segments(id) },
                Message::SessionDetailLoaded,
            )
        }

        Message::SessionDetailLoaded(result) => {
            state.selected_session_loading = false;
            match result {
                Ok(segments) => {
                    state.selected_session_segments = segments;
                }
                Err(err) => {
                    state.error = Some(format!("Erro ao carregar segmentos: {err}"));
                }
            }
            Task::none()
        }

        Message::CopySessionTranscript => {
            let transcript = state.selected_session_segments.join(" ");
            if transcript.is_empty() {
                return Task::none();
            }
            Task::batch([
                iced::clipboard::write(transcript.clone()),
                iced::clipboard::write_primary(transcript),
            ])
        }

        // ------------------------------------------------------------------ //
        // Window behavior
        // ------------------------------------------------------------------ //
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
            if let Some(session) = state.live_transcription.take() {
                session.stop();
            }

            iced::exit()
        }
    }
}
