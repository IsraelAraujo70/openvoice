//! OpenVoice - Voice-to-clipboard transcription app
//! Main library with Tauri setup, commands, and state management

mod audio;
mod transcription;

use audio::{AudioDevice, AudioRecorder};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{
    async_runtime,
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_store::StoreExt;
use transcription::TranscriptionClient;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_key: String,
    pub audio_device: Option<String>,
    pub model: Option<String>,
}

/// Application state - wrapped in Arc for thread-safe sharing
pub struct AppState {
    pub recorder: Arc<AudioRecorder>,
    pub transcription_client: TranscriptionClient,
    pub tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    pub is_processing: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            recorder: Arc::new(AudioRecorder::new()),
            transcription_client: TranscriptionClient::new(),
            tray_icon: Arc::new(Mutex::new(None)),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ============================================================================
// Core Recording Logic (Backend-controlled)
// ============================================================================

/// Handle the recording toggle - this is called from UI and tray
fn handle_toggle_recording(app: &AppHandle) {
    let state: State<AppState> = app.state();
    
    // Prevent multiple simultaneous operations
    if state.is_processing.load(Ordering::SeqCst) {
        log::warn!("Already processing, ignoring toggle");
        return;
    }
    
    let is_recording = state.recorder.is_recording();
    log::info!("Toggle recording called. Currently recording: {}", is_recording);
    
    if is_recording {
        // Stop recording and transcribe
        handle_stop_and_transcribe(app);
    } else {
        // Start recording
        handle_start_recording(app);
    }
}

/// Start recording - called from backend
fn handle_start_recording(app: &AppHandle) {
    let state: State<AppState> = app.state();
    
    if state.recorder.is_recording() {
        log::warn!("Already recording");
        return;
    }
    
    // Check if we have an API key
    let api_key = get_api_key(app);
    if api_key.is_empty() {
        log::error!("No API key configured");
        // Show settings window
        if let Some(window) = app.get_webview_window("settings") {
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }
    
    log::info!("=== STARTING RECORDING ===");
    
    // Show main window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    
    // Update UI via event
    let _ = app.emit("recording-started", ());
    
    // Update tray
    update_tray_icon(app, true);
    
    // Start recording in background thread
    let recorder = Arc::clone(&state.recorder);
    thread::spawn(move || {
        log::info!("Recording thread started");
        if let Err(e) = recorder.start_recording() {
            log::error!("Recording error: {}", e);
        }
        log::info!("Recording thread ended");
    });
    
    // Give it a moment to start
    thread::sleep(std::time::Duration::from_millis(100));
}

/// Stop recording and transcribe - called from backend
fn handle_stop_and_transcribe(app: &AppHandle) {
    let state: State<AppState> = app.state();
    
    if !state.recorder.is_recording() {
        log::warn!("Not recording, nothing to stop");
        return;
    }
    
    // Set processing flag
    state.is_processing.store(true, Ordering::SeqCst);
    
    log::info!("=== STOPPING RECORDING ===");
    
    // Signal stop
    state.recorder.signal_stop();
    
    // Update UI
    let _ = app.emit("recording-stopped", ());
    
    // Update tray
    update_tray_icon(app, false);
    
    // Wait for recording to stop
    thread::sleep(std::time::Duration::from_millis(300));
    
    // Get audio data
    let audio_data = match state.recorder.get_audio_base64() {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to get audio: {}", e);
            let _ = app.emit("transcription-error", e.clone());
            state.is_processing.store(false, Ordering::SeqCst);
            return;
        }
    };
    
    log::info!("Audio data length: {} chars", audio_data.len());
    
    // Get API key
    let api_key = get_api_key(app);
    if api_key.is_empty() {
        log::error!("No API key");
        let _ = app.emit("transcription-error", "No API key configured");
        state.is_processing.store(false, Ordering::SeqCst);
        return;
    }
    
    // Transcribe in async task
    let app_handle = app.clone();
    let transcription_client = state.transcription_client.clone();
    let is_processing = Arc::clone(&state.is_processing);
    
    async_runtime::spawn(async move {
        log::info!("Starting transcription...");
        let _ = app_handle.emit("transcription-started", ());
        
        match transcription_client.transcribe(&audio_data, &api_key, None).await {
            Ok(text) => {
                log::info!("Transcription successful: {}", &text[..text.len().min(50)]);
                
                // Copy to clipboard
                if let Err(e) = app_handle.clipboard().write_text(&text) {
                    log::error!("Failed to copy to clipboard: {}", e);
                }
                
                let _ = app_handle.emit("transcription-complete", text);
            }
            Err(e) => {
                log::error!("Transcription failed: {}", e);
                let _ = app_handle.emit("transcription-error", e);
            }
        }
        
        is_processing.store(false, Ordering::SeqCst);
        
    });
}

/// Get API key from store
fn get_api_key(app: &AppHandle) -> String {
    if let Ok(store) = app.store("config.json") {
        store
            .get("api_key")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default()
    } else {
        String::new()
    }
}

/// Update tray icon based on recording state
fn update_tray_icon(app: &AppHandle, is_recording: bool) {
    let state: State<AppState> = app.state();
    let tray_icon = Arc::clone(&state.tray_icon);
    
    if let Ok(tray_guard) = tray_icon.lock() {
        if let Some(tray) = tray_guard.as_ref() {
            let icon_bytes = if is_recording {
                include_bytes!("../icons/icon-recording.png").as_slice()
            } else {
                include_bytes!("../icons/icon.png").as_slice()
            };
            
            if let Ok(icon) = Image::from_bytes(icon_bytes) {
                let _ = tray.set_icon(Some(icon));
            }
            
            let tooltip = if is_recording {
                "OpenVoice - Recording..."
            } else {
                "OpenVoice"
            };
            let _ = tray.set_tooltip(Some(tooltip));
            
            // Update menu
            let record_text = if is_recording { "Stop Recording" } else { "Start Recording" };
            if let Ok(record_item) = MenuItem::with_id(app, "record", record_text, true, None::<&str>) {
                if let Ok(settings_item) = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>) {
                    if let Ok(quit_item) = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>) {
                        if let Ok(new_menu) = Menu::with_items(app, &[&record_item, &settings_item, &quit_item]) {
                            let _ = tray.set_menu(Some(new_menu));
                        }
                    }
                }
            }
        }
    };
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get list of available audio input devices
#[tauri::command]
fn get_audio_devices(state: State<AppState>) -> Vec<AudioDevice> {
    state.recorder.get_input_devices()
}

