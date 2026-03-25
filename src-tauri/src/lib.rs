use log::info;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

mod commands;
mod document;
mod config;

pub fn get_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

pub fn ensure_directories(app_data_dir: &PathBuf) -> Result<(), String> {
    let dirs = ["documents", "chroma", "logs"];
    
    for dir_name in dirs {
        let dir_path = app_data_dir.join(dir_name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create {} directory: {}", dir_name, e))?;
            info!("Created directory: {:?}", dir_path);
        }
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = get_app_data_dir(app.handle())?;
            
            info!("local-rag starting up...");
            info!("App data directory: {:?}", app_data_dir);
            
            fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;
            
            ensure_directories(&app_data_dir)?;
            
            info!("local-rag initialized successfully");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_app_data_dir_cmd,
            commands::list_documents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
