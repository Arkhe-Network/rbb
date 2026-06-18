use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
// use polyglot_parser::PolyglotParser;
use wormgraph::WormGraph;
use crate::governance::Governance;
use crate::fastbrain_client::FastBrainClient;
use polyglot_parser::analysis::vulnerability::Finding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub code_changes: Option<String>,
    pub architecture_changes: Option<String>,
    pub model_updates: Option<String>,
    pub proposed_by: String,
}

pub struct SecondSelfOrchestrator {
    // pub polyglot: Arc<PolyglotParser>,
    pub governance: Arc<Governance>,
    pub wormgraph: Arc<WormGraph>,
}

impl SecondSelfOrchestrator {
    pub async fn new() -> Result<Self> {
        // let polyglot = Arc::new(PolyglotParser::new());
        let wormgraph = Arc::new(WormGraph::new(
            "https://turbo-gateway.com",
            "https://arweave.net"
        ));
        let governance = Arc::new(Governance::new(
            "https://mainnet.infura.io/v3/your_project_id",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        ).await?);
        Ok(Self { governance, wormgraph })
    }

    pub async fn handle_vulnerability(&self, finding: &Finding, code: &str) -> Result<()> {
        let finding_val = serde_json::to_value(finding)?;
        let tx_id = self.wormgraph.store_vulnerability(&finding_val).await?;
        info!("Vulnerability stored in WormGraph: {}", tx_id);

        let fastbrain = FastBrainClient::new("http://localhost:8080");
        let fixed_code = fastbrain.generate_fix(
            &finding_val,
            code,
            &format!("{:?}", finding.language).to_lowercase()
        ).await?;

        let proposal = EvolutionProposal {
            id: Uuid::new_v4().to_string(),
            title: format!("Fix for {}", finding.title),
            description: finding.description.clone(),
            code_changes: Some(fixed_code),
            architecture_changes: None,
            model_updates: None,
            proposed_by: "security_agent".to_string(),
        };

        let proposal_id = self.governance.propose_evolution(
            proposal.title.clone(),
            proposal.description.clone(),
            proposal.code_changes.clone().unwrap_or_default().into_bytes()
        ).await?;
        info!("Proposal submitted to onchain governance: {:?}", proposal_id);

        let proposal_val = serde_json::to_value(&proposal)?;
        self.wormgraph.store_proposal(&proposal_val).await?;

        Ok(())
    }
}
