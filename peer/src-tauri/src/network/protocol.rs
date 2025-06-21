use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_lite::future::Boxed as BoxedFuture;
use iroh::endpoint::Connection;
use iroh::endpoint::RecvStream;
use iroh::endpoint::SendStream;
use iroh::protocol::ProtocolHandler;
use iroh::NodeAddr;
use iroh_blobs::format::collection::Collection;
use iroh_blobs::util::SetTagOption;
use iroh_blobs::Hash;
use iroh_blobs::Tag;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::Mutex;
pub const ALPN: &[u8] = b"hermes/file-protocol/0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    id: String,
    name: String,
    hash: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolRequestCommand {
    Ping,
    ListFileRequest { filter: Option<FileFilter> },
    Quit,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolResponseCommand {
    ListFileResponse { files: Vec<SharedFileInfo> },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFileInfo {
    pub hash: iroh_blobs::Hash,
    pub name: String,
    pub size: u64,
    pub path: std::path::PathBuf,
    pub children: Option<Vec<SharedFileInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: Option<String>,
    /// (min_size, max_size)
    pub size_range: Option<(u64, u64)>,
    pub is_collection: Option<bool>,
    pub depth: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct VersionMessage<T> {
    version: u16,
    data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    supported_versions: Vec<u16>,
    current_version: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMessage {
    pub file_name: String,
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
}

const CURRENT_PROTOCOL_VERSION: u16 = 1;
const SUPPORTED_VERSIONS: [u16; 1] = [CURRENT_PROTOCOL_VERSION];
type BlobsClient = iroh_blobs::rpc::client::blobs::Client<
    quic_rpc::transport::flume::FlumeConnector<
        iroh_blobs::rpc::proto::Response,
        iroh_blobs::rpc::proto::Request,
    >,
>;
#[derive(Debug, Clone)]
pub struct FileProtocol {
    uploaded_files: Arc<Mutex<Vec<SharedFileInfo>>>,
    taggeduploaded_files: Arc<Mutex<BTreeMap<Hash, Tag>>>,
    blobs_client: BlobsClient,
    app_handle: AppHandle,
}

impl ProtocolHandler for FileProtocol {
    fn accept(&self, connection: Connection) -> BoxedFuture<Result<()>> {
        let this = self.clone();
        Box::pin(async move {
            let node_id = connection.remote_node_id()?;
            println!("accepted connection from {node_id}");
            let (mut send, mut recv) = connection.accept_bi().await?;

            // Could be used to handle different protocol versions in the future
            let _negotiated_version =
                negotiate_version(&mut send, &mut recv, ConnectionRole::Listener).await?;

            loop {
                let command: ProtocolRequestCommand = recv_msg(&mut recv).await?;
                match command {
                    ProtocolRequestCommand::Ping => {
                        println!("Received ping, sending pong.");
                        let response = ProtocolResponseCommand::Pong;
                        send_msg(&mut send, &response).await?;
                    }
                    ProtocolRequestCommand::ListFileRequest { filter } => {
                        let files = {
                            let uploaded_files = this.uploaded_files.lock().await;
                            let mut filtered_files = uploaded_files.clone();
                            if let Some(filter) = filter {
                                if let Some(name) = filter.name {
                                    // TODO implement fuzzy search
                                    filtered_files.retain(|f| f.name.contains(&name));
                                }
                                if let Some((min_size, max_size)) = filter.size_range {
                                    filtered_files
                                        .retain(|f| f.size >= min_size && f.size <= max_size);
                                }
                                if let Some(is_collection) = filter.is_collection {
                                    filtered_files
                                        .retain(|f| f.children.is_some() == is_collection);
                                }
                                if let Some(depth) = filter.depth {
                                    //filtered_files.retain(|f| f.depth == depth);
                                }
                            }
                            filtered_files
                        };
                        let response = ProtocolResponseCommand::ListFileResponse { files };
                        send_msg(&mut send, &response).await?;
                    }
                    ProtocolRequestCommand::Quit => {
                        println!("Received quit command, closing connection.");
                        break;
                    }
                }
            }

            Ok(())
        })
    }
}

impl FileProtocol {
    pub fn new(blobs_client: BlobsClient, app: AppHandle) -> Self {
        Self {
            uploaded_files: Arc::new(Mutex::new(Vec::new())),
            blobs_client,
            app_handle: app,
            taggeduploaded_files: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
    pub fn get_all_files(&self) -> Vec<SharedFileInfo> {
        let uploaded_files = self.uploaded_files.blocking_lock();
        uploaded_files.clone()
    }

    async fn add_single_file(&mut self, file_path: impl AsRef<Path>) -> Result<SharedFileInfo> {
        let file_path = file_path.as_ref();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_size = std::fs::metadata(&file_path)
            .with_context(|| format!("Failed to get metadata for file: {}", file_path.display()))?
            .len();

        // Add file with progress tracking
        let mut progress = self
            .blobs_client
            .add_from_path(
                file_path.to_owned(),
                false,
                SetTagOption::Auto,
                iroh_blobs::rpc::client::blobs::WrapOption::Wrap { name: None },
            )
            .await
            .with_context(|| format!("Failed to add file: {}", file_path.display()))?;

        use futures_lite::StreamExt;

        while let Some(event) = progress.next().await {
            match event {
                Ok(iroh_blobs::provider::AddProgress::AllDone { hash, .. }) => {
                    let file_info = SharedFileInfo {
                        hash,
                        name: file_name.clone(),
                        size: file_size,
                        path: file_path.to_owned(),
                        children: None,
                    };

                    // Emit completion event
                    self.app_handle.emit(
                        "upload-progress",
                        ProgressMessage {
                            file_name: file_name.clone(),
                            bytes_uploaded: file_size,
                            total_bytes: file_size,
                            percentage: 100.0,
                        },
                    )?;

                    return Ok(file_info);
                }
                Ok(iroh_blobs::provider::AddProgress::Progress { offset, .. }) => {
                    let bytes_uploaded = offset;
                    let percentage = (bytes_uploaded as f32 / file_size as f32) * 100.0;

                    self.app_handle.emit(
                        "upload-progress",
                        ProgressMessage {
                            file_name: file_name.clone(),
                            bytes_uploaded,
                            total_bytes: file_size,
                            percentage,
                        },
                    )?;
                }
                Err(e) => {
                    Err(e).with_context(|| {
                        format!("Failed to upload file: {}", file_path.display())
                    })?;
                }

                _ => continue,
            }
        }

        anyhow::bail!("Upload completed without hash")
    }

    /// Public API: Add a folder and all its contents as a collection.
    pub async fn add_folder(&mut self, folder_path: impl AsRef<Path>) -> Result<SharedFileInfo> {
        if !folder_path.as_ref().is_absolute() {
            return Err(anyhow::anyhow!(
                "Folder/File path must be absolute: {}",
                folder_path.as_ref().display()
            ));
        }
        if !folder_path.as_ref().is_dir() {
            return self.add_single_file(folder_path).await;
        }
        self.add_folder_rec(folder_path).await
    }

    /// Internal recursive function. Do not call directly.
    async fn add_folder_rec(&mut self, folder_path: impl AsRef<Path>) -> Result<SharedFileInfo> {
        let folder_path = folder_path.as_ref();
        let folder_name = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut collection = Collection::default();
        let mut total_size = 0u64;

        let mut children = Vec::new();
        // Recursively add all files in the folder
        let entries = std::fs::read_dir(folder_path)?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            // We are not adding symlinks
            if entry_path.is_file() {
                let file_info = self.add_single_file(entry_path.to_owned()).await?;
                children.push(file_info.clone());

                let relative_path = entry_path
                    .strip_prefix(folder_path)
                    .unwrap_or(&entry_path)
                    .to_string_lossy() // possible bug here, if the path is not valid UTF-8
                    .into_owned();

                collection.push(relative_path, file_info.hash);
                total_size += file_info.size;
            } else if entry_path.is_dir() {
                // Recursively handle subdirectories
                let subfolder_info = Box::pin(self.add_folder_rec(entry_path.to_owned())).await?;
                children.push(subfolder_info.clone());

                let relative_path = entry_path
                    .strip_prefix(folder_path)
                    .unwrap_or(&entry_path)
                    .to_string_lossy()
                    .into_owned();

                collection.push(relative_path, subfolder_info.hash);
                total_size += subfolder_info.size;
            }
        }

        // collection in iroh-blobs represents a folder in our case
        let (collection_hash, _tag) = self
            .blobs_client
            .create_collection(collection, SetTagOption::Auto, vec![])
            .await?;

        let folder_info = SharedFileInfo {
            hash: collection_hash,
            name: folder_name,
            size: total_size,
            path: folder_path.to_owned(),
            children: Some(children),
        };

        Ok(folder_info)
    }
    pub async fn get_files_tree(&self) -> Result<TreeNode, anyhow::Error> {
        let tagged_files = self.taggeduploaded_files.lock().await;

        // Create a root node
        let mut root = TreeNode {
            id: "root".to_string(),
            name: "Uploaded Files".to_string(),
            hash: "".to_string(),
            path: "".to_string(),
            size: None,
            modified: None,
            children: Vec::new(),
        };

        // Build a path map to organize files into a tree
        let mut path_map: HashMap<String, Vec<(Hash, Tag)>> = HashMap::new();

        for (hash, tag) in tagged_files.iter() {
            let path_str = tag.to_string();
            let path = Path::new(&path_str);

            if let Some(parent) = path.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                path_map
                    .entry(parent_str)
                    .or_insert_with(Vec::new)
                    .push((hash.clone(), tag.clone()));
            } else {
                path_map
                    .entry("".to_string())
                    .or_insert_with(Vec::new)
                    .push((hash.clone(), tag.clone()));
            }
        }

        // Build the tree recursively
        self.build_tree(&mut root, "", &path_map)?;

        Ok(root)
    }

    fn build_tree(
        &self,
        node: &mut TreeNode,
        current_path: &str,
        path_map: &HashMap<String, Vec<(Hash, Tag)>>,
    ) -> Result<(), anyhow::Error> {
        if let Some(entries) = path_map.get(current_path) {
            for (hash, tag) in entries {
                let path_str = tag.to_string();
                let path = Path::new(&path_str);
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let metadata = fs::metadata(&path_str).ok();
                let size = metadata.as_ref().map(|m| m.len());
                let modified = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|time| {
                        let datetime: DateTime<Utc> = time.into();
                        datetime
                    });

                let mut child = TreeNode {
                    id: hash.to_string(),
                    name,
                    hash: hash.to_string(),
                    path: path_str.clone(),
                    size,
                    modified,
                    children: Vec::new(),
                };

                // If this is a directory, recursively build its children
                if path.is_dir() {
                    self.build_tree(&mut child, &path_str, path_map)?;
                }

                node.children.push(child);
            }
        }

        // Sort children by name for consistent ordering
        node.children.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(())
    }

    /// Import from a file or directory into the database.
    ///
    /// The returned tag always refers to a collection. If the input is a file, this
    /// is a collection with a single blob, named like the file.
    ///
    /// If the input is a directory, the collection contains all the files in the
    /// directory.
    pub async fn import(&self, path: impl AsRef<Path>) -> Result<(iroh_blobs::Tag, Hash)> {
        let batch = self.blobs_client.batch().await?;
        let path = path.as_ref();
        let temp_tag = batch
            .add_dir_with_opts(
                path.into(),
                iroh_blobs::rpc::client::blobs::AddDirOpts {
                    import_mode: iroh_blobs::store::ImportMode::TryReference,
                    wrap: iroh_blobs::rpc::client::blobs::WrapOption::Wrap { name: None },
                    io_parallelism: 4,
                },
            )
            .await
            .context("Failed to import file or directory")?;
        let tag = iroh_blobs::Tag::from(path.to_str().context("Not a valid UTF-8 path")?);
        let hash = temp_tag.hash().clone();
        batch.persist_to(temp_tag, tag.clone()).await?;
        drop(batch);
        self.taggeduploaded_files
            .lock()
            .await
            .insert(hash, tag.clone());
        println!("Imported {} with hash {}", path.display(), hash);
        Ok((tag, hash))
    }

    pub async fn clear_all_files(&mut self) -> Result<()> {
        self.blobs_client.tags().delete_all().await?;
        let mut uploaded_files = self.uploaded_files.lock().await;
        uploaded_files.clear();
        Ok(())
    }

    /// Can remove file or a folder (collection) from the shared files.
    pub async fn remove_file(&mut self, file_hash: iroh_blobs::Hash) -> Result<()> {
        let mut uploaded_files = self.uploaded_files.lock().await;
        // TODO check if two files can have the same hash, should WrapOption::Wrap prevent this?
        if let Some(pos) = uploaded_files.iter().position(|f| f.hash == file_hash) {
            let file_info = uploaded_files.remove(pos);
            if file_info.children.is_some() {
                let blobs = self.blobs_client.get_collection(file_info.hash).await?;
                for (name, hash) in blobs.iter() {
                    println!("Removing child blob: {} with hash: {}", name, hash);
                    self.blobs_client.delete_blob(*hash).await?;
                }
            } else {
                self.blobs_client.delete_blob(file_info.hash).await?;
            }
            Ok(())
        } else {
            anyhow::bail!("File with hash {} not found in uploaded files", file_hash)
        }
    }

    pub async fn get_shared_paths(&self) -> Result<Vec<std::path::PathBuf>> {
        let uploaded_files = self.uploaded_files.lock().await;
        Ok(uploaded_files
            .iter()
            .map(|file| file.path.clone())
            .collect())
    }
}

enum ConnectionRole {
    Listener,
    Initiator,
}
// During connection setup
async fn negotiate_version(
    send: &mut SendStream,
    recv: &mut RecvStream,
    role: ConnectionRole,
) -> Result<u16> {
    let our_versions = ProtocolVersion {
        supported_versions: SUPPORTED_VERSIONS.to_vec(),
        current_version: CURRENT_PROTOCOL_VERSION,
    };
    let their_versions: ProtocolVersion = match role {
        ConnectionRole::Listener => {
            // If we are the listener, we wait for their versions first
            let result = recv_msg(recv).await?;
            send_msg(send, &our_versions).await?;
            result
        }
        ConnectionRole::Initiator => {
            // If we are the initiator, we send our versions first
            send_msg(send, &our_versions).await?;
            recv_msg(recv).await?
        }
    };

    // Find highest mutually supported version
    let negotiated = our_versions
        .supported_versions
        .iter()
        .filter(|v| their_versions.supported_versions.contains(v))
        .max()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No common protocol version found. Our versions: {:?}, their versions: {:?}",
                our_versions.supported_versions,
                their_versions.supported_versions
            )
        })?;

    Ok(*negotiated)
}

