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
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_store::StoreExt;
use transcription::TranscriptionClient;

/// Default shortcut string
const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+V";
/// Shortcut to stop recording (Escape key)
const STOP_SHORTCUT: &str = "Escape";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_key: String,
    pub audio_device: Option<String>,
    pub model: Option<String>,
    pub shortcut: Option<String>,
}

/// Application state - wrapped in Arc for thread-safe sharing
pub struct AppState {
    pub recorder: Arc<AudioRecorder>,
    pub transcription_client: TranscriptionClient,
    pub tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    pub current_shortcut: Arc<Mutex<Option<Shortcut>>>,
    pub is_processing: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            recorder: Arc::new(AudioRecorder::new()),
            transcription_client: TranscriptionClient::new(),
            tray_icon: Arc::new(Mutex::new(None)),
            current_shortcut: Arc::new(Mutex::new(None)),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ============================================================================
// Shortcut Parsing
// ============================================================================

/// Parse a shortcut string like "Ctrl+Shift+V" into a Tauri Shortcut
fn parse_shortcut_string(s: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    
    if parts.is_empty() {
        return Err("Empty shortcut string".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "ALT" => modifiers |= Modifiers::ALT,
            "META" | "SUPER" | "CMD" | "COMMAND" => modifiers |= Modifiers::META,
            // Letters
            "A" => key_code = Some(Code::KeyA),
            "B" => key_code = Some(Code::KeyB),
            "C" => key_code = Some(Code::KeyC),
            "D" => key_code = Some(Code::KeyD),
            "E" => key_code = Some(Code::KeyE),
            "F" => key_code = Some(Code::KeyF),
            "G" => key_code = Some(Code::KeyG),
            "H" => key_code = Some(Code::KeyH),
            "I" => key_code = Some(Code::KeyI),
            "J" => key_code = Some(Code::KeyJ),
            "K" => key_code = Some(Code::KeyK),
            "L" => key_code = Some(Code::KeyL),
            "M" => key_code = Some(Code::KeyM),
            "N" => key_code = Some(Code::KeyN),
            "O" => key_code = Some(Code::KeyO),
            "P" => key_code = Some(Code::KeyP),
            "Q" => key_code = Some(Code::KeyQ),
            "R" => key_code = Some(Code::KeyR),
            "S" => key_code = Some(Code::KeyS),
            "T" => key_code = Some(Code::KeyT),
            "U" => key_code = Some(Code::KeyU),
            "V" => key_code = Some(Code::KeyV),
            "W" => key_code = Some(Code::KeyW),
            "X" => key_code = Some(Code::KeyX),
            "Y" => key_code = Some(Code::KeyY),
            "Z" => key_code = Some(Code::KeyZ),
            // Numbers
            "0" => key_code = Some(Code::Digit0),
            "1" => key_code = Some(Code::Digit1),
            "2" => key_code = Some(Code::Digit2),
            "3" => key_code = Some(Code::Digit3),
            "4" => key_code = Some(Code::Digit4),
            "5" => key_code = Some(Code::Digit5),
            "6" => key_code = Some(Code::Digit6),
            "7" => key_code = Some(Code::Digit7),
            "8" => key_code = Some(Code::Digit8),
            "9" => key_code = Some(Code::Digit9),
            // Function keys
            "F1" => key_code = Some(Code::F1),
            "F2" => key_code = Some(Code::F2),
            "F3" => key_code = Some(Code::F3),
            "F4" => key_code = Some(Code::F4),
            "F5" => key_code = Some(Code::F5),
            "F6" => key_code = Some(Code::F6),
            "F7" => key_code = Some(Code::F7),
            "F8" => key_code = Some(Code::F8),
            "F9" => key_code = Some(Code::F9),
            "F10" => key_code = Some(Code::F10),
            "F11" => key_code = Some(Code::F11),
            "F12" => key_code = Some(Code::F12),
            // Special keys
            "SPACE" => key_code = Some(Code::Space),
            "ENTER" | "RETURN" => key_code = Some(Code::Enter),
            "TAB" => key_code = Some(Code::Tab),
            "ESCAPE" | "ESC" => key_code = Some(Code::Escape),
            "BACKSPACE" => key_code = Some(Code::Backspace),
            "DELETE" | "DEL" => key_code = Some(Code::Delete),
            "INSERT" | "INS" => key_code = Some(Code::Insert),
            "HOME" => key_code = Some(Code::Home),
            "END" => key_code = Some(Code::End),
            "PAGEUP" => key_code = Some(Code::PageUp),
            "PAGEDOWN" => key_code = Some(Code::PageDown),
            "UP" | "ARROWUP" => key_code = Some(Code::ArrowUp),
            "DOWN" | "ARROWDOWN" => key_code = Some(Code::ArrowDown),
            "LEFT" | "ARROWLEFT" => key_code = Some(Code::ArrowLeft),
            "RIGHT" | "ARROWRIGHT" => key_code = Some(Code::ArrowRight),
            other => return Err(format!("Unknown key: {}", other)),
        }
    }

    let code = key_code.ok_or("No key specified in shortcut")?;
    
    // Allow shortcuts without modifiers for special keys like Escape
    Ok(Shortcut::new(if modifiers.is_empty() { None } else { Some(modifiers) }, code))
}

// ============================================================================
// Core Recording Logic (Backend-controlled)
// ============================================================================

/// Handle the recording toggle - this is called from shortcuts and tray
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
        
