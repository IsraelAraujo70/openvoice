use crate::app::message::Message;
use crate::app::state::{HomeTab, MainView, Overlay, OverlayPhase};
use crate::modules::audio::infrastructure::microphone;
use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::CredentialStoreStrategy;
use crate::modules::copilot::application as copilot_application;
use crate::modules::copilot::domain::{
    CopilotAnswer, CopilotChatMessage, CopilotContext, CopilotMode, CopilotRole,
};
use crate::modules::dictation::application as dictation_application;
use crate::modules::dictation::domain::DictationConfig;
use crate::modules::live_transcription::application as live_transcription_application;
use crate::modules::live_transcription::domain::RuntimeEvent;
use crate::modules::live_transcription::infrastructure::db;
use crate::modules::settings::application as settings_application;
use crate::modules::settings::domain::SettingsForm;
use crate::platform::screenshot as screenshot_platform;
use crate::platform::window as app_window;
use iced::widget::text_editor;
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
            // Secondary windows (subtitle) just close themselves.
            if state.main_window_id == Some(id) {
                Task::done(Message::Quit)
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
            if state.main_view == MainView::Home {
                return Task::none();
            }

            state.hud_position = Some(position);
            Task::none()
        }

        // ------------------------------------------------------------------ //
        // Input events
        // ------------------------------------------------------------------ //
        Message::KeyEvent(event) => match event {
            keyboard::Event::KeyPressed {
                key, physical_key, ..
            } => match key.as_ref() {
                Key::Named(Named::Escape) if state.main_view == MainView::Copilot => {
                    Task::done(Message::CloseCopilotView)
                }
                Key::Named(Named::Escape) if state.main_view == MainView::Home => {
                    Task::done(Message::CloseHomeView)
                }
                Key::Named(Named::Escape) => Task::done(Message::Quit),
                _ if matches!(key.to_latin(physical_key), Some('p'))
                    && state.main_view == MainView::Hud =>
                {
                    Task::done(Message::TogglePassthrough)
                }
                _ => Task::none(),
            },
            _ => Task::none(),
        },

        // ------------------------------------------------------------------ //
        // Home navigation
        // ------------------------------------------------------------------ //
        Message::OpenHomeView => {
            if state.is_recording() || state.is_processing() {
                state.error = Some(String::from("Finalize o ditado antes de abrir a Home."));
                return Task::none();
            }

            state.main_view = MainView::Home;
            state.home_tab = HomeTab::Home;
            state.error = None;

            // Pre-load sessions for the recent sessions summary on the Home tab
            state.sessions_loading = true;

            state.main_window_id.map_or_else(
                || Task::perform(async { db::list_sessions() }, Message::SessionsLoaded),
                |window_id| {
                    let settings = app_window::home_window_settings();
                    let position = match settings.position {
                        window::Position::Specific(point) => point,
                        _ => iced::Point::ORIGIN,
                    };

                    Task::batch([
                        window::disable_mouse_passthrough(window_id),
                        window::resize(window_id, settings.size),
                        window::move_to(window_id, position),
                        window::set_level(window_id, window::Level::Normal),
                        Task::perform(async { db::list_sessions() }, Message::SessionsLoaded),
                    ])
                },
            )
        }

        Message::OpenCopilotView => open_copilot_view(state),

        Message::CloseCopilotView => close_copilot_view(state),

        Message::CloseHomeView => {
            state.main_view = MainView::Hud;
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

        Message::SwitchHomeTab(tab) => {
            let needs_open = state.main_view == MainView::Hud;
            let reload_sessions = matches!(tab, HomeTab::Sessions);

            if needs_open {
                if state.is_recording() || state.is_processing() {
                    state.error = Some(String::from("Finalize o ditado antes de abrir a Home."));
                    return Task::none();
                }

                state.main_view = MainView::Home;
                state.home_tab = tab;
                state.error = None;

                let mut tasks = Vec::new();

                if let Some(window_id) = state.main_window_id {
                    let settings = app_window::home_window_settings();
                    let position = match settings.position {
                        window::Position::Specific(point) => point,
                        _ => iced::Point::ORIGIN,
                    };

                    tasks.push(window::disable_mouse_passthrough(window_id));
                    tasks.push(window::resize(window_id, settings.size));
                    tasks.push(window::move_to(window_id, position));
                    tasks.push(window::set_level(window_id, window::Level::Normal));
                }

                if reload_sessions {
                    state.sessions_loading = true;
                    tasks.push(Task::perform(
                        async { db::list_sessions() },
                        Message::SessionsLoaded,
                    ));
                }

                Task::batch(tasks)
            } else {
                state.home_tab = tab;

                if reload_sessions {
                    state.sessions_loading = true;
                    Task::perform(async { db::list_sessions() }, Message::SessionsLoaded)
                } else {
                    Task::none()
                }
            }
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
        Message::SettingsOpenAiRealtimeProfileChanged(value) => {
            state.settings_form.openai_realtime_profile = value;
            Task::none()
        }
        Message::SettingsCopilotModelChanged(value) => {
            state.settings_form.copilot_model = value;
            Task::none()
        }
        Message::SettingsCopilotDefaultModeChanged(value) => {
            state.settings_form.copilot_default_mode = value;
            Task::none()
        }
        Message::SettingsCopilotAutoIncludeTranscriptChanged(value) => {
            state.settings_form.copilot_auto_include_transcript = value;
            Task::none()
        }
        Message::SettingsCopilotSaveHistoryChanged(value) => {
            state.settings_form.copilot_save_history = value;
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
            let openai_realtime_profile = state.settings_form.openai_realtime_profile.clone();
            let copilot_model = state.settings_form.copilot_model.clone();
            let copilot_default_mode = state.settings_form.copilot_default_mode.clone();
            let copilot_auto_include_transcript =
                state.settings_form.copilot_auto_include_transcript;
            let copilot_save_history = state.settings_form.copilot_save_history;

            Task::perform(
                async move {
                    settings_application::save_settings(
                        openrouter_api_key,
                        openai_realtime_api_key,
                        openrouter_model,
                        openai_realtime_model,
                        openai_realtime_language,
                        openai_realtime_profile,
                        copilot_model,
                        copilot_default_mode,
                        copilot_auto_include_transcript,
                        copilot_save_history,
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
                    state.copilot_mode = state.settings.copilot_default_mode();
                    state.copilot_include_transcript =
                        state.settings.copilot_auto_include_transcript;
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

            // Auto-close Home → HUD before starting dictation
            let mut morph_tasks = if state.main_view == MainView::Home {
                morph_home_to_hud(state)
            } else {
                Vec::new()
            };

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

                        if let Some(window_id) = state.main_window_id {
                            morph_tasks.push(window::disable_mouse_passthrough(window_id));
                            morph_tasks
                                .push(window::set_level(window_id, window::Level::AlwaysOnTop));
                        }
                    }

                    if morph_tasks.is_empty() {
                        Task::none()
                    } else {
                        Task::batch(morph_tasks)
                    }
                }
                Err(error) => {
                    state.phase = OverlayPhase::Error;
                    state.hint = String::from("Nao consegui iniciar a captura do microfone.");
                    state.error = Some(error);
                    if morph_tasks.is_empty() {
                        Task::none()
                    } else {
                        Task::batch(morph_tasks)
                    }
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

            // Auto-close Home → HUD before starting realtime
            let mut tasks: Vec<Task<Message>> = if state.main_view == MainView::Home {
                morph_home_to_hud(state)
            } else {
                Vec::new()
            };

            match live_transcription_application::start_live_transcription(&state.settings) {
                Ok(session) => {
                    let receiver = session.receiver();
                    let started_at = db::now_iso();
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
                    state.live_session_started_at = Some(started_at.clone());
                    state.live_session_db_id = None;
                    state.live_session_creating = true;
                    state.live_session_finalizing = false;
                    state.live_session_stopped_at = None;
                    state.live_segments_persisting = false;
                    state.live_persisted_segment_count = 0;

                    // Open the subtitle window
                    let subtitle_settings =
                        app_window::subtitle_window_settings(state.primary_monitor);
                    let (_, open_subtitle) = window::open(subtitle_settings);
                    let language = Some(state.settings.openai_realtime_language.clone())
                        .filter(|value| !value.trim().is_empty());
                    let model = Some(state.settings.openai_realtime_model.clone())
                        .filter(|value| !value.trim().is_empty());

                    tasks.push(open_subtitle.map(Message::SubtitleWindowOpened));
                    tasks.push(Task::perform(
                        async move { db::create_live_session(started_at, language, model) },
                        Message::LiveSessionCreated,
                    ));
                    tasks.push(Task::perform(
                        async move { live_transcription_application::poll_next_event(receiver) },
                        Message::RealtimeEventReceived,
                    ));

                    Task::batch(tasks)
                }
                Err(error) => {
                    state.phase = OverlayPhase::Error;
                    state.hint = String::from("Nao consegui iniciar a transcription realtime.");
                    state.error = Some(error);
                    if tasks.is_empty() {
                        Task::none()
                    } else {
                        Task::batch(tasks)
                    }
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
            state.live_session_stopped_at = Some(db::now_iso());

            // Close subtitle after 3 seconds
            let close_task = Task::perform(
                async {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                },
                |_| Message::CloseSubtitleWindow,
            );

            let persist_task = queue_live_persistence(state);
            Task::batch([persist_task, close_task])
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
            state.live_partial_transcript.clear();

            if state.live_transcription.is_none()
                && !state.live_session_creating
                && !state.live_segments_persisting
                && !state.live_session_finalizing
            {
                state.live_completed_segments.clear();
            }

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
                if state.live_session_stopped_at.is_none() {
                    state.live_session_stopped_at = Some(db::now_iso());
                }

                let close_task = Task::perform(
                    async {
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    },
                    |_| Message::CloseSubtitleWindow,
                );
                let persist_task = queue_live_persistence(state);
                return Task::batch([persist_task, close_task]);
            };

            let mut continue_polling = state.live_transcription.is_some();
            let mut tasks = Vec::new();

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

                        push_live_delta(&mut state.live_partial_transcript, &delta);
                    }
                }
                RuntimeEvent::TranscriptCompleted {
                    item_id,
                    transcript,
                } => {
                    let final_transcript = resolve_completed_transcript(
                        &item_id,
                        &transcript,
                        state.live_partial_item_id.as_deref(),
                        &state.live_partial_transcript,
                    );

                    if !final_transcript.is_empty() {
                        state.live_completed_segments.push(final_transcript);
                        tasks.push(queue_pending_live_segments(state));
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
                    if state.live_session_stopped_at.is_none() {
                        state.live_session_stopped_at = Some(db::now_iso());
                    }

                    if let Some(session) = state.live_transcription.take() {
                        session.stop();
                    }
                    tasks.push(queue_live_persistence(state));
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
                    if state.live_session_stopped_at.is_none() {
                        state.live_session_stopped_at = Some(db::now_iso());
                    }
                    tasks.push(queue_live_persistence(state));
                }
            }

            if continue_polling {
                if let Some(session) = state.live_transcription.as_ref() {
                    let receiver = session.receiver();
                    tasks.push(Task::perform(
                        async move { live_transcription_application::poll_next_event(receiver) },
                        Message::RealtimeEventReceived,
                    ));
                }
            }

            if tasks.is_empty() {
                Task::none()
            } else {
                Task::batch(tasks)
            }
        }

        // ------------------------------------------------------------------ //
        // Live transcription persistence
        // ------------------------------------------------------------------ //
        Message::LiveSessionCreated(result) => match result {
            Ok(session_id) => {
                state.live_session_creating = false;
                state.live_session_db_id = Some(session_id);
                queue_live_persistence(state)
            }
            Err(err) => {
                state.live_session_creating = false;
                state.live_session_db_id = None;
                state.error = Some(format!("Erro ao iniciar sessao realtime local: {err}"));
                Task::none()
            }
        },
        Message::LiveSessionSegmentsPersisted(result) => {
            state.live_segments_persisting = false;

            match result {
                Ok(persisted_count) => {
                    state.live_persisted_segment_count =
                        state.live_persisted_segment_count.max(persisted_count);
                    queue_live_persistence(state)
                }
                Err(err) => {
                    state.error = Some(format!("Erro ao persistir segmento realtime: {err}"));
                    Task::none()
                }
            }
        }
        Message::LiveSessionFinalized(result) => {
            state.live_session_finalizing = false;

            match result {
                Ok(()) => {
                    let finalized_session_id = state.live_session_db_id;
                    state.live_session_db_id = None;
                    state.live_session_stopped_at = None;
                    state.live_session_started_at = None;
                    state.live_session_creating = false;
                    state.live_persisted_segment_count = state.live_completed_segments.len();

                    if state.live_transcription.is_none() {
                        state.hint = String::from("Sessao realtime salva.");
                    }

                    if state.subtitle_window_id.is_none() {
                        state.live_completed_segments.clear();
                    }

                    // Kick off title generation via ChatGPT backend (OAuth)
                    if let Some(session_id) = finalized_session_id {
                        if state.has_openai_credentials {
                            state.title_gen_failed_ids.insert(session_id);
                            eprintln!(
                                "[openvoice][title] dispatching title generation for session_id={session_id}"
                            );
                            return Task::perform(
                                async move {
                                    live_transcription_application::generate_session_title(
                                        session_id,
                                    )
                                },
                                Message::LiveSessionTitleGenerated,
                            );
                        } else {
                            eprintln!(
                                "[openvoice][title] skipped: no OAuth credentials (has_openai_credentials=false)"
                            );
                        }
                    } else {
                        eprintln!("[openvoice][title] skipped: finalized_session_id was None");
                    }

                    Task::none()
                }
                Err(err) => {
                    state.error = Some(format!("Erro ao finalizar sessao realtime: {err}"));
                    Task::none()
                }
            }
        }

        Message::LiveSessionTitleGenerated(result) => {
            match result {
                Ok((session_id, title)) => {
                    eprintln!(
                        "[openvoice][title] title generated for session {session_id}: {title}"
                    );
                    // Update the title in our cached sessions list if present
                    if let Some(session) =
                        state.sessions_list.iter_mut().find(|s| s.id == session_id)
                    {
                        session.title = Some(title.clone());
                    }
                    // Clear from failed set on success (in case it was retried)
                    state.title_gen_failed_ids.remove(&session_id);
                    state.hint = format!("Titulo gerado: {title}");
                }
                Err(err) => {
                    eprintln!("[openvoice][title] title generation failed: {err}");
                    // Title generation is best-effort; don't block on errors.
                    // NOTE: We cannot extract session_id from the error string alone,
                    // so the circuit breaker is applied before dispatching (see below).
                }
            }

            // Chain: generate title for the next session without one,
            // skipping sessions that already failed.
            if state.has_openai_credentials {
                if let Some(session) = state.sessions_list.iter().find(|s| {
                    s.title.is_none()
                        && s.segment_count > 0
                        && !state.title_gen_failed_ids.contains(&s.id)
                }) {
                    let session_id = session.id;
                    // Mark as attempted so we don't retry on failure
                    state.title_gen_failed_ids.insert(session_id);
                    eprintln!("[openvoice][title] chaining title gen for session_id={session_id}");
                    return Task::perform(
                        async move {
                            live_transcription_application::generate_session_title(session_id)
                        },
                        Message::LiveSessionTitleGenerated,
                    );
                }
            }

            Task::none()
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

            // Retroactive: generate title for the first session without one
            if state.has_openai_credentials {
                // Clear the failed set when sessions are freshly loaded
                // (new load = new opportunity to try again)
                state.title_gen_failed_ids.clear();

                if let Some(session) = state
                    .sessions_list
                    .iter()
                    .find(|s| s.title.is_none() && s.segment_count > 0)
                {
                    let session_id = session.id;
                    state.title_gen_failed_ids.insert(session_id);
                    eprintln!(
                        "[openvoice][title] retroactive title gen for session_id={session_id}"
                    );
                    return Task::perform(
                        async move {
                            live_transcription_application::generate_session_title(session_id)
                        },
                        Message::LiveSessionTitleGenerated,
                    );
                }
            }

            Task::none()
        }

        Message::SessionsSearchChanged(query) => {
            state.sessions_search_query = query;
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
        // Copilot
        // ------------------------------------------------------------------ //
        Message::CopilotInputEdited(action) => {
            state.copilot_input.perform(action);
            Task::none()
        }
        Message::CopilotModeChanged(mode) => {
            if state.copilot_mode != mode {
                state.copilot_mode = mode;
                state.copilot_thread_id = None;
                state.copilot_messages.clear();
                state.copilot_error = None;
                if state.main_view == MainView::Copilot {
                    return resize_copilot_window(state);
                }
            }
            Task::none()
        }
        Message::CopilotIncludeTranscriptChanged(value) => {
            state.copilot_include_transcript = value;
            Task::none()
        }
        Message::CaptureCopilotScreenshot => {
            if state.copilot_busy {
                return Task::none();
            }

            state.copilot_error = None;
            Task::perform(
                async move { screenshot_platform::capture_primary_display() },
                Message::CopilotScreenshotCaptured,
            )
        }
        Message::CopilotScreenshotCaptured(result) => {
            match result {
                Ok(screenshot) => {
                    state.copilot_screenshot = Some(screenshot);
                    state.copilot_error = None;
                }
                Err(error) => {
                    state.copilot_error = Some(error);
                }
            }

            Task::none()
        }
        Message::ClearCopilotScreenshot => {
            state.copilot_screenshot = None;
            Task::none()
        }
        Message::SubmitCopilotRequest => {
            if state.copilot_busy {
                return Task::none();
            }

            if !state.has_openai_credentials {
                state.copilot_error = Some(String::from(
                    "Faca login com ChatGPT nas settings antes de usar o copiloto.",
                ));
                return Task::none();
            }

            let question = state.copilot_input.text();
            if question.trim().is_empty() {
                state.copilot_error = Some(String::from(
                    "Escreva uma pergunta antes de chamar o copiloto.",
                ));
                return Task::none();
            }

            state.copilot_busy = true;
            state.copilot_error = None;
            state
                .copilot_messages
                .push(CopilotChatMessage::user(question.trim().to_owned()));

            let settings = state.settings.clone();
            let context = build_copilot_context(state, question);
            let thread_id = state.copilot_thread_id;
            state.copilot_input = text_editor::Content::new();

            Task::perform(
                async move { copilot_application::answer_question(&settings, context, thread_id) },
                Message::CopilotAnswerReceived,
            )
        }
        Message::CopilotAnswerReceived(result) => {
            state.copilot_busy = false;

            match result {
                Ok(CopilotAnswer { answer, thread_id }) => {
                    state
                        .copilot_messages
                        .push(CopilotChatMessage::assistant(answer));
                    state.copilot_thread_id = thread_id;
                    state.copilot_error = None;
                }
                Err(error) => {
                    state.copilot_error = Some(error);
                }
            }

            Task::none()
        }
        Message::CopilotMarkdownLinkClicked(_uri) => Task::none(),
        Message::CopyCopilotAnswer => {
            let Some(answer) = state
                .copilot_messages
                .iter()
                .rev()
                .find(|message| message.role == CopilotRole::Assistant)
                .map(|message| message.content.clone())
            else {
                return Task::none();
            };

            Task::batch([
                iced::clipboard::write(answer.clone()),
                iced::clipboard::write_primary(answer),
            ])
        }

        // ------------------------------------------------------------------ //
        // Window behavior
        // ------------------------------------------------------------------ //
        Message::TogglePassthrough => {
            if state.main_view == MainView::Home {
                return Task::none();
            }

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

/// Morph the main window from Home (700x800 Normal) back to HUD (380x96 AlwaysOnTop).
/// Returns window tasks. Caller should batch these with follow-up actions.
fn morph_home_to_hud(state: &mut Overlay) -> Vec<Task<Message>> {
    state.main_view = MainView::Hud;
    state.error = None;

    let Some(window_id) = state.main_window_id else {
        return Vec::new();
    };

    apply_main_window_settings(
        state,
        window_id,
        app_window::hud_settings(),
        window::Level::AlwaysOnTop,
    )
}

fn push_live_delta(target: &mut String, delta: &str) {
    if target.is_empty() {
        target.push_str(delta.trim_start());
    } else if target.ends_with(char::is_whitespace) && delta.starts_with(char::is_whitespace) {
        target.push_str(delta.trim_start());
    } else {
        target.push_str(delta);
    }
}

fn resolve_completed_transcript(
    item_id: &str,
    transcript: &str,
    partial_item_id: Option<&str>,
    partial_transcript: &str,
) -> String {
    let transcript = transcript.trim();
    if !transcript.is_empty() {
        return transcript.to_owned();
    }

    if partial_item_id == Some(item_id) {
        return partial_transcript.trim().to_owned();
    }

    String::new()
}

fn queue_live_persistence(state: &mut Overlay) -> Task<Message> {
    if state.live_session_db_id.is_some()
        && !state.live_segments_persisting
        && state.live_persisted_segment_count < state.live_completed_segments.len()
    {
        return queue_pending_live_segments(state);
    }

    queue_finalize_live_session(state)
}

fn queue_pending_live_segments(state: &mut Overlay) -> Task<Message> {
    if state.live_segments_persisting {
        return Task::none();
    }

    let Some(session_id) = state.live_session_db_id else {
        return Task::none();
    };

    let start = state.live_persisted_segment_count;
    if start >= state.live_completed_segments.len() {
        return Task::none();
    }

    state.live_segments_persisting = true;
    let segments = state.live_completed_segments[start..].to_vec();

    Task::perform(
        async move { db::append_live_segments(session_id, start, segments) },
        Message::LiveSessionSegmentsPersisted,
    )
}

fn queue_finalize_live_session(state: &mut Overlay) -> Task<Message> {
    if state.live_session_finalizing
        || state.live_session_creating
        || state.live_segments_persisting
        || state.live_persisted_segment_count < state.live_completed_segments.len()
    {
        return Task::none();
    }

    let Some(session_id) = state.live_session_db_id else {
        return Task::none();
    };
    let Some(stopped_at) = state.live_session_stopped_at.clone() else {
        return Task::none();
    };

    state.live_session_finalizing = true;

    Task::perform(
        async move { db::finalize_live_session(session_id, stopped_at) },
        Message::LiveSessionFinalized,
    )
}

fn open_copilot_view(state: &mut Overlay) -> Task<Message> {
    if state.is_processing() {
        state.error = Some(String::from(
            "Finalize o processamento atual antes de abrir o copiloto.",
        ));
        return Task::none();
    }

    if state.main_view != MainView::Copilot {
        state.previous_main_view = state.main_view;
    }

    state.main_view = MainView::Copilot;
    state.copilot_error = None;
    state.error = None;
    state.passthrough_enabled = false;

    resize_copilot_window(state)
}

fn close_copilot_view(state: &mut Overlay) -> Task<Message> {
    let target = state.previous_main_view;
    state.main_view = target;
    state.copilot_busy = false;
    state.copilot_error = None;

    let Some(window_id) = state.main_window_id else {
        return Task::none();
    };

    let tasks = match target {
        MainView::Hud => apply_main_window_settings(
            state,
            window_id,
            app_window::hud_settings(),
            window::Level::AlwaysOnTop,
        ),
        MainView::Home => apply_main_window_settings(
            state,
            window_id,
            app_window::home_window_settings(),
            window::Level::Normal,
        ),
        MainView::Copilot => apply_main_window_settings(
            state,
            window_id,
            app_window::hud_settings(),
            window::Level::AlwaysOnTop,
        ),
    };

    Task::batch(tasks)
}

fn apply_main_window_settings(
    state: &mut Overlay,
    window_id: window::Id,
    settings: window::Settings,
    level: window::Level,
) -> Vec<Task<Message>> {
    let position = match settings.position {
        window::Position::Specific(point) if state.main_view == MainView::Hud => {
            state.hud_position.unwrap_or(point)
        }
        window::Position::Specific(point) => point,
        _ => Point::ORIGIN,
    };

    let passthrough_task = if state.main_view == MainView::Hud && state.passthrough_enabled {
        window::enable_mouse_passthrough(window_id)
    } else {
        window::disable_mouse_passthrough(window_id)
    };

    vec![
        passthrough_task,
        window::resize(window_id, settings.size),
        window::move_to(window_id, position),
        window::set_level(window_id, level),
    ]
}

fn resize_copilot_window(state: &mut Overlay) -> Task<Message> {
    state.main_window_id.map_or_else(Task::none, |window_id| {
        let settings = if state.copilot_mode == CopilotMode::Meeting {
            app_window::copilot_compact_window_settings()
        } else {
            app_window::copilot_chat_window_settings()
        };

        Task::batch(apply_main_window_settings(
            state,
            window_id,
            settings,
            window::Level::AlwaysOnTop,
        ))
    })
}

fn build_copilot_context(state: &Overlay, question: String) -> CopilotContext {
    let (session_id, session_label, mut transcript_segments) =
        if state.is_live_transcribing() || !state.live_completed_segments.is_empty() {
            let mut segments = state.live_completed_segments.clone();
            if !state.live_partial_transcript.trim().is_empty() {
                segments.push(state.live_partial_transcript.trim().to_owned());
            }

            (
                state.live_session_db_id,
                Some(String::from("live transcript")),
                segments,
            )
        } else if let Some(session_id) = state.selected_session_id {
            (
                Some(session_id),
                Some(format!("saved session #{session_id}")),
                state.selected_session_segments.clone(),
            )
        } else {
            (None, None, Vec::new())
        };

    if !state.copilot_include_transcript {
        transcript_segments.clear();
    }

    CopilotContext {
        mode: state.copilot_mode,
        question,
        transcript_segments,
        session_id,
        session_label,
        screenshot: state.copilot_screenshot.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_copilot_context, push_live_delta, resolve_completed_transcript};
    use crate::app::state::boot;
    use crate::modules::copilot::domain::CopilotMode;

    #[test]
    fn appends_delta_without_double_leading_space() {
        let mut transcript = String::from("hello ");
        push_live_delta(&mut transcript, " world");

        assert_eq!(transcript, "hello world");
    }

    #[test]
    fn falls_back_to_partial_when_completed_is_empty() {
        let transcript = resolve_completed_transcript("item-1", "", Some("item-1"), "partial text");
        assert_eq!(transcript, "partial text");
    }

    #[test]
    fn copilot_context_prefers_live_segments() {
        let (mut state, _task) = boot();
        state.copilot_mode = CopilotMode::Interview;
        state.live_completed_segments = vec![String::from("recent segment")];
        state.selected_session_id = Some(99);
        state.selected_session_segments = vec![String::from("saved session")];

        let context = build_copilot_context(&state, String::from("help"));

        assert_eq!(context.session_label.as_deref(), Some("live transcript"));
        assert_eq!(
            context.transcript_segments,
            vec![String::from("recent segment")]
        );
    }
}
