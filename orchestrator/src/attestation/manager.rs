use serde::{Deserialize, Serialize};
use crate::identity_attestation::IdentityAttestation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDescriptor {
    pub name: String,
    pub description: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAttestation {
    pub policy_compliance: bool,
    pub policy_attestation_id: String,
}

impl ExecutionAttestation {
    pub fn is_policy_compliant(&self) -> bool {
        self.policy_compliance
    }
    pub fn policy_attestation_id(&self) -> String {
        self.policy_attestation_id.clone()
    }
}

pub struct AttestationManager;

impl AttestationManager {
    pub fn new() -> Self {
        AttestationManager
    }

    pub async fn list_active_policies(&self) -> Result<Vec<PolicyDescriptor>, String> {
        // Em produção: consultar o GeometricPolicyEngine.
        // Por enquanto, retorna políticas padrão.
        Ok(vec![
            PolicyDescriptor {
                name: "pii_prohibition".to_string(),
                description: "Proíbe a saída de PII em respostas".to_string(),
                blocking: true,
            },
            PolicyDescriptor {
                name: "steering_safety".to_string(),
                description: "Garante que steering vectors não afetem segurança".to_string(),
                blocking: true,
            },
            PolicyDescriptor {
                name: "no_representation_collapse".to_string(),
                description: "Evita colapso de conceitos em embeddings".to_string(),
                blocking: false,
            },
        ])
    }

    pub async fn get_execution(&self, _id: &str) -> Result<Option<ExecutionAttestation>, String> {
        Ok(Some(ExecutionAttestation {
            policy_compliance: true,
            policy_attestation_id: "dummy-id".to_string(),
        }))
    }

    pub async fn validate_execution(&self, _exec: &ExecutionAttestation) -> Result<bool, String> {
        Ok(true)
    }

    pub async fn store_execution(&self, _exec: &ExecutionAttestation, _provenance: &str) -> Result<(), String> {
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait AttestationProvider {
    async fn run_authorized(&self, workload: &str, cost_cap: Option<f64>, identity: &IdentityAttestation) -> Result<ExecutionAttestation, String>;
}

pub trait AttestationVerifier {
    fn verify(&self, signature: &str) -> Result<bool, String>;
}

pub struct CathedralComputeProvider;

impl CathedralComputeProvider {
    pub fn new() -> Self {
        CathedralComputeProvider
    }
}

#[async_trait::async_trait]
impl AttestationProvider for CathedralComputeProvider {
    async fn run_authorized(&self, _workload: &str, _cost_cap: Option<f64>, _identity: &IdentityAttestation) -> Result<ExecutionAttestation, String> {
        Ok(ExecutionAttestation {
            policy_compliance: true,
            policy_attestation_id: "dummy-id".to_string(),
        })
    }
}
