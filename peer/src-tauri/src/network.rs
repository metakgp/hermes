use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use futures_lite::StreamExt;
use iroh::Endpoint;
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{instrument, info, error};

use crate::state::Peer;
use crate::state::PeerSerializable;
use crate::state::AppStateWrapper;

#[instrument(ret, err)]
pub async fn setup_iroh() -> Result<Endpoint> {
    // TODO set username on discovery
    let endpoint = Endpoint::builder().discovery_local_network().bind().await?;
    Ok(endpoint)
}

#[instrument(skip_all, ret, err)]
pub async fn run_discovery(app: AppHandle) -> Result<()> {
    let state_lock = app
        .try_state::<AppStateWrapper>()
        .ok_or_else(|| anyhow!("Error accessing AppState"))?; // `state_lock` lives long enough

    let mut state = state_lock.0.lock().await;

    let mut stream = state.endpoint.as_mut().unwrap().discovery_stream();
    let peers = Arc::clone(&state.peers);
    drop(state);
    let cleaner_handle = tokio::spawn(background_cleanup_task(
        Arc::clone(&peers),
        app.clone(),
        tokio::time::Duration::from_secs(10), // TODO make this configurable
    ));

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
                    last_seen: Instant::now(),
                };

                {
                    let mut peer_lock = peers.lock().await;
                    if let Some(old_peer) = peer_lock
                        .iter_mut()
                        .find(|p| p.node_id == peer.node_id && p.username != peer.username)
                    {
                        let payload: (PeerSerializable, PeerSerializable) =
                            (old_peer.clone().into(), peer.clone().into());
                        let _ = app.emit("peer::username_changed", payload);
                        info!(
                            old_username = old_peer.username,
                            new_username = peer.username, 
                            "Peer username changed: {} -> {}", old_peer.username, peer.username
                        );
                        *old_peer = peer;
                    } else if !peer_lock.iter().any(|p| p.node_id == peer.node_id) {
                        peer_lock.push(peer.clone());
                        let payload: PeerSerializable = peer.clone().into();
                        let _ = app.emit("peer::added", payload);
                        info!(
                            new_peer_username = peer.username,
                            "New peer added: {}", peer.username
                        );
                    }
                }
            }
            Err(e) => {
                error!(Error = ?e, "Lagged or stream error: {:?}", e);
            }
        }
    }
    cleaner_handle.abort();

    Ok(())
}

/// Periodically checks for peers that have not been seen within the timeout period,
/// removes them from the tracker, and emits a "peer::left" event for each.
pub async fn background_cleanup_task(
    peers: Arc<Mutex<Vec<Peer>>>,
    app: AppHandle,
    timeout: tokio::time::Duration,
) {
    let mut interval = tokio::time::interval(timeout);
    loop {
        interval.tick().await;
        let mut peers_lock = peers.lock().await;

        let left_peers: Vec<Peer> = peers_lock
            .iter()
            .filter(|peer| Instant::now().duration_since(peer.last_seen) > timeout)
            .cloned()
            .collect();

        for left_peer in left_peers {
            peers_lock.retain(|p| p.node_id != left_peer.node_id);
            let payload: PeerSerializable = left_peer.clone().into();
            let _ = app.emit("peer::left", payload);
            println!("Peer left: {}", left_peer.username);
        }
    }
}
