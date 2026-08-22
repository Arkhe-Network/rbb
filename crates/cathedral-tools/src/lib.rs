use async_trait::async_trait;
use cathedral_identity::Did;
use cathedral_permissions::PermissionEntry;
use cathedral_wormgraph::{ProvenanceEntry, Wormgraph};
use std::sync::Arc;

pub type ToolParams = serde_json::Value;
pub type ToolResult = serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn permissions(&self) -> Vec<PermissionEntry>;
    async fn execute(
        &self,
        params: &ToolParams,
        context: &ToolContext,
    ) -> Result<ToolResult, String>;
}

pub struct ToolContext {
    pub agent_did: Did,
    pub session_id: String,
    pub wormgraph: Arc<Wormgraph>,
}

impl ToolContext {
    pub async fn record_action(
        &self,
        action: &str,
        params: &ToolParams,
        result: &ToolResult,
    ) -> Result<(), String> {
        self.wormgraph
            .append(ProvenanceEntry {
                id: uuid::Uuid::new_v4().to_string(),
                version: 1,
                decision_type: "tool_execution".to_string(),
                before_state: serde_json::to_string(params).unwrap_or_default(),
                after_state: serde_json::to_string(result).unwrap_or_default(),
                rationale: Some(action.to_string()),
                timestamp: chrono::Utc::now().timestamp(),
                agent_id: self.agent_did.id.clone(),
                entry_hash: vec![],
                nostr_event_id: None,
                tree_id: Some(self.session_id.clone()),
                agent_identity: Some(self.agent_did.id.clone()),
            })
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
