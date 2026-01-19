import { useState, useEffect } from 'react';

interface AudioControlsProps {
  isPlaying: boolean;
  isPreparing: boolean;
  hasAudio: boolean;
  positionMs: number;
  durationMs: number;
  speed: number;
  volume: number;
  onPlay: () => void;
  onPause: () => void;
  onSpeedChange: (speed: number) => void;
  onVolumeChange: (volume: number) => void;
}

export function AudioControls({
  isPlaying,
  isPreparing,
  hasAudio,
  positionMs,
  durationMs,
  speed,
  volume,
  onPlay,
  onPause,
  onSpeedChange,
  onVolumeChange,
}: AudioControlsProps) {
  const formatTime = (ms: number) => {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const progress = durationMs > 0 ? (positionMs / durationMs) * 100 : 0;

  const speedOptions = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

  return (
    <div className="audio-controls">
      <div className="progress-section">
        <div className="progress-bar">
          <div 
            className="progress-fill" 
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="time-display">
          <span>{formatTime(positionMs)}</span>
          <span>{formatTime(durationMs)}</span>
        </div>
      </div>

      <div className="controls-main">
        <button
          className={`play-button ${isPlaying ? 'playing' : ''}`}
          onClick={isPlaying ? onPause : onPlay}
          disabled={!hasAudio || isPreparing}
        >
          {isPreparing ? (
            <span className="button-spinner"></span>
          ) : isPlaying ? (
            <svg viewBox="0 0 24 24" fill="currentColor">
              <rect x="6" y="4" width="4" height="16" rx="1" />
              <rect x="14" y="4" width="4" height="16" rx="1" />
            </svg>
          ) : (
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z" />
            </svg>
          )}
        </button>
      </div>

      <div className="controls-secondary">
        <div className="speed-control">
          <label>Speed</label>
          <select 
            value={speed} 
            onChange={(e) => onSpeedChange(parseFloat(e.target.value))}
          >
            {speedOptions.map(s => (
              <option key={s} value={s}>{s}x</option>
            ))}
          </select>
        </div>

        <div className="volume-control">
          <label>🔊</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={volume}
            onChange={(e) => onVolumeChange(parseFloat(e.target.value))}
          />
        </div>
      </div>
    </div>
  );
}
