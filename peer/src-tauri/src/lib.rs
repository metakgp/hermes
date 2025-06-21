mod global;
mod network;
mod state;
mod utils;
use tracing::{instrument, trace, debug, info, error, warn};
use std::{path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use crate::state::{AppState, AppStateWrapper, PeerSerializable};
use anyhow::Result;
use tauri::Manager;

#[instrument(skip(state), ret, err)]
#[tauri::command]
async fn add_path(path: String, state: tauri::State<'_, AppStateWrapper>) -> Result<(), String> {
    let state = state.0.lock().await;
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err("Path must be absolute".to_string());
    }
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }
    state
        .file_protocol
        .clone()
        .ok_or("File protocol not initialized")?
        .add_folder(path)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[instrument(skip(state), ret, err)]
#[tauri::command]
async fn get_uploaded_files_tree(
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<network::protocol::TreeNode, String> {
    let state = state.0.lock().await;
    let file_protocol = state
        .file_protocol
        .as_ref()
        .ok_or("File protocol not initialized")?;

    file_protocol
        .get_files_tree()
        .await
        .map_err(|e| format!("Failed to get files tree: {}", e))
}

#[instrument(skip(state), ret, err)]
#[tauri::command]
async fn get_shared_paths(
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<Vec<PathBuf>, String> {
    let state = state.0.lock().await;
    state
        .file_protocol
        .as_ref()
        .ok_or("File protocol not initialized")?
        .get_shared_paths()
        .await
        .map_err(|err| err.to_string())
}

#[instrument(skip(state), ret, err)]
#[tauri::command]
async fn ping_peer(
    peer_id: String,
    state: tauri::State<'_, AppStateWrapper>,
) -> Result<(), String> {
    let state = state.0.lock().await;
    if state.router.is_none() {
        return Err("Endpoint not initialized".to_string());
    }
    let endpoint = state.router.clone().unwrap().endpoint().clone();
    let node_id =
        iroh::NodeId::from_str(peer_id.as_str()).map_err(|_| "Invalid node ID".to_string())?;
    let node_addr = state
        .peers
        .lock()
        .await
        .iter()
        .find_map(|peer| {
            if peer.node_addr.node_id == node_id {
                Some(peer.node_addr.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| "Peer not found".to_string())?;
    crate::network::protocol::client::ping_peer(&endpoint, node_addr)
        .await
        .map_err(|err| err.to_string())
}

#[instrument(skip(state, app), ret, err)]
#[tauri::command]
async fn set_username(
    username: String,
    state: tauri::State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = state.0.lock().await;
    if state.router.is_none() {
        state
            .spawn_endpoint(app.clone())
            .await
            .map_err(|err| err.to_string())?;
    }
    state
        .update_username(username)
        .map_err(|err| err.to_string())?;
    state.start_discovery(app); // TODO Move this to a better place
    Ok(())
}

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn get_username(state: tauri::State<'_, AppStateWrapper>) -> Result<String, String> {
    let state = state.0.lock().await;
    state
        .get_username()
        .clone()
        .ok_or_else(|| "Username not set".to_string())
}

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn clear_files(state: tauri::State<'_, AppStateWrapper>) -> Result<(), String> {
    let mut state = state.0.lock().await;
    state
        .file_protocol
        .as_mut()
        .ok_or("File protocol not initialized")?
        .clear_all_files()
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[instrument(skip_all, ret, err)]
#[tauri::command]
async fn get_peers(
    state: tauri::State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<Vec<PeerSerializable>, String> {
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
            app.manage(AppStateWrapper(Arc::new(Mutex::new(
                AppState::new().expect("Failed to create AppState"),
            ))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_username,
            get_username,
            add_path,
            get_shared_paths,
            clear_files,
            get_peers,
            log,
            ping_peer,
            get_uploaded_files_tree,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
