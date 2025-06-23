use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_lite::future::Boxed as BoxedFuture;
use futures_lite::StreamExt;
use iroh::endpoint::Connection;
use iroh::endpoint::RecvStream;
use iroh::endpoint::SendStream;
use iroh::protocol::ProtocolHandler;
use iroh::NodeAddr;
use iroh_blobs::Hash;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Emitter;
pub const ALPN: &[u8] = b"hermes/file-protocol/0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    id: String,
    name: String,
    hash: String,
    path: String,
    size: Option<u64>,
    modified: Option<DateTime<Utc>>,
    children: Option<Vec<TreeNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolRequestCommand {
    Ping,
    ListFileRequest { filter: Option<FileFilter> },
    Quit,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolResponseCommand {
    ListFileResponse { files: Vec<TreeNode> },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: Option<String>,
    /// (min_size, max_size)
    pub size_range: Option<(u64, u64)>,
    pub is_dir: Option<bool>,
    pub depth: Option<usize>,
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
                        let files = if let Some(filter) = filter {
                            let uploaded_files = this.get_files_tree(filter.depth).await?;
                            // TODO implement filtering logic
                            uploaded_files
                        } else {
                            this.get_files_tree(None).await?
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
            blobs_client,
            app_handle: app,
        }
    }

    pub async fn get_files_tree(&self, depth: Option<usize>) -> Result<Vec<TreeNode>> {
        let mut res = Vec::new();

        let mut tag_stream = self.blobs_client.tags().list().await?;
        // stream
        while let Some(tag) = tag_stream.next().await {
            let tag_info = tag?;
            let root_path = tag_info.name.to_string();
            let collection = self.blobs_client.get_collection(tag_info.hash).await?;

            for (name, hash) in collection.iter() {
                let path = PathBuf::from(&name);
                let mut node = &mut res;
                let len = path.iter().count();
                let mut cur_path = PathBuf::new();
                'inner: for (i, p) in path.iter().enumerate() {
                    cur_path.push(p);
                    let absolute_path =
                        root_path.clone() + "/" + cur_path.to_str().unwrap_or("unknown");
                    let id = Hash::new(&absolute_path);

                    let child_index = node.iter().position(|c: &TreeNode| c.id == id.to_string());

                    if let Some(idx) = child_index {
                        node = node[idx]
                            .children
                            .as_mut()
                            .expect("Node should have children");
                    } else {
                        let size = self.blobs_client.read(*hash).await?.size();
                        let children = if i == len - 1 { None } else { Some(Vec::new()) };
                        let new_node = TreeNode {
                            id: id.to_string(),
                            name: p.to_string_lossy().to_string(),
                            hash: hash.to_string(),
                            path: cur_path.to_string_lossy().to_string(),
                            size: Some(size),
                            children,
                            modified: None,
                        };
                        node.push(new_node);
                        if i == len - 1 {
                            break 'inner;
                        }
                        node = node
                            .last_mut()
                            .expect("Should exist see above line")
                            .children
                            .as_mut()
                            .expect("Node should have children");
                    }
                    if let Some(max_depth) = depth {
                        if i >= max_depth {
                            break 'inner;
                        }
                    }
                }
            }
        }
        Ok(res)
    }

    /// Import from a file or directory into the database.
    ///
    /// The returned tag always refers to a collection. If the input is a file, this
    /// is a collection with a single blob, named like the file.
    ///
    /// If the input is a directory, the collection contains all the files in the
    /// directory.
    pub async fn import(&self, path: impl AsRef<Path>) -> Result<(iroh_blobs::Tag, Hash)> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let _ = self.app_handle.emit("file-import-started", &path_str);
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
        let hash = *temp_tag.hash();
        batch.persist_to(temp_tag, tag.clone()).await?;
        drop(batch);
        let _ = self.app_handle.emit(
            &format!("file-imported::{}", path_str),
            path.to_string_lossy().to_string(),
        );
        println!("Imported {} with hash {}", path.display(), hash);
        Ok((tag, hash))
    }

    pub async fn clear_all_files(&mut self) -> Result<()> {
        self.blobs_client.tags().delete_all().await?;
        Ok(())
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
    ) -> Result<Vec<TreeNode>> {
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
