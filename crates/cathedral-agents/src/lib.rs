use async_trait::async_trait;
use cathedral_identity::Did;
use cathedral_permissions::{AgentPermissions, PermissionEntry, PermissionLevel};
use cathedral_tools::ToolContext;
use cathedral_wormgraph::{ProvenanceEntry, Wormgraph};
use std::sync::Arc;

#[async_trait]
pub trait Agent: Send + Sync {
    fn did(&self) -> &Did;
    fn permissions(&self) -> &AgentPermissions;
    async fn run(&self, prompt: &str, session_id: &str) -> Result<serde_json::Value, String>;
}

pub struct BuildAgent {
    pub did: Did,
    pub permissions: AgentPermissions,
    pub wormgraph: Arc<Wormgraph>,
    // In a real implementation this would hold an instance to the ZK Gateway and ML-DSA keys
}

impl BuildAgent {
    pub fn new(did: Did, wormgraph: Arc<Wormgraph>) -> Self {
        let permissions = AgentPermissions {
            agent_did: did.clone(),
            operations: vec![
                PermissionEntry {
                    operation: "read".to_string(),
                    level: PermissionLevel::Allowed,
                    scope: Some("**".to_string()),
                    justification: "Leitura de arquivos necessária para análise".to_string(),
                },
                PermissionEntry {
                    operation: "write".to_string(),
                    level: PermissionLevel::Restricted,
                    scope: Some("src/**".to_string()),
                    justification: "Edição de código fonte requer confirmação".to_string(),
                },
            ],
            signature: vec![], // Signed in real system
        };
        Self {
            did,
            permissions,
            wormgraph,
        }
    }
}

#[async_trait]
impl Agent for BuildAgent {
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
                rationale: Some("BuildAgent starting task".to_string()),
                timestamp: chrono::Utc::now().timestamp(),
                agent_id: self.did.id.clone(),
                entry_hash: vec![],
                nostr_event_id: None,
                tree_id: Some(session_id.to_string()),
                agent_identity: Some(self.did.id.clone()),
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({ "status": "completed" }))
    }
}

pub struct PlanAgent {
    pub did: Did,
    pub permissions: AgentPermissions,
    pub wormgraph: Arc<Wormgraph>,
}

impl PlanAgent {
    pub fn new(did: Did, wormgraph: Arc<Wormgraph>) -> Self {
        let permissions = AgentPermissions {
            agent_did: did.clone(),
            operations: vec![PermissionEntry {
                operation: "read".to_string(),
                level: PermissionLevel::Allowed,
                scope: Some("**".to_string()),
                justification: "Exploration".to_string(),
            }],
            signature: vec![],
        };
        Self {
            did,
            permissions,
            wormgraph,
        }
    }
}

#[async_trait]
impl Agent for PlanAgent {
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
                rationale: Some("PlanAgent planning".to_string()),
                timestamp: chrono::Utc::now().timestamp(),
                agent_id: self.did.id.clone(),
                entry_hash: vec![],
                nostr_event_id: None,
                tree_id: Some(session_id.to_string()),
                agent_identity: Some(self.did.id.clone()),
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({ "status": "planned", "plan": [] }))
    }
}
pub mod rfq;
pub mod taproot;
pub mod taproot_agent;
