use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioState {
    pub is_playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub speed: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioError {
    pub message: String,
}

// Commands sent to the audio thread
#[derive(Debug)]
pub enum AudioCommand {
    Load { path: String, duration_ms: u64 },
    Play,
    Pause,
    Stop,
    SetSpeed(f32),
    SetVolume(f32),
    GetState(Sender<AudioState>),
    IsFinished(Sender<bool>),
}

// Thread-safe audio controller that communicates with the audio thread
pub struct AudioController {
    command_tx: Sender<AudioCommand>,
    state: Arc<Mutex<AudioState>>,
}

// Make AudioController Send + Sync by not storing non-Send types
unsafe impl Send for AudioController {}
unsafe impl Sync for AudioController {}

impl AudioController {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<AudioCommand>();
        let state = Arc::new(Mutex::new(AudioState {
            is_playing: false,
            position_ms: 0,
            duration_ms: 0,
            speed: 1.0,
            volume: 1.0,
        }));

        let state_clone = state.clone();

        // Spawn audio thread
        thread::spawn(move || {
            run_audio_thread(command_rx, state_clone);
        });

        AudioController { command_tx, state }
    }

    pub fn load(&self, path: &str, duration_ms: u64) -> Result<(), AudioError> {
        self.command_tx
            .send(AudioCommand::Load {
                path: path.to_string(),
                duration_ms,
            })
            .map_err(|e| AudioError {
                message: format!("Failed to send load command: {}", e),
            })
    }

    pub fn play(&self) -> Result<(), AudioError> {
        self.command_tx.send(AudioCommand::Play).map_err(|e| AudioError {
            message: format!("Failed to send play command: {}", e),
        })
    }

    pub fn pause(&self) -> Result<(), AudioError> {
        self.command_tx.send(AudioCommand::Pause).map_err(|e| AudioError {
            message: format!("Failed to send pause command: {}", e),
        })
    }

    pub fn stop(&self) -> Result<(), AudioError> {
        self.command_tx.send(AudioCommand::Stop).map_err(|e| AudioError {
            message: format!("Failed to send stop command: {}", e),
        })
    }

    pub fn set_speed(&self, speed: f32) {
        let _ = self.command_tx.send(AudioCommand::SetSpeed(speed));
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.command_tx.send(AudioCommand::SetVolume(volume));
    }

    pub fn get_state(&self) -> AudioState {
        self.state.lock().unwrap().clone()
    }

    pub fn is_finished(&self) -> bool {
        let (tx, rx) = mpsc::channel();
        if self.command_tx.send(AudioCommand::IsFinished(tx)).is_ok() {
            rx.recv_timeout(Duration::from_millis(100)).unwrap_or(true)
        } else {
            true
        }
    }
}

fn run_audio_thread(command_rx: Receiver<AudioCommand>, state: Arc<Mutex<AudioState>>) {
    use rodio::{Decoder, OutputStream, Sink};
    use std::fs::File;
    use std::io::BufReader;

    let mut sink: Option<Sink> = None;
    let mut _stream: Option<OutputStream> = None;
    let mut start_time: Option<Instant> = None;
    let mut pause_position = Duration::ZERO;
    let mut duration = Duration::ZERO;
    let mut speed = 1.0f32;
    let mut volume = 1.0f32;
    let mut is_playing = false;

    loop {
        // Process commands with timeout to allow state updates
        match command_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(cmd) => match cmd {
                AudioCommand::Load { path, duration_ms } => {
                    // Stop any existing playback
                    if let Some(s) = sink.take() {
                        s.stop();
                    }

                    // Create new audio output
                    match OutputStream::try_default() {
                        Ok((stream, stream_handle)) => {
                            match File::open(&path) {
                                Ok(file) => {
                                    let reader = BufReader::new(file);
                                    match Decoder::new(reader) {
                                        Ok(source) => {
                                            match Sink::try_new(&stream_handle) {
                                                Ok(new_sink) => {
                                                    new_sink.set_speed(speed);
                                                    new_sink.set_volume(volume);
                                                    new_sink.append(source);
                                                    new_sink.pause();
                                                    sink = Some(new_sink);
                                                    _stream = Some(stream);
                                                    duration = Duration::from_millis(duration_ms);
                                                    pause_position = Duration::ZERO;
                                                    start_time = None;
                                                    is_playing = false;
                                                }
                                                Err(e) => eprintln!("Sink error: {}", e),
                                            }
                                        }
                                        Err(e) => eprintln!("Decoder error: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("File open error: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Audio output error: {}", e),
                    }
                }
                AudioCommand::Play => {
                    if let Some(ref s) = sink {
                        s.play();
                        start_time = Some(Instant::now());
                        is_playing = true;
                    }
                }
                AudioCommand::Pause => {
                    if let Some(ref s) = sink {
                        s.pause();
                        if let Some(start) = start_time.take() {
                            pause_position += start.elapsed();
                        }
                        is_playing = false;
                    }
                }
                AudioCommand::Stop => {
                    if let Some(s) = sink.take() {
                        s.stop();
                    }
                    _stream = None;
                    start_time = None;
                    pause_position = Duration::ZERO;
                    is_playing = false;
                }
                AudioCommand::SetSpeed(s) => {
                    speed = s.clamp(0.5, 2.0);
                    if let Some(ref sink) = sink {
                        sink.set_speed(speed);
                    }
                }
                AudioCommand::SetVolume(v) => {
                    volume = v.clamp(0.0, 1.0);
                    if let Some(ref sink) = sink {
                        sink.set_volume(volume);
                    }
                }
                AudioCommand::GetState(tx) => {
                    let position = if let Some(start) = start_time {
                        pause_position + start.elapsed()
                    } else {
                        pause_position
                    };
                    let position_ms = (position.as_secs_f64() * speed as f64 * 1000.0) as u64;

                    let _ = tx.send(AudioState {
                        is_playing,
                        position_ms,
                        duration_ms: duration.as_millis() as u64,
                        speed,
                        volume,
                    });
                }
                AudioCommand::IsFinished(tx) => {
                    let finished = sink.as_ref().map(|s| s.empty()).unwrap_or(true);
                    let _ = tx.send(finished);
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Update state periodically
                let position = if let Some(start) = start_time {
                    pause_position + start.elapsed()
                } else {
                    pause_position
                };
                let position_ms = (position.as_secs_f64() * speed as f64 * 1000.0) as u64;

                if let Ok(mut s) = state.lock() {
                    s.is_playing = is_playing;
                    s.position_ms = position_ms;
                    s.duration_ms = duration.as_millis() as u64;
                    s.speed = speed;
                    s.volume = volume;
                }

                // Check if finished
                if let Some(ref s) = sink {
                    if s.empty() && is_playing {
                        is_playing = false;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Controller dropped, exit thread
                break;
            }
        }
    }
}

pub fn create_audio_controller() -> AudioController {
    AudioController::new()
}
