use anyhow::{Context, Result};
use iroh::protocol::Router;
use iroh::Endpoint;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::network::discovery::run_discovery;
use crate::network::protocol::FileProtocol;
use crate::network::protocol::ALPN;
use iroh_blobs::net_protocol::Blobs;

#[derive(Debug)]
pub struct AppState {
    pub router: Option<iroh::protocol::Router>,
    username: Option<String>,
    discovery_task: Option<tokio::task::JoinHandle<()>>,
    pub file_protocol: Option<FileProtocol>,
    pub peers: Arc<Mutex<Vec<Peer>>>,
}

pub struct AppStateWrapper(pub Arc<Mutex<AppState>>);

impl AppState {
    pub fn new() -> Result<Self> {
        //let endpoint = setup_iroh(None).await?;
        Ok(Self {
            peers: Arc::new(Mutex::new(Vec::new())),
            router: None,
            username: None,
            discovery_task: None,
            file_protocol: None,
        })
    }

    pub async fn spawn_endpoint(&mut self, app: AppHandle) -> Result<()> {
        if self.router.is_some() {
            return Ok(());
        }
        let blobs_data_dir = crate::global::APP_DATA_DIR.join("blobs");
        let endpoint = Endpoint::builder().discovery_local_network().bind().await?;
        let blobs = Blobs::persistent(&blobs_data_dir).await?.build(&endpoint);

        // TODO Recover uploaded_files from previous session
        let proto = FileProtocol::new(blobs.client().clone(), app);
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs.clone())
            .accept(ALPN, proto.clone())
            .spawn();

        self.router = Some(router);
        self.file_protocol = Some(proto.clone());
        Ok(())
    }

    pub fn update_username(&mut self, username: String) -> Result<()> {
        match &mut self.router {
            Some(router) => {
                router
                    .endpoint()
                    .set_user_data_for_discovery(Some(username.clone().try_into()?));
                self.username = Some(username);
                Ok(())
            }
            None => {
                println!("No endpoint");
                Err(anyhow::anyhow!("No endpoint"))
            }
        }
    }

    pub fn get_username(&self) -> &Option<String> {
        &self.username
    }

    pub async fn get_peers(&mut self, app: AppHandle) -> Result<Vec<PeerSerializable>> {
        if self.discovery_task.is_none() {
            self.start_discovery(app);
        }
        Ok(self
            .peers
            .lock()
            .await
            .clone()
            .iter()
            .map(|p| p.clone().into())
            .collect())
    }
    pub fn start_discovery(&mut self, app: AppHandle) {
        if let Some(guard) = &self.discovery_task {
            if !guard.is_finished() {
                // Already running
                return;
            }
        }

        let app_clone = app.clone();

        let handle = tokio::task::spawn(async move {
            if let Err(e) = run_discovery(app_clone).await {
                eprintln!("Discovery stream error: {:?}", e);
            }
        });

        self.discovery_task = Some(handle);
    }

    pub async fn get_node_addr(&self, node_id: iroh::NodeId) -> Result<iroh::NodeAddr> {
        self.peers
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
            .context("Peer not found")
    }
}

#[derive(Clone, Debug)]
pub struct Peer {
    pub username: String,
    pub node_addr: iroh::NodeAddr,
    pub last_seen: tokio::time::Instant,
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct PeerSerializable {
    pub username: String,
    pub node_id: iroh::NodeId,
}

impl From<Peer> for PeerSerializable {
    fn from(peer: Peer) -> Self {
        Self {
            username: peer.username,
            node_id: peer.node_addr.node_id,
        }
    }
}
