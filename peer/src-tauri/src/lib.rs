mod network;
mod state;
mod utils;
use tracing::{instrument, trace, debug, info, error, warn};
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::state::{AppState, PeerSerializable, File, AppStateWrapper};
use anyhow::Result;
use tauri::Manager;

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn get_paths(state: tauri::State<'_, AppStateWrapper>) -> Result<Vec<String>, String> {
    Ok(state.0.lock().await.shared_path_string())
}

#[instrument(skip(state), ret, err)]
#[tauri::command]
async fn add_path(
    path: String,
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<Vec<String>, String> {
    let mut state = state.0.lock().await;
    state.add_path(path).map_err(|err| err.to_string())?;
    Ok(state.shared_path_string())
}

#[instrument(skip(state, app), ret, err)]
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

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn get_username(
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<String, String> {
    let state = state.0.lock().await;
    state.get_username().clone().ok_or_else(|| "Username not set".to_string())
}

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn get_files(state: tauri::State<'_, AppStateWrapper>) -> Result<Vec<File>, String> {
    let state = state.0.lock().await;
    Ok(state.files.clone())
}

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn clear_files(state: tauri::State<'_, AppStateWrapper>) -> Result<(), String> {
    let mut state = state.0.lock().await;
    state.clear_files();
    Ok(())
}

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn get_peers(state: tauri::State<'_, AppStateWrapper>, app: tauri::AppHandle) -> Result<Vec<PeerSerializable>, String> {
    let mut state = state.0.lock().await;
    state.get_peers(app).await.map_err(|err| err.to_string())
}

#[tauri::command]
fn log(level: String, message: String, context: Option<serde_json::Value>) {
    match level.as_str() {
        "error" => error!(message, ?context),
        "warn" => warn!(message, ?context),
        "info" => info!(message, ?context),
        "debug" => debug!(message, ?context),
        "trace" => trace!(message, ?context),
        _ => info!(message, ?context),
    }
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
            get_peers,
            log
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
