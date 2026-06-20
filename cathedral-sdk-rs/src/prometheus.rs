use crate::event::{PrometheusEvent, EventType, EventMetadata};
use tokio::sync::mpsc;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PrometheusAdapter {
    tx: mpsc::Sender<PrometheusEvent>,
    project_id: String,
    agent_id: String,
}

impl PrometheusAdapter {
    pub fn new(tx: mpsc::Sender<PrometheusEvent>, project_id: String, agent_id: String) -> Self {
        Self { tx, project_id, agent_id }
    }

    pub async fn on_design_proposed(
        &self,
        design_hash: String,
        parent_hashes: Vec<String>,
        metadata: EventMetadata,
    ) -> anyhow::Result<()> {
        let event = PrometheusEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64,
            event_type: EventType::DesignProposed,
            project_id: self.project_id.clone(),
            agent_id: self.agent_id.clone(),
            design_hash,
            parent_hashes,
            metadata,
            payload: serde_json::json!({}),
        };
        self.tx.send(event).await?;
        Ok(())
    }

    pub async fn on_simulation_completed(
        &self,
        design_hash: String,
        metrics: serde_json::Value,
    ) -> anyhow::Result<()> {
        let event = PrometheusEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64,
            event_type: EventType::SimulationCompleted,
            project_id: self.project_id.clone(),
            agent_id: self.agent_id.clone(),
            design_hash,
            parent_hashes: vec![],
            metadata: EventMetadata {
                domain: "aerospace".to_string(),
                confidence: metrics.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5),
                compute_cost_usd: 150.0,
                tags: vec!["simulation".to_string()],
            },
            payload: metrics,
        };
        self.tx.send(event).await?;
        Ok(())
    }

    pub async fn on_agent_mutation(
        &self,
        mutation_description: String,
        previous_hash: String,
    ) -> anyhow::Result<()> {
        let event = PrometheusEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64,
            event_type: EventType::AgentMutation,
            project_id: self.project_id.clone(),
            agent_id: self.agent_id.clone(),
            design_hash: blake3::hash(mutation_description.as_bytes()).to_hex().to_string(),
            parent_hashes: vec![previous_hash],
            metadata: EventMetadata {
                domain: "meta".to_string(),
                confidence: 0.7,
                compute_cost_usd: 0.0,
                tags: vec!["recursive_engineering".to_string()],
            },
            payload: serde_json::json!({ "mutation": mutation_description }),
        };
        self.tx.send(event).await?;
        Ok(())
    }
}
