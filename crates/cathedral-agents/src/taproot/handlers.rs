use cathedral_taproot_bridge::TaprootClient;
use cathedral_wormgraph::Wormgraph;
use std::sync::Arc;

pub struct TaprootMcpIntegration {
    pub bridge: Arc<tokio::sync::Mutex<TaprootClient>>,
    pub wormgraph: Arc<Wormgraph>,
}

impl TaprootMcpIntegration {
    pub async fn new(
        tapd_addr: &str,
        macaroon_path: Option<&str>,
        wormgraph: Arc<Wormgraph>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client =
            TaprootClient::connect(tapd_addr, None, macaroon_path.map(std::path::Path::new))
                .await?;

        Ok(Self {
            bridge: Arc::new(tokio::sync::Mutex::new(client)),
            wormgraph,
        })
    }
}
