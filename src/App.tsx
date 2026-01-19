import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PdfUploader } from './components/PdfUploader';
import { Teleprompter } from './components/Teleprompter';
import { AudioControls } from './components/AudioControls';
import './App.css';

interface WordTiming {
  word: string;
  start_ms: number;
  end_ms: number;
}

interface TtsResult {
  audio_path: string;
  word_timings: WordTiming[];
  duration_ms: number;
}

interface AudioState {
  is_playing: boolean;
  position_ms: number;
  duration_ms: number;
  speed: number;
  volume: number;
}

function App() {
  // PDF State
  const [paragraphs, setParagraphs] = useState<string[]>([]);
  const [isLoadingPdf, setIsLoadingPdf] = useState(false);

  // Audio State
  const [hasAudio, setHasAudio] = useState(false);
  const [isPreparing, setIsPreparing] = useState(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const [positionMs, setPositionMs] = useState(0);
  const [durationMs, setDurationMs] = useState(0);
  const [speed, setSpeed] = useState(1.0);
  const [volume, setVolume] = useState(1.0);
  const [wordTimings, setWordTimings] = useState<WordTiming[]>([]);
  const [error, setError] = useState<string | null>(null);
  
  // TTS availability
  const [ttsAvailable, setTtsAvailable] = useState<boolean | null>(null);
  
  const positionIntervalRef = useRef<number | null>(null);

  // Check TTS availability on mount
  useEffect(() => {
    const checkTts = async () => {
      try {
        const available = await invoke<boolean>('check_tts_available');
        setTtsAvailable(available);
      } catch (err) {
        console.error('Failed to check TTS:', err);
        setTtsAvailable(false);
      }
    };
    checkTts();
  }, []);

  // Poll for audio position while playing
  useEffect(() => {
    if (isPlaying) {
      positionIntervalRef.current = window.setInterval(async () => {
        try {
          const state = await invoke<AudioState>('get_audio_state');
          setPositionMs(state.position_ms);
          setIsPlaying(state.is_playing);
          
          // Check if finished
          const finished = await invoke<boolean>('is_audio_finished');
          if (finished) {
            setIsPlaying(false);
          }
        } catch (err) {
          console.error('Failed to get audio state:', err);
        }
      }, 50); // Update every 50ms for smooth tracking
    } else {
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current);
        positionIntervalRef.current = null;
      }
    }

    return () => {
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current);
      }
    };
  }, [isPlaying]);

  // Clear PDF and reset state
  const handleClear = async () => {
    try {
      await invoke('stop_audio');
    } catch (e) {
      // Ignore if no audio
    }
    setParagraphs([]);
    setHasAudio(false);
    setWordTimings([]);
    setPositionMs(0);
    setDurationMs(0);
    setIsPlaying(false);
    setError(null);
  };

  // Handle PDF loaded
  const handlePdfLoaded = useCallback(async (newParagraphs: string[]) => {
    setParagraphs(newParagraphs);
    setHasAudio(false);
    setWordTimings([]);
    setPositionMs(0);
    setError(null);

    // Automatically prepare audio
    const fullText = newParagraphs.join(' ');
    if (fullText.trim()) {
      setIsPreparing(true);
      try {
        const result = await invoke<TtsResult>('prepare_audio', { text: fullText });
        setWordTimings(result.word_timings);
        setDurationMs(result.duration_ms);
        setHasAudio(true);
      } catch (err) {
        setError(`Failed to prepare audio: ${err}`);
        console.error('TTS Error:', err);
      } finally {
        setIsPreparing(false);
      }
    }
  }, []);

  // Audio controls
  const handlePlay = async () => {
    try {
      await invoke('play_audio');
      setIsPlaying(true);
    } catch (err) {
      setError(`Failed to play: ${err}`);
    }
  };

  const handlePause = async () => {
    try {
      await invoke('pause_audio');
      setIsPlaying(false);
    } catch (err) {
      setError(`Failed to pause: ${err}`);
    }
  };

  const handleSpeedChange = async (newSpeed: number) => {
    try {
      await invoke('set_speed', { speed: newSpeed });
      setSpeed(newSpeed);
      // Recalculate word timings with new speed
      const fullText = paragraphs.join(' ');
      const timings = await invoke<WordTiming[]>('get_word_timings', { 
        text: fullText, 
        speed: newSpeed 
      });
      setWordTimings(timings);
    } catch (err) {
      console.error('Failed to set speed:', err);
    }
  };

  const handleVolumeChange = async (newVolume: number) => {
    try {
      await invoke('set_volume', { volume: newVolume });
      setVolume(newVolume);
    } catch (err) {
      console.error('Failed to set volume:', err);
    }
  };

  const fullText = paragraphs.join(' ');

  return (
    <div className="app">
      <header className="app-header">
        <h1>📖 PDF Audiobook</h1>
        <p className="subtitle">Listen to your PDFs with natural voice</p>
      </header>

      {ttsAvailable === false && (
        <div className="tts-warning">
          <h3>⚠️ Piper TTS Not Found</h3>
          <p>
            Please download <a href="https://github.com/rhasspy/piper/releases" target="_blank" rel="noopener">Piper TTS</a> and 
            place it in a <code>piper</code> folder in the app directory.
          </p>
        </div>
      )}

      {error && (
        <div className="error-banner">
          <span>⚠️ {error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <main className="app-main">
        <aside className="sidebar">
          <PdfUploader
            onPdfLoaded={handlePdfLoaded}
            isLoading={isLoadingPdf}
            setIsLoading={setIsLoadingPdf}
          />
          
          {paragraphs.length > 0 && (
            <div className="pdf-info">
              <p><strong>{paragraphs.length}</strong> paragraphs</p>
              <p><strong>{fullText.split(/\s+/).length}</strong> words</p>
              <button className="clear-button" onClick={handleClear}>
                ✕ Clear PDF
              </button>
            </div>
          )}
        </aside>

        <section className="content">
          <Teleprompter
            text={fullText}
            wordTimings={wordTimings}
            currentPositionMs={positionMs}
            isPlaying={isPlaying}
            speed={speed}
          />
        </section>
      </main>

      <footer className="app-footer">
        <AudioControls
          isPlaying={isPlaying}
          isPreparing={isPreparing}
          hasAudio={hasAudio}
          positionMs={positionMs}
          durationMs={durationMs}
          speed={speed}
          volume={volume}
          onPlay={handlePlay}
          onPause={handlePause}
          onSpeedChange={handleSpeedChange}
          onVolumeChange={handleVolumeChange}
        />
      </footer>
    </div>
  );
}

export default App;
