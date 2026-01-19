import { useState, useCallback, DragEvent } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

interface PdfUploaderProps {
  onPdfLoaded: (paragraphs: string[]) => void;
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
}

interface TextContent {
  paragraphs: string[];
  word_count: number;
  page_count: number;
}

export function PdfUploader({ onPdfLoaded, isLoading, setIsLoading }: PdfUploaderProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);

  const handleFile = useCallback(async (path: string) => {
    setIsLoading(true);
    setError(null);

    try {
      const result = await invoke<TextContent>('extract_pdf', { path });
      setFileName(path.split(/[/\\]/).pop() || 'Unknown');
      onPdfLoaded(result.paragraphs);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  }, [onPdfLoaded, setIsLoading]);

  const handleClick = async () => {
    const selected = await open({
      filters: [{
        name: 'PDF',
        extensions: ['pdf']
      }],
      multiple: false,
    });

    if (selected && typeof selected === 'string') {
      handleFile(selected);
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = async (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    // Note: In Tauri, file drops work differently
    // For now, we'll rely on the click handler
    setError('Please click to select a file. Drag and drop support coming soon!');
  };

  return (
    <div 
      className={`pdf-uploader ${isDragging ? 'dragging' : ''} ${isLoading ? 'loading' : ''}`}
      onClick={!isLoading ? handleClick : undefined}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <div className="upload-content">
        {isLoading ? (
          <>
            <div className="loading-spinner"></div>
            <p>Processing PDF...</p>
          </>
        ) : fileName ? (
          <>
            <div className="file-icon">📄</div>
            <p className="file-name">{fileName}</p>
            <p className="click-hint">Click to select a different PDF</p>
          </>
        ) : (
          <>
            <div className="upload-icon">📚</div>
            <h3>Select a PDF</h3>
            <p>Click to browse for a PDF file</p>
          </>
        )}
      </div>
      
      {error && (
        <div className="error-message">
          <span>⚠️ {error}</span>
        </div>
      )}
    </div>
  );
}
