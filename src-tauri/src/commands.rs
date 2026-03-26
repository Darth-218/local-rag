use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::document::{DocumentMetadata, DocumentProcessor, TextChunk};
use crate::embedding::{self, ChromaEntry};
use crate::ollama::OllamaClient;

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

#[derive(Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub chunks_embedded: usize,
    pub document_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub chunks: Vec<SearchChunk>,
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchChunk {
    pub content: String,
    pub document_id: String,
    pub document_name: String,
    pub score: f32,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaStatus {
    pub available: bool,
    pub models: Vec<String>,
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

    let chroma_path = app_data_dir.join("chroma").join("index.json");
    if chroma_path.exists() {
        let content = fs::read_to_string(&chroma_path)
            .map_err(|e| format!("Failed to read chroma index: {}", e))?;
        let mut entries: Vec<ChromaEntry> = serde_json::from_str(&content)
            .unwrap_or_default();
        entries.retain(|e| e.metadata["document_id"].as_str().unwrap_or("") != document_id);
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("Failed to serialize chroma: {}", e))?;
        fs::write(&chroma_path, json)
            .map_err(|e| format!("Failed to write chroma: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn check_ollama_status() -> Result<OllamaStatus, String> {
    let client = OllamaClient::new();
    
    if !client.is_available().await {
        return Ok(OllamaStatus {
            available: false,
            models: vec![],
        });
    }

    let models = client.list_models()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?
        .into_iter()
        .map(|m| m.name)
        .collect();

    Ok(OllamaStatus {
        available: true,
        models,
    })
}

#[tauri::command]
pub async fn embed_document(
    app: AppHandle,
    document_id: String,
    model: Option<String>,
) -> Result<EmbeddingResult, String> {
    let app_data_dir = get_app_data_dir(&app)?;
    let client = OllamaClient::new();
    let embedding_model = model.unwrap_or_else(|| "nomic-embed-text".to_string());

    let metadata_path = app_data_dir.join("document_metadata.json");
    let content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    let metadata_list: Vec<DocumentMetadata> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metadata: {}", e))?;
    
    let doc_meta = metadata_list.iter()
        .find(|d| d.id == document_id)
        .ok_or("Document not found")?;

    let text = DocumentProcessor::extract_text(&PathBuf::from(&doc_meta.file_path))
        .map_err(|e| format!("Failed to extract text: {}", e))?;
    let chunks = DocumentProcessor::chunk_text(&text, 512, 50);

    let chroma_dir = app_data_dir.join("chroma");
    fs::create_dir_all(&chroma_dir)
        .map_err(|e| format!("Failed to create chroma dir: {}", e))?;

    let mut entries: Vec<ChromaEntry> = if chroma_dir.join("index.json").exists() {
        let content = fs::read_to_string(chroma_dir.join("index.json"))
            .map_err(|e| format!("Failed to read index: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    entries.retain(|e| e.metadata["document_id"].as_str().unwrap_or("") != document_id);

    for (idx, _start, content) in chunks {
        let embedding = client.generate_embedding(&embedding_model, &content)
            .await
            .map_err(|e| format!("Failed to generate embedding: {}", e))?;

        entries.push(ChromaEntry {
            id: format!("{}_{}", document_id, idx),
            embedding,
            document: content,
            metadata: serde_json::json!({
                "document_id": document_id,
                "chunk_index": idx,
                "document_name": doc_meta.name,
            }),
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize entries: {}", e))?;
    fs::write(chroma_dir.join("index.json"), json)
        .map_err(|e| format!("Failed to write index: {}", e))?;

    Ok(EmbeddingResult {
        chunks_embedded: entries.iter().filter(|e| e.metadata["document_id"].as_str() == Some(&document_id)).count(),
        document_id,
    })
}

#[tauri::command]
pub async fn search_documents(
    app: AppHandle,
    query: String,
    top_k: Option<usize>,
) -> Result<SearchResult, String> {
    let app_data_dir = get_app_data_dir(&app)?;
    let client = OllamaClient::new();
    let embedding_model = "nomic-embed-text".to_string();
    let k = top_k.unwrap_or(5);

    let chroma_path = app_data_dir.join("chroma").join("index.json");
    if !chroma_path.exists() {
        return Ok(SearchResult {
            chunks: vec![],
            query,
        });
    }

    let content = fs::read_to_string(&chroma_path)
        .map_err(|e| format!("Failed to read index: {}", e))?;
    let entries: Vec<ChromaEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse index: {}", e))?;

    let query_embedding = client.generate_embedding(&embedding_model, &query)
        .await
        .map_err(|e| format!("Failed to embed query: {}", e))?;

    let similarities = embedding::find_similar(&entries, &query_embedding, k);

    let search_chunks: Vec<SearchChunk> = similarities
        .iter()
        .filter_map(|(id, score)| {
            entries.iter().find(|e| &e.id == id).map(|e| SearchChunk {
                content: e.document.clone(),
                document_id: e.metadata["document_id"].as_str().unwrap_or("").to_string(),
                document_name: e.metadata["document_name"].as_str().unwrap_or("").to_string(),
                score: *score,
            })
        })
        .collect();

    Ok(SearchResult {
        chunks: search_chunks,
        query,
    })
}

#[tauri::command]
pub async fn ask_question(
    app: AppHandle,
    query: String,
    model: Option<String>,
) -> Result<String, String> {
    let _app_data_dir = get_app_data_dir(&app)?;
    let client = OllamaClient::new();
    let llm_model = model.unwrap_or_else(|| "phi3.5-mini".to_string());

    let search_result = search_documents(app.clone(), query.clone(), Some(5)).await?;

    if search_result.chunks.is_empty() {
        return Ok("No relevant documents found. Please add and index some documents first.".to_string());
    }

    let context = search_result.chunks
        .iter()
        .map(|c| format!("[{}]: {}", c.document_name, c.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "You are a helpful assistant answering questions based on the provided context. \
        If the answer is not in the context, say so.\n\n\
        Context:\n{}\n\n\
        Question: {}\n\n\
        Answer:",
        context, query
    );

    let response = client.generate(&llm_model, &prompt)
        .await
        .map_err(|e| format!("Failed to generate response: {}", e))?;

    Ok(response)
}
