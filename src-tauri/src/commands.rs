use serde::Serialize;
use std::fs;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct Document {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: u64,
}

fn get_app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
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
pub fn list_documents(app: AppHandle) -> Result<Vec<Document>, String> {
    let app_data_dir = get_app_data_dir(&app)?;
    let documents_dir = app_data_dir.join("documents");
    
    if !documents_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut documents = Vec::new();
    
    let entries = fs::read_dir(&documents_dir)
        .map_err(|e| format!("Failed to read documents directory: {}", e))?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                documents.push(Document {
                    name: path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    modified: metadata.modified()
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0))
                        .unwrap_or(0),
                });
            }
        }
    }
    
    Ok(documents)
}