        // Hide window after a delay
        if let Some(window) = app_handle.get_webview_window("main") {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            let _ = window.hide();
        }
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
                "OpenVoice - Recording... (Press Escape to stop)"
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
    store.set("shortcut", serde_json::json!(config.shortcut));
    
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

    let shortcut = store
        .get("shortcut")
        .and_then(|v| v.as_str().map(String::from));

    Ok(Config {
        api_key,
        audio_device,
        model,
        shortcut,
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

/// Update the global shortcut
#[tauri::command]
async fn update_shortcut(app: AppHandle, new_shortcut: String, state: State<'_, AppState>) -> Result<(), String> {
    let new_parsed = parse_shortcut_string(&new_shortcut)?;
    
    // Unregister current shortcut first
    {
        let mut current = state.current_shortcut.lock().map_err(|e| e.to_string())?;
        if let Some(old_shortcut) = current.take() {
            if let Err(e) = app.global_shortcut().unregister(old_shortcut) {
                log::warn!("Failed to unregister old shortcut: {}", e);
            }
        }
    }
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    if app.global_shortcut().is_registered(new_parsed) {
        let _ = app.global_shortcut().unregister(new_parsed);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    
    // Register new shortcut
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(new_parsed, move |_app, _shortcut, _event| {
        log::info!("Start shortcut triggered");
        handle_toggle_recording(&app_handle);
    }).map_err(|e| format!("Failed to set shortcut handler: {}", e))?;
    
    {
        let mut current = state.current_shortcut.lock().map_err(|e| e.to_string())?;
        *current = Some(new_parsed);
    }
    
    log::info!("New shortcut registered: {}", new_shortcut);
    Ok(())
}

/// Get current shortcut string
#[tauri::command]
async fn get_current_shortcut(app: AppHandle) -> Result<String, String> {
    let store = app
        .store("config.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let shortcut = store
        .get("shortcut")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());
    
    Ok(shortcut)
}

// ============================================================================
// Setup Functions
// ============================================================================

/// Setup global shortcuts
fn setup_global_shortcuts(app: &AppHandle, state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    // Load start shortcut from config
    let start_shortcut_str = if let Ok(store) = app.store("config.json") {
        store
            .get("shortcut")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string())
    } else {
        DEFAULT_SHORTCUT.to_string()
    };

    let start_shortcut = parse_shortcut_string(&start_shortcut_str)
        .unwrap_or_else(|_| Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV));

    log::info!("Setting up start shortcut: {}", start_shortcut_str);

    // Register START shortcut (configurable)
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(start_shortcut, move |_app, _shortcut, _event| {
        log::info!("Start/Toggle shortcut triggered");
        handle_toggle_recording(&app_handle);
    })?;
    
    {
        let mut current = state.current_shortcut.lock().unwrap();
        *current = Some(start_shortcut);
    }

    // Register STOP shortcut (Escape - always available)
    let stop_shortcut = Shortcut::new(None, Code::Escape);
    let app_handle2 = app.clone();
    app.global_shortcut().on_shortcut(stop_shortcut, move |_app, _shortcut, _event| {
        let state: State<AppState> = app_handle2.state();
        if state.recorder.is_recording() {
            log::info!("Escape pressed - stopping recording");
            handle_stop_and_transcribe(&app_handle2);
        }
    })?;

    log::info!("Global shortcuts registered: {} (start/toggle), Escape (stop)", start_shortcut_str);

    Ok(())
}

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
        window.hide()?;
        log::info!("Main window configured (hidden by default)");
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            start_recording,
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
            update_shortcut,
            get_current_shortcut,
        ])
        .setup(|app| {
            let handle = app.handle();
            let state: State<AppState> = handle.state();

            if let Err(e) = setup_main_window(handle) {
                log::error!("Failed to setup main window: {}", e);
            }

            if let Err(e) = setup_global_shortcuts(handle, &state) {
                log::error!("Failed to setup global shortcuts: {}", e);
            }

            if let Err(e) = setup_tray(handle, &state) {
                log::error!("Failed to setup tray: {}", e);
            }

            log::info!("OpenVoice started successfully");
            log::info!("Press your configured shortcut to start recording, Escape to stop");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
