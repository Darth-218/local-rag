use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::info;
use lopdf::Document;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Failed to read file: {0}")]
    ReadError(String),
    #[error("Failed to parse PDF: {0}")]
    PdfParseError(String),
    #[error("Unsupported file type: {0}")]
    UnsupportedType(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub chat_id: String,
    pub name: String,
    pub file_path: String,
    pub file_type: String,
    pub size: u64,
    pub page_count: Option<u32>,
    pub word_count: u32,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub id: String,
    pub document_id: String,
    pub chat_id: String,
    pub content: String,
    pub chunk_index: u32,
    pub start_char: u32,
    pub end_char: u32,
}

pub struct DocumentProcessor {
    chunk_size: usize,
    chunk_overlap: usize,
    documents_dir: PathBuf,
}

impl DocumentProcessor {
    pub fn new(documents_dir: PathBuf) -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            documents_dir,
        }
    }

    pub fn get_file_type(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    pub fn is_supported(path: &Path) -> bool {
        matches!(
            Self::get_file_type(path),
            Some(ext) if ["pdf", "txt", "md", "markdown", "text"].contains(&ext.as_str())
        )
    }

    pub fn extract_pdf_text(path: &Path) -> Result<String> {
        info!("Extracting text from PDF: {:?}", path);
        
        let doc = Document::load(path)
            .map_err(|e| DocumentError::PdfParseError(e.to_string()))?;

        let mut text_content = Vec::new();
        let mut pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
        pages.sort();

        for page_num in pages {
            if let Ok(content) = doc.extract_text(&[page_num]) {
                if !content.trim().is_empty() {
                    text_content.push(content);
                }
            }
        }

        let result = text_content.join("\n\n");
        info!("Extracted {} characters from PDF", result.len());
        Ok(result)
    }

    pub fn extract_text_file(path: &Path) -> Result<String> {
        info!("Reading text file: {:?}", path);
        let content = fs::read_to_string(path)?;
        info!("Read {} characters from text file", content.len());
        Ok(content)
    }

    pub fn extract_text(path: &Path) -> Result<String> {
        match Self::get_file_type(path) {
            Some(ext) if ext == "pdf" => Self::extract_pdf_text(path),
            Some(_) => Self::extract_text_file(path),
            None => Err(DocumentError::UnsupportedType(
                "No file extension found".to_string(),
            ).into()),
        }
    }

    pub fn get_pdf_page_count(path: &Path) -> Result<u32> {
        let doc = Document::load(path)
            .map_err(|e| DocumentError::PdfParseError(e.to_string()))?;
        Ok(doc.get_pages().len() as u32)
    }

    pub fn count_words(text: &str) -> u32 {
        text.split_whitespace().count() as u32
    }

    pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<(u32, u32, String)> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        
        if total_chars == 0 {
            return chunks;
        }

        let mut start = 0;
        let mut chunk_index = 0;

        while start < total_chars {
            let end = (start + chunk_size).min(total_chars);
            
            let chunk_text: String = chars[start..end].iter().collect();
            
            chunks.push((chunk_index, start as u32, chunk_text.trim().to_string()));

            chunk_index += 1;
            
            if end >= total_chars {
                break;
            }
            
            start = end - overlap.min(end);
        }

        info!("Created {} chunks from {} characters", chunks.len(), total_chars);
        chunks
    }

    pub fn process_document(&self, source_path: &Path, chat_id: &str) -> Result<(DocumentMetadata, Vec<TextChunk>)> {
        if !source_path.exists() {
            return Err(DocumentError::ReadError(
                format!("File not found: {:?}", source_path)
            ).into());
        }

        if !Self::is_supported(source_path) {
            return Err(DocumentError::UnsupportedType(
                source_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ).into());
        }

        let id = Uuid::new_v4().to_string();
        let file_type = Self::get_file_type(source_path)
            .unwrap_or_else(|| "unknown".to_string());
        
        let metadata = fs::metadata(source_path)?;
        let file_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let chat_dir = self.documents_dir.join(chat_id);
        fs::create_dir_all(&chat_dir)?;

        let dest_path = chat_dir.join(format!("{}_{}", id, file_name));
        fs::copy(source_path, &dest_path)
            .context("Failed to copy document to storage")?;

        let text = Self::extract_text(&dest_path)?;
        let word_count = Self::count_words(&text);
        
        let page_count = if file_type == "pdf" {
            Self::get_pdf_page_count(&dest_path).ok()
        } else {
            None
        };

        let now = Utc::now();
        let doc_metadata = DocumentMetadata {
            id: id.clone(),
            chat_id: chat_id.to_string(),
            name: file_name,
            file_path: dest_path.to_string_lossy().to_string(),
            file_type,
            size: metadata.len(),
            page_count,
            word_count,
            created_at: now,
            modified_at: now,
        };

        let raw_chunks = Self::chunk_text(&text, self.chunk_size, self.chunk_overlap);
        let chunks: Vec<TextChunk> = raw_chunks
            .into_iter()
            .map(|(idx, start, content)| {
                let content_len = content.len() as u32;
                TextChunk {
                    id: Uuid::new_v4().to_string(),
                    document_id: id.clone(),
                    chat_id: chat_id.to_string(),
                    content,
                    chunk_index: idx,
                    start_char: start,
                    end_char: start + content_len,
                }
            })
            .collect();

        info!("Processed document '{}' for chat '{}': {} chunks", doc_metadata.name, chat_id, chunks.len());

        Ok((doc_metadata, chunks))
    }
}