/// Start recording audio (spawns a thread)
#[tauri::command]
fn start_recording(app: AppHandle) -> Result<(), String> {
    handle_start_recording(&app);
    Ok(())
}

/// Toggle recording (start/stop)
#[tauri::command]
fn toggle_recording(app: AppHandle) -> Result<(), String> {
    handle_toggle_recording(&app);
    Ok(())
}

/// Stop recording and return base64 WAV audio
#[tauri::command]
fn stop_recording(state: State<AppState>) -> Result<String, String> {
    if !state.recorder.is_recording() {
        return Err("Not recording".to_string());
    }

    state.recorder.signal_stop();
    thread::sleep(std::time::Duration::from_millis(200));
    state.recorder.get_audio_base64()
}

/// Check if currently recording
#[tauri::command]
fn is_recording(state: State<AppState>) -> bool {
    state.recorder.is_recording()
}

/// Transcribe audio using OpenRouter API
#[tauri::command]
async fn transcribe_audio(
    audio_base64: String,
    api_key: String,
    model: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    state
        .transcription_client
        .transcribe(&audio_base64, &api_key, model.as_deref())
        .await
}

/// Set the audio device to use
#[tauri::command]
fn set_audio_device(device_name: Option<String>, state: State<AppState>) {
    state.recorder.set_device(device_name);
    log::info!("Audio device set via command");
}

/// Save configuration to store
#[tauri::command]
async fn save_config(app: AppHandle, config: Config) -> Result<(), String> {
    let store = app
        .store("config.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    store.set("api_key", serde_json::json!(config.api_key));
    store.set("audio_device", serde_json::json!(config.audio_device));
    store.set("model", serde_json::json!(config.model));
    
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;

    log::info!("Config saved");
    Ok(())
}

/// Load configuration from store
#[tauri::command]
async fn load_config(app: AppHandle) -> Result<Config, String> {
    let store = app
        .store("config.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let api_key = store
        .get("api_key")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();

    let audio_device = store
        .get("audio_device")
        .and_then(|v| v.as_str().map(String::from));

    let model = store
        .get("model")
        .and_then(|v| v.as_str().map(String::from));

    Ok(Config {
        api_key,
        audio_device,
        model,
    })
}

/// Copy text to clipboard
#[tauri::command]
fn copy_to_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}

/// Show settings window
#[tauri::command]
fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Hide settings window
#[tauri::command]
fn hide_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Update the tray icon based on recording state
#[tauri::command]
fn update_tray_recording_state(app: AppHandle, is_recording: bool) -> Result<(), String> {
    update_tray_icon(&app, is_recording);
    Ok(())
}

// ============================================================================
// Setup Functions
// ============================================================================

/// Setup system tray with recording button
fn setup_tray(app: &AppHandle, state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let record_item = MenuItem::with_id(app, "record", "Start Recording", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&record_item, &settings_item, &quit_item])?;

    let icon_bytes = include_bytes!("../icons/icon.png");
    let icon = Image::from_bytes(icon_bytes)
        .unwrap_or_else(|_| Image::new(&[0u8; 32 * 32 * 4], 32, 32));

    let app_handle = app.clone();
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("OpenVoice")
        .on_menu_event(move |_app, event| match event.id.as_ref() {
            "record" => {
                handle_toggle_recording(&app_handle);
            }
            "settings" => {
                if let Some(window) = app_handle.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app_handle.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    {
        let mut tray_guard = state.tray_icon.lock().unwrap();
        *tray_guard = Some(tray);
    }

    log::info!("System tray setup complete");
    Ok(())
}

/// Setup main window
fn setup_main_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        log::info!("Main window configured");
    }
    Ok(())
}

// ============================================================================
// Tauri Run
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            start_recording,
            toggle_recording,
            stop_recording,
            is_recording,
            transcribe_audio,
            set_audio_device,
            save_config,
            load_config,
            copy_to_clipboard,
            show_settings,
            hide_settings,
            update_tray_recording_state,
        ])
        .setup(|app| {
            let handle = app.handle();
            let state: State<AppState> = handle.state();

            if let Err(e) = setup_main_window(handle) {
                log::error!("Failed to setup main window: {}", e);
            }

            if let Err(e) = setup_tray(handle, &state) {
                log::error!("Failed to setup tray: {}", e);
            }

            log::info!("OpenVoice started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
