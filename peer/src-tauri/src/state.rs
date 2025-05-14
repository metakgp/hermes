use anyhow::Result;
use iroh::endpoint::Source;
use std::{collections::HashSet, path::PathBuf};
use tauri::AppHandle;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::network::{setup_iroh, run_discovery};

#[derive(Debug)]
pub struct AppState {
    shared_path: HashSet<PathBuf>,
    pub files: Vec<File>,
    pub endpoint: Option<iroh::endpoint::Endpoint>,
    username: Option<String>,
    discovery_task: Option<tokio::task::JoinHandle<()>>,
    pub peers: Arc<Mutex<Vec<Peer>>>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        //let endpoint = setup_iroh(None).await?;
        Ok(Self {
            shared_path: HashSet::new(),
            files: Vec::new(),
            peers: Arc::new(Mutex::new(Vec::new())),
            endpoint: None,
            username: None,
            discovery_task: None,
        })
    }

    pub async fn spawn_endpoint(&mut self) -> Result<()> {
        self.endpoint = Some(setup_iroh().await?);
        Ok(())
    }


    pub fn shared_path_string(&self) -> Vec<String> {
        self.shared_path
            .clone()
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    pub fn clear_files(&mut self) {
        self.files.clear();
        self.shared_path.clear();
    }
    pub fn add_path(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        self.shared_path.insert(path.into());
        self.update_files()?;
        Ok(())
    }

    pub fn update_files(&mut self) -> Result<()> {
        use std::fs;
        println!("Scanning shared path");
        self.files.clear();
        for dirpath in &self.shared_path {
            println!("Scanning {}", dirpath.display());
            let paths = fs::read_dir(dirpath)?;
            for path in paths {
                let path = path?;

                // Skip directories and non-existent paths
                if !path.file_type()?.is_file() {
                    continue;
                }

                // Get file name
                let name = path.file_name().to_string_lossy().into_owned();

                // Get file metadata for size
                let metadata = fs::metadata(path.path())?;

                let size = metadata.len().to_string();

                let hash = String::from("test");

                self.files.push(File { name, hash, size });
            }
        }

        Ok(())
    }

    pub fn update_username(&mut self, username: String) -> Result<()> {
        match &mut self.endpoint {
            Some(endpoint) => {
                endpoint.set_user_data_for_discovery(Some(username.clone().try_into()?));
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

    pub async fn get_peers(&mut self, app: AppHandle) -> Result<Vec<Peer>> {
        if self.discovery_task.is_none() {
            self.start_discovery(app);
        }
        Ok(self.peers.lock().await.clone())
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

}

#[derive(Clone, serde::Serialize, Debug)]
pub struct File {
    name: String,
    hash: String,
    size: String,
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct Peer {
    pub username: String,
    pub node_id: iroh::NodeId,
}

