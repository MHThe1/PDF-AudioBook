use pdf_extract::extract_text;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub paragraphs: Vec<String>,
    pub word_count: usize,
    pub page_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfError {
    pub message: String,
}

impl From<pdf_extract::OutputError> for PdfError {
    fn from(err: pdf_extract::OutputError) -> Self {
        PdfError {
            message: format!("PDF extraction error: {:?}", err),
        }
    }
}

/// Extract text content from a PDF file
pub fn extract_pdf_text(path: &str) -> Result<TextContent, PdfError> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err(PdfError {
            message: format!("File not found: {}", path.display()),
        });
    }

    let text = extract_text(path)?;
    
    // Split into paragraphs (double newlines or significant whitespace)
    let paragraphs: Vec<String> = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            // Clean up internal whitespace
            p.split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ")
        })
        .collect();

    let word_count = paragraphs
        .iter()
        .map(|p| p.split_whitespace().count())
        .sum();

    // Note: pdf-extract doesn't provide page count directly
    // We estimate based on content or use 1 as placeholder
    let page_count = 1;

    Ok(TextContent {
        paragraphs,
        word_count,
        page_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonexistent_file() {
        let result = extract_pdf_text("nonexistent.pdf");
        assert!(result.is_err());
    }
}
