use std::sync::Arc;


use anyhow::anyhow;
use anyhow::Result;
use futures_lite::StreamExt;
use iroh::Endpoint;
use tauri::Emitter;
use tauri::{AppHandle, Manager};

use crate::state::Peer;
use crate::AppStateWrapper;

pub async fn setup_iroh() -> Result<Endpoint> {
    // TODO set username on discovery
    let endpoint = Endpoint::builder().discovery_local_network().bind().await?;
    Ok(endpoint)
}

pub async fn run_discovery(app: AppHandle) -> Result<()> {
    let state_lock = app
        .try_state::<AppStateWrapper>()
        .ok_or_else(|| anyhow!("Error accessing AppState"))?; // `state_lock` lives long enough

    let mut state = state_lock.0.lock().await;

    let mut stream = state.endpoint.as_mut().unwrap().discovery_stream();
    let peers = Arc::clone(&state.peers);
    drop(state);

    while let Some(event) = stream.next().await {
        match event {
            Ok(discovery_item) => {
                let node_id = discovery_item.node_id();
                let user_data = discovery_item
                    .node_info()
                    .data
                    .user_data()
                    .map(|ud| ud.as_ref())
                    .unwrap_or("");

                // TODO verify username is with the host
                //if !host.contains(&user_data) {
                //    continue;
                //}
                let peer = Peer {
                    node_id,
                    username: user_data.to_owned(),
                };

                {
                    let mut peers_lock = peers.lock().await;
                    if let Some(old_peer) = peers_lock.iter_mut().find(|p| p.node_id == peer.node_id && p.username != peer.username) {
                        let _ =
                            app.emit("peer::username_changed", (old_peer.clone(), peer.clone()));
                        println!(
                            "Peer username changed: {} -> {}",
                            old_peer.username, peer.username
                        );
                        *old_peer = peer;
                    } else if !peers_lock.iter().any(|p| p.node_id == peer.node_id) {
                        peers_lock.push(peer.clone());
                        let _ = app.emit("peer::added", &peer);
                        println!("New peer added: {}", peer.username);
                    }
                }
            }
            Err(e) => {
                eprintln!("Lagged or stream error: {:?}", e);
            }
        }
    }

    Ok(())
}
