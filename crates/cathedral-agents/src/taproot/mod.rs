pub mod handlers;

use crate::taproot::handlers::TaprootMcpIntegration;
use cathedral_taproot_bridge::TaprootClient;
use cathedral_wormgraph::Wormgraph;
use std::sync::Arc;

pub async fn setup_taproot_integration(
    tapd_addr: &str,
    macaroon_path: Option<&str>,
    wormgraph: Arc<Wormgraph>,
) -> Result<TaprootMcpIntegration, Box<dyn std::error::Error>> {
    let client =
        TaprootClient::connect(tapd_addr, None, macaroon_path.map(std::path::Path::new)).await?;

    Ok(TaprootMcpIntegration {
        bridge: Arc::new(tokio::sync::Mutex::new(client)),
        wormgraph,
    })
}
