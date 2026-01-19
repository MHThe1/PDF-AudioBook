use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub audio_path: String,
    pub word_timings: Vec<WordTiming>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsError {
    pub message: String,
}

/// Get the path to the Piper TTS executable
fn get_piper_path() -> PathBuf {
    // Look for piper in the resources directory
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    
    // Try multiple locations (including dev mode paths)
    let possible_paths = vec![
        // Production: next to exe
        exe_dir.join("piper").join("piper.exe"),
        // Dev mode: project root (exe is in src-tauri/target/debug)
        exe_dir.join("..").join("..").join("..").join("piper").join("piper.exe"),
        exe_dir.join("..").join("..").join("..").join("..").join("piper").join("piper.exe"),
        // Current working directory
        PathBuf::from("piper").join("piper.exe"),
        // Absolute fallback for this specific project
        PathBuf::from("F:\\Programming\\PdfAudio\\piper\\piper.exe"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Fallback - assume it's in PATH
    PathBuf::from("piper")
}

/// Get the default voice model path
fn get_voice_model_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let possible_paths = vec![
        // Production: next to exe
        exe_dir.join("piper").join("voices").join("en_US-amy-medium.onnx"),
        // Dev mode: project root (exe is in src-tauri/target/debug)
        exe_dir.join("..").join("..").join("..").join("piper").join("voices").join("en_US-amy-medium.onnx"),
        exe_dir.join("..").join("..").join("..").join("..").join("piper").join("voices").join("en_US-amy-medium.onnx"),
        // Current working directory
        PathBuf::from("piper").join("voices").join("en_US-amy-medium.onnx"),
        // Absolute fallback for this specific project
        PathBuf::from("F:\\Programming\\PdfAudio\\piper\\voices\\en_US-amy-medium.onnx"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Fallback
    PathBuf::from("piper/voices/en_US-amy-medium.onnx")
}

/// Estimate word timings based on text and speech rate
/// Average speaking rate is about 150 words per minute
pub fn estimate_word_timings(text: &str, speed: f32) -> Vec<WordTiming> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let words_per_second = (150.0 * speed) / 60.0;
    let ms_per_word = (1000.0 / words_per_second) as u64;
    
    let mut timings = Vec::new();
    let mut current_ms = 0u64;
    
    for word in words {
        // Adjust timing based on word length
        let word_length_factor = (word.len() as f32 / 5.0).max(0.5).min(2.0);
        let duration = (ms_per_word as f32 * word_length_factor) as u64;
        
        timings.push(WordTiming {
            word: word.to_string(),
            start_ms: current_ms,
            end_ms: current_ms + duration,
        });
        
        current_ms += duration;
    }
    
    timings
}

/// Generate audio from text using Piper TTS
pub fn generate_audio(text: &str, output_path: &str) -> Result<TtsResult, TtsError> {
    let piper_path = get_piper_path();
    let model_path = get_voice_model_path();
    
    // Check if Piper exists
    if !piper_path.exists() && which::which("piper").is_err() {
        return Err(TtsError {
            message: format!(
                "Piper TTS not found. Please download it from https://github.com/rhasspy/piper/releases and place it in the 'piper' folder. Looking for: {}",
                piper_path.display()
            ),
        });
    }

    // Run Piper to generate audio
    let mut child = Command::new(&piper_path)
        .args([
            "--model", model_path.to_str().unwrap_or(""),
            "--output_file", output_path,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| TtsError {
            message: format!("Failed to start Piper: {}", e),
        })?;

    // Write text to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).map_err(|e| TtsError {
            message: format!("Failed to write to Piper stdin: {}", e),
        })?;
    }

    let output = child.wait_with_output().map_err(|e| TtsError {
        message: format!("Failed to wait for Piper: {}", e),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TtsError {
            message: format!("Piper failed: {}", stderr),
        });
    }

    // Estimate word timings
    let word_timings = estimate_word_timings(text, 1.0);
    let duration_ms = word_timings.last().map(|w| w.end_ms).unwrap_or(0);

    Ok(TtsResult {
        audio_path: output_path.to_string(),
        word_timings,
        duration_ms,
    })
}

/// Check if Piper TTS is available
pub fn is_piper_available() -> bool {
    let piper_path = get_piper_path();
    piper_path.exists() || which::which("piper").is_ok()
}

/// Get voice model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub name: String,
    pub language: String,
    pub available: bool,
}

pub fn get_available_voices() -> Vec<VoiceInfo> {
    let model_path = get_voice_model_path();
    
    vec![VoiceInfo {
        name: "Amy (US English)".to_string(),
        language: "en-US".to_string(),
        available: model_path.exists(),
    }]
}
