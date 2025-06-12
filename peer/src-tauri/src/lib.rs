mod network;
mod state;
mod utils;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::state::{AppState, File, Peer};
use anyhow::Result;
use tauri::Manager;

// in all commands replace AppState by AppStateWrapper
struct AppStateWrapper(Arc<Mutex<AppState>>);

#[tauri::command]
async fn get_paths(state: tauri::State<'_, AppStateWrapper>) -> Result<Vec<String>, String> {
    Ok(state.0.lock().await.shared_path_string())
}


#[tauri::command]
async fn add_path(
    path: String,
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<Vec<String>, String> {
    let mut state = state.0.lock().await;
    state.add_path(path).map_err(|err| err.to_string())?;
    Ok(state.shared_path_string())
}

#[tauri::command]
async fn set_username(
    username: String,
    state: tauri::State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = state.0.lock().await;
    if state.endpoint.is_none() {
        state.spawn_endpoint().await.map_err(|err| err.to_string())?;
    }
    state.update_username(username)
        .map_err(|err| err.to_string())?;
    state.start_discovery(app); // TODO Move this to a better place
    Ok(())
}
#[tauri::command]
async fn get_username(
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<String, String> {
    let state = state.0.lock().await;
    state.get_username().clone().ok_or_else(|| "Username not set".to_string())
}

#[tauri::command]
async fn get_files(state: tauri::State<'_, AppStateWrapper>) -> Result<Vec<File>, String> {
    let state = state.0.lock().await;
    Ok(state.files.clone())
}

#[tauri::command]
async fn clear_files(state: tauri::State<'_, AppStateWrapper>) -> Result<(), String> {
    let mut state = state.0.lock().await;
    state.clear_files();
    Ok(())
}

#[tauri::command]
async fn get_peers(state: tauri::State<'_, AppStateWrapper>, app: tauri::AppHandle) -> Result<Vec<Peer>, String> {
    let mut state = state.0.lock().await;
    state.get_peers(app).await.map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            app.manage(AppStateWrapper(Arc::new(Mutex::new(AppState::new().expect("Failed to create AppState")))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_username,
            get_username,
            get_paths,
            add_path,
            get_files,
            clear_files,
            get_peers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
