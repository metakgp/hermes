pub mod discovery;

use anyhow::Result;
use iroh::Endpoint;
use tracing::instrument;

#[instrument(ret, err)]
pub async fn setup_iroh() -> Result<Endpoint> {
    let endpoint = Endpoint::builder().discovery_local_network().bind().await?;
    Ok(endpoint)
}
