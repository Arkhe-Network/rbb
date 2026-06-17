use std::sync::Arc;
use tracing::{info, error, instrument};
use serde_json::json;
use futures::future::join_all;

use crate::testing::test_agent::{TestAgent, TestResult, TestType};
use crate::testing::deps::{SubagentSpawner, AttestationManager, AttestationSigner, TrajectoryStore, ExecutionAttestation};
use crate::testing::test_attestation::{TestAttestation, TestAttestationExt};

/// Orquestrador que executa múltiplos agentes de teste e agrega resultados.
pub struct TestOrchestrator {
    pub spawner: Arc<SubagentSpawner>,
    pub attestation_manager: Arc<AttestationManager>,
    pub store: Arc<dyn TrajectoryStore + Send + Sync>,
    pub signer: Arc<dyn AttestationSigner + Send + Sync>,
    pub test_agents: Vec<Arc<dyn TestAgent>>,
}

impl TestOrchestrator {
    pub fn new(
        spawner: Arc<SubagentSpawner>,
        attestation_manager: Arc<AttestationManager>,
        store: Arc<dyn TrajectoryStore + Send + Sync>,
        signer: Arc<dyn AttestationSigner + Send + Sync>,
    ) -> Self {
        Self {
            spawner,
            attestation_manager,
            store,
            signer,
            test_agents: Vec::new(),
        }
    }

    pub async fn register_test_agent(&mut self, agent: Arc<dyn TestAgent>) {
        info!("📋 Agente de teste registado: {}", agent.test_name());
        self.test_agents.push(agent);
    }

    /// Gera relatório agregado com atestação como TestAttestation.
    pub async fn generate_report(&self, results: &[TestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let report = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_tests": total,
            "passed": passed,
            "failed": failed,
            "success_rate": if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            "results": results.iter().map(|r| json!({
                "name": r.test_name,
                "type": format!("{:?}", r.test_type),
                "passed": r.passed,
                "duration_ms": r.duration_ms,
                "details": r.details,
            })).collect::<Vec<_>>(),
        });

        let report_json = serde_json::to_string_pretty(&report).unwrap_or_default();
        info!("📊 Relatório de testes:\n{}", report_json);

        // Persistir relatório como TestAttestation
        let report_result = TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: "test_report".to_string(),
            test_type: TestType::Integration,
            passed: true,
            duration_ms: 0,
            details: report.clone(),
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        };

        // Converter para TestAttestation e persistir
        if let Err(e) = report_result.store_test_result_as_attestation(
            self.signer.as_ref(),
            self.store.as_ref(),
        ).await {
            error!("Falha ao persistir relatório como TestAttestation: {}", e);
        }
    }

    pub async fn stats(&self) -> serde_json::Value {
        let trajs = self.store.list_trajectories().await;
        let test_results: Vec<_> = trajs.iter()
            .filter(|t| t.goal.starts_with("test_result:"))
            .collect();

        json!({
            "total_test_results": test_results.len(),
            "registered_test_agents": self.test_agents.len(),
        })
    }
}
