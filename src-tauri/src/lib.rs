// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::{Manager, State};

mod audio;
mod pdf_parser;
mod tts_engine;

use audio::{create_audio_controller, AudioController, AudioState};
use pdf_parser::{extract_pdf_text, TextContent};
use tts_engine::{estimate_word_timings, generate_audio, is_piper_available, get_available_voices, TtsResult, VoiceInfo, WordTiming};

// App state for managing audio player
pub struct AppState {
    audio_controller: AudioController,
    current_text: Mutex<String>,
    temp_audio_path: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            audio_controller: create_audio_controller(),
            current_text: Mutex::new(String::new()),
            temp_audio_path: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Extract text from a PDF file
#[tauri::command]
fn extract_pdf(path: String) -> Result<TextContent, String> {
    extract_pdf_text(&path).map_err(|e| e.message)
}

/// Check if Piper TTS is available
#[tauri::command]
fn check_tts_available() -> bool {
    is_piper_available()
}

/// Get available voices
#[tauri::command]
fn get_voices() -> Vec<VoiceInfo> {
    get_available_voices()
}

/// Generate audio from text and prepare for playback
#[tauri::command]
fn prepare_audio(
    text: String,
    state: State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<TtsResult, String> {
    // Create temp directory for audio
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    
    let audio_path = app_data_dir.join("current_audio.wav");
    let audio_path_str = audio_path.to_string_lossy().to_string();

    // Generate audio using Piper TTS
    let result = generate_audio(&text, &audio_path_str).map_err(|e| e.message)?;

    // Store current text for timing
    {
        let mut current_text = state.current_text.lock().unwrap();
        *current_text = text;
    }

    // Store audio path
    {
        let mut temp_path = state.temp_audio_path.lock().unwrap();
        *temp_path = Some(audio_path_str.clone());
    }

    // Load audio into player
    state.audio_controller.load(&audio_path_str, result.duration_ms).map_err(|e| e.message)?;

    Ok(result)
}

/// Get word timings for the current text
#[tauri::command]
fn get_word_timings(text: String, speed: f32) -> Vec<WordTiming> {
    estimate_word_timings(&text, speed)
}

/// Play audio
#[tauri::command]
fn play_audio(state: State<AppState>) -> Result<(), String> {
    state.audio_controller.play().map_err(|e| e.message)
}

/// Pause audio
#[tauri::command]
fn pause_audio(state: State<AppState>) -> Result<(), String> {
    state.audio_controller.pause().map_err(|e| e.message)
}

/// Stop audio
#[tauri::command]
fn stop_audio(state: State<AppState>) {
    let _ = state.audio_controller.stop();
}

/// Set playback speed (0.5 - 2.0)
#[tauri::command]
fn set_speed(speed: f32, state: State<AppState>) {
    state.audio_controller.set_speed(speed);
}

/// Set volume (0.0 - 1.0)
#[tauri::command]
fn set_volume(volume: f32, state: State<AppState>) {
    state.audio_controller.set_volume(volume);
}

/// Get current audio state
#[tauri::command]
fn get_audio_state(state: State<AppState>) -> AudioState {
    state.audio_controller.get_state()
}

/// Check if audio playback has finished
#[tauri::command]
fn is_audio_finished(state: State<AppState>) -> bool {
    state.audio_controller.is_finished()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            extract_pdf,
            check_tts_available,
            get_voices,
            prepare_audio,
            get_word_timings,
            play_audio,
            pause_audio,
            stop_audio,
            set_speed,
            set_volume,
            get_audio_state,
            is_audio_finished,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
