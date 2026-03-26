use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::document::{DocumentMetadata, DocumentProcessor, TextChunk};

#[derive(Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessingResult {
    pub metadata: DocumentMetadata,
    pub chunks: Vec<TextChunk>,
}

fn get_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn get_app_data_dir_cmd(app: AppHandle) -> Result<String, String> {
    let dir = get_app_data_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn pick_file(app: AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("Documents", &["pdf", "txt", "md", "markdown"])
        .add_filter("PDF Files", &["pdf"])
        .add_filter("Text Files", &["txt", "md", "markdown"])
        .blocking_pick_file();

    match file_path {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn process_document(app: AppHandle, file_path: String) -> Result<ProcessingResult, String> {
    let app_data_dir = get_app_data_dir(&app)?;
    let documents_dir = app_data_dir.join("documents");
    
    fs::create_dir_all(&documents_dir)
        .map_err(|e| format!("Failed to create documents directory: {}", e))?;

    let processor = DocumentProcessor::new(documents_dir.clone());
    let path = PathBuf::from(&file_path);

    let (metadata, chunks) = processor
        .process_document(&path)
        .map_err(|e| format!("Failed to process document: {}", e))?;

    let metadata_path = app_data_dir.join("document_metadata.json");
    
    let mut metadata_list: Vec<DocumentMetadata> = if metadata_path.exists() {
        let content = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    metadata_list.push(metadata.clone());
    
    let json = serde_json::to_string_pretty(&metadata_list)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_path, json)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    Ok(ProcessingResult { metadata, chunks })
}

#[tauri::command]
pub async fn get_documents_metadata(app: AppHandle) -> Result<Vec<DocumentMetadata>, String> {
    let app_data_dir = get_app_data_dir(&app)?;
    let metadata_path = app_data_dir.join("document_metadata.json");
    
    if !metadata_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metadata: {}", e))
}

#[tauri::command]
pub async fn delete_document(app: AppHandle, document_id: String) -> Result<(), String> {
    let app_data_dir = get_app_data_dir(&app)?;
    let metadata_path = app_data_dir.join("document_metadata.json");
    
    if !metadata_path.exists() {
        return Err("No documents found".to_string());
    }

    let content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    let mut metadata_list: Vec<DocumentMetadata> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metadata: {}", e))?;

    let original_len = metadata_list.len();
    metadata_list.retain(|doc| doc.id != document_id);

    if metadata_list.len() == original_len {
        return Err("Document not found".to_string());
    }

    let json = serde_json::to_string_pretty(&metadata_list)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_path, json)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    Ok(())
}
