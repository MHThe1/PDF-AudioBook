import { useEffect, useRef, useState, useCallback } from 'react';

interface WordTiming {
  word: string;
  start_ms: number;
  end_ms: number;
}

interface TeleprompterProps {
  text: string;
  wordTimings: WordTiming[];
  currentPositionMs: number;
  isPlaying: boolean;
  speed: number;
}

export function Teleprompter({ 
  text, 
  wordTimings, 
  currentPositionMs, 
  isPlaying,
  speed 
}: TeleprompterProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [currentWordIndex, setCurrentWordIndex] = useState(0);

  // Find current word based on position
  useEffect(() => {
    if (wordTimings.length === 0) return;

    // Adjust position for speed
    const adjustedPosition = currentPositionMs;
    
    let newIndex = 0;
    for (let i = 0; i < wordTimings.length; i++) {
      if (adjustedPosition >= wordTimings[i].start_ms) {
        newIndex = i;
      } else {
        break;
      }
    }
    
    setCurrentWordIndex(newIndex);
  }, [currentPositionMs, wordTimings, speed]);

  // Auto-scroll to current word
  useEffect(() => {
    if (!containerRef.current || wordTimings.length === 0) return;

    const currentWordEl = containerRef.current.querySelector('.word.current');
    if (currentWordEl) {
      const container = containerRef.current;
      const wordRect = currentWordEl.getBoundingClientRect();
      const containerRect = container.getBoundingClientRect();
      
      // Calculate target scroll position (center the current word)
      const targetScrollTop = container.scrollTop + 
        (wordRect.top - containerRect.top) - 
        (containerRect.height / 2) + 
        (wordRect.height / 2);

      container.scrollTo({
        top: targetScrollTop,
        behavior: 'smooth'
      });
    }
  }, [currentWordIndex, wordTimings.length]);

  const words = text.split(/\s+/).filter(w => w.length > 0);

  if (!text) {
    return (
      <div className="teleprompter empty">
        <div className="empty-state">
          <span className="empty-icon">📖</span>
          <p>Select a PDF to start reading</p>
        </div>
      </div>
    );
  }

  return (
    <div 
      className={`teleprompter ${isPlaying ? 'playing' : 'paused'}`}
      ref={containerRef}
    >
      <div className="teleprompter-content">
        {words.map((word, index) => {
          const isPast = index < currentWordIndex;
          const isCurrent = index === currentWordIndex;
          const isUpcoming = index > currentWordIndex;
          
          return (
            <span
              key={index}
              className={`word ${isPast ? 'past' : ''} ${isCurrent ? 'current' : ''} ${isUpcoming ? 'upcoming' : ''}`}
            >
              {word}{' '}
            </span>
          );
        })}
      </div>
    </div>
  );
}
