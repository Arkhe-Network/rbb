//! Cathedral ARKHE v28.3 — Consensus Ledger
//! Registro imutável de decisões multi-agente com SPHINCS+ e ancoragem na TemporalChain.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-CONSENSUS-LEDGER-2026-06-16

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

use serde::{Serialize, Deserialize};

#[cfg(feature = "std")]
use std::sync::Mutex;

use crate::orchestrator::{AgentId};
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRecord {
    pub record_id: String,
    pub workflow_id: String,
    pub timestamp: u64,
    pub participating_agents: Vec<String>,
    pub decision: String,
    // Note: Option<String> is used to replace serde_json::Value for no_std
    pub outcome: Option<String>,
    pub votes: Vec<(String, bool)>,           // agent_id -> approved
    pub temporal_chain_hash: Option<String>,            // hash da TemporalChain
    pub on_chain_tx_hash: Option<String>,       // hash da transação on-chain (futuro)
    pub sphincs_signature: String,
}

#[cfg(feature = "std")]
pub struct CathedralConsensusLedger {
    records: Mutex<HashMap<String, ConsensusRecord>>,
}

#[cfg(feature = "std")]
impl CathedralConsensusLedger {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
        }
    }

    /// Registra uma decisão de consenso (chamado após votação ou aprovação do Guardian)
    pub async fn record_decision(
        &self,
        workflow_id: &str,
        decision: &str,
        agents: Vec<AgentId>,
        outcome: serde_json::Value,
        votes: Vec<(String, bool)>,
    ) -> Result<ConsensusRecord, String> {
        let record_id = uuid::Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut record = ConsensusRecord {
            record_id: record_id.clone(),
            workflow_id: workflow_id.to_string(),
            timestamp,
            participating_agents: agents.iter().map(|a| a.0.clone()).collect(),
            decision: decision.to_string(),
            outcome: Some(outcome.to_string()),
            votes,
            temporal_chain_hash: None,
            on_chain_tx_hash: None,
            sphincs_signature: String::new(),
        };

        // Assinatura SPHINCS+ (normalmente feita pelo Guardian ou orquestrador)
        // Stub implementation
        record.sphincs_signature = "stub_signature".to_string();

        self.records.lock().unwrap().insert(record_id.clone(), record.clone());
        Ok(record)
    }

    pub async fn get_record(&self, record_id: &str) -> Option<ConsensusRecord> {
        self.records.lock().unwrap().get(record_id).cloned()
    }

    pub async fn get_by_workflow(&self, workflow_id: &str) -> Vec<ConsensusRecord> {
        self.records.lock().unwrap()
            .values()
            .filter(|r| r.workflow_id == workflow_id)
            .cloned()
            .collect()
    }
}
