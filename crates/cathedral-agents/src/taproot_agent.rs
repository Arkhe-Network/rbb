use crate::Agent;
use async_trait::async_trait;
use cathedral_identity::Did;
use cathedral_permissions::AgentPermissions;
use cathedral_taproot_bridge::TaprootClient;
use cathedral_wormgraph::{ProvenanceEntry, Wormgraph};
use std::sync::Arc;

pub struct TaprootAgent {
    pub did: Did,
    pub client: Arc<tokio::sync::Mutex<TaprootClient>>,
    pub permissions: AgentPermissions,
    pub wormgraph: Arc<Wormgraph>,
}

impl TaprootAgent {
    pub fn new(
        did: Did,
        client: Arc<tokio::sync::Mutex<TaprootClient>>,
        wormgraph: Arc<Wormgraph>,
    ) -> Self {
        let permissions = AgentPermissions {
            agent_did: did.clone(),
            operations: vec![],
            signature: vec![],
        };
        Self {
            did,
            client,
            permissions,
            wormgraph,
        }
    }
}

#[async_trait]
impl Agent for TaprootAgent {
    fn did(&self) -> &Did {
        &self.did
    }

    fn permissions(&self) -> &AgentPermissions {
        &self.permissions
    }

    async fn run(&self, prompt: &str, session_id: &str) -> Result<serde_json::Value, String> {
        self.wormgraph
            .append(ProvenanceEntry {
                id: uuid::Uuid::new_v4().to_string(),
                version: 1,
                decision_type: "agent_started".to_string(),
                before_state: "".to_string(),
                after_state: prompt.to_string(),
                rationale: Some("TaprootAgent execution".to_string()),
                timestamp: chrono::Utc::now().timestamp(),
                agent_id: self.did.id.clone(),
                entry_hash: vec![],
                nostr_event_id: None,
                tree_id: Some(session_id.to_string()),
                agent_identity: Some(self.did.id.clone()),
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({ "status": "executed taproot operations" }))
    }
}