pub async fn recv_msg<T>(recv: &mut RecvStream) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut incoming_len = [0u8; 8];
    recv.read_exact(&mut incoming_len).await?;
    let len = u64::from_le_bytes(incoming_len);

    let mut buffer = vec![0u8; len as usize];
    recv.read_exact(&mut buffer).await?;
    let msg: T = postcard::from_bytes(&buffer)?;
    Ok(msg)
}

async fn send_msg<T>(send: &mut SendStream, msg: &T) -> Result<()>
where
    T: Serialize,
{
    let encoded = postcard::to_stdvec(msg)?;
    send.write_all(&(encoded.len() as u64).to_le_bytes())
        .await?;
    send.write_all(&encoded).await?;
    Ok(())
}

///  Contains client-side functions for interacting with remote peers.
pub mod client {
    use super::*;
    pub async fn ping_peer(
        endpoint: &iroh::endpoint::Endpoint,
        node_addr: impl Into<NodeAddr>,
    ) -> Result<()> {
        let node_addr = node_addr.into();
        let conn = endpoint.connect(node_addr.clone(), ALPN).await?;

        let (mut send, mut recv) = conn.open_bi().await?;
        negotiate_version(&mut send, &mut recv, ConnectionRole::Initiator).await?;
        send_msg(&mut send, &ProtocolRequestCommand::Ping).await?;
        println!("Sent ping to {}", &node_addr.node_id);
        let recv: ProtocolResponseCommand = recv_msg(&mut recv).await?;
        println!("Received response: {:?}", recv);
        Ok(())
    }

    pub async fn list_remote_files(
        endpoint: &iroh::endpoint::Endpoint,
        node_addr: impl Into<NodeAddr>,
        filter: Option<FileFilter>,
    ) -> Result<Vec<SharedFileInfo>> {
        let node_addr = node_addr.into();
        let conn = endpoint.connect(node_addr.clone(), ALPN).await?;
        let (mut send, mut recv) = conn.open_bi().await?;
        negotiate_version(&mut send, &mut recv, ConnectionRole::Initiator).await?;

        let request = ProtocolRequestCommand::ListFileRequest { filter };
        send_msg(&mut send, &request).await?;

        let response: ProtocolResponseCommand = recv_msg(&mut recv).await?;

        match response {
            ProtocolResponseCommand::ListFileResponse { files, .. } => Ok(files),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }
}
