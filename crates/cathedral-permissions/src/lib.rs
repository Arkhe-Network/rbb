use serde::{Deserialize, Serialize};
use cathedral_identity::Did;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    Allowed,      // Executa automaticamente
    Restricted,   // Requer confirmação do usuário
    Denied,       // Proibido
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub agent_did: Did,
    pub operations: Vec<PermissionEntry>,
    pub signature: Vec<u8>,      // Assinatura ML-DSA do agente
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub operation: String,       // Ex: "read", "write", "bash", "git"
    pub level: PermissionLevel,
    pub scope: Option<String>,   // Ex: "*.rs", "/etc/**"
    pub justification: String,   // Por que esta permissão foi concedida
}

impl AgentPermissions {
    pub fn verify(&self) -> Result<bool, String> {
        // Verifica a assinatura ML-DSA
        cathedral_identity::verify_signature(
            &self.agent_did,
            &self.signature,
            &serde_json::to_vec(self).map_err(|e| e.to_string())?,
        ).map_err(|_| "Signature verification failed".to_string())
    }
}
