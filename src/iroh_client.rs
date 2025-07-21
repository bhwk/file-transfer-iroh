use anyhow::anyhow;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{BlobsProtocol, store::mem::MemStore, ticket::BlobTicket};
use std::{fmt::Display, path::PathBuf, str::FromStr};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct BlobData {
    ticket: BlobTicket,
    encoded_name: String,
}

impl BlobData {
    pub fn new(ticket: BlobTicket, name: String) -> Self {
        let encoded_name = URL_SAFE_NO_PAD.encode(name);

        BlobData {
            ticket,
            encoded_name,
        }
    }

    pub fn decode_name(&self) -> anyhow::Result<String> {
        let decoded_name = URL_SAFE_NO_PAD
            .decode(&self.encoded_name)?
            .iter()
            .map(|&b| b as char)
            .collect::<String>();

        Ok(decoded_name)
    }
}

impl FromStr for BlobData {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // this should always be 2 parts
        let parts: Vec<String> = s.split("!").map(|s| s.to_string()).collect();
        let ticket: BlobTicket = parts[0].parse()?;
        let encoded_name = parts[1].clone();

        Ok(Self {
            ticket,
            encoded_name,
        })
    }
}

impl Display for BlobData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}!{}", self.ticket, self.encoded_name)
    }
}

pub async fn send_file(path: PathBuf, cancel_clone: CancellationToken) -> anyhow::Result<String> {
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    // create store and blob protocol
    let store = MemStore::new();
    let blobs = BlobsProtocol::new(&store, endpoint.clone(), None);

    // for ticket information
    let abs_path = std::path::absolute(&path)?;
    let tag = blobs.store().add_path(abs_path).await?;

    let node_id = endpoint.node_id();
    let ticket = BlobTicket::new(node_id.into(), tag.hash, tag.format);

    let filename = path
        .file_name()
        .and_then(|osstr| osstr.to_str())
        .ok_or_else(|| anyhow!("Filename is missing or not valid UTF-8"))?
        .to_string();

    let blob_data = BlobData::new(ticket, filename);

    info!("Spawning router");
    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs)
        .spawn();

    #[allow(clippy::never_loop)]
    let _router_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        router.shutdown().await.unwrap();
                        info!("Router shutdown");
                        break;
                    }
                    _ = futures::future::pending::<()>() => {
                        unreachable!("futures::future::pending() should never resolve");
                    }

            }
        }
    });

    Ok(blob_data.to_string())
}

pub async fn receive_file(dir_path: PathBuf, input_ticket: &str) -> anyhow::Result<()> {
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;
    // create store and blob protocol
    let store = MemStore::new();

    let blob_data = BlobData::from_str(input_ticket)?;

    let file_name = blob_data.decode_name()?;
    // create new path, concat the filename to path
    let path = dir_path.join(file_name);

    // lets just make sure we don't save directories.
    if path.is_dir() {
        return Err(anyhow!("Writing to a directory!"));
    }

    let abs_path = std::path::absolute(path)?;
    let ticket: BlobTicket = blob_data.ticket;

    let downloader = store.downloader(&endpoint);
    downloader
        .download(ticket.hash(), Some(ticket.node_addr().node_id))
        .await?;
    store.blobs().export(ticket.hash(), abs_path).await?;
    endpoint.close().await;

    Ok(())
}
