//! Adaptadores para integrar GovernanceGuard no NEXUS/Safe Core.
use arkhe_governance::{
    AdministrativeAction, GovernanceGuard, GovernanceProposal, GovernanceError,
    ExecutionResult, GovernanceViolation,
};
use chrono::Duration;
/// Adaptador principal — envolve o NEXUS/Safe Core com governança.
///
/// Uso: Substituir o SafeCoreGuard existente por esta struct.
pub struct NexusGovernanceAdapter {
    guard: std::sync::Arc<GovernanceGuard>
}
impl NexusGovernanceAdapter {
    /// Cria adaptador com GovernanceGuard padrão (5/8, 48h).
    pub fn new() -> Self {
        Self {
            guard: std::sync::Arc::new(GovernanceGuard::new()),
        }
    }

    /// Cria adaptador com GovernanceGuard customizado.
    pub fn with_guard(guard: std::sync::Arc<GovernanceGuard>) -> Self {
        Self { guard }
    }

    /// Executa ação administrativa com verificação de I_gov.
    ///
    /// Esta é a função principal de substituição para o NEXUS.
    /// Deve ser chamada em vez de executar ação diretamente.
    pub fn execute_admin_action<F>(
        &self,
        proposal: GovernanceProposal,
        action: F,
    ) -> Result<ExecutionResult, NexusGovernanceError>
    where
        F: FnOnce(&GovernanceProposal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        // Passo 1: Submit (verifica I_gov no momento da submissão)
        self.guard.submit(proposal.clone())
            .map_err(NexusGovernanceError::GovernanceViolation)?;

        // Passo 2: Execute (re-verifica I_gov + timelock no momento da execução)
        self.guard.execute(&proposal.id, action)
            .map_err(NexusGovernanceError::GovernanceError)
    }

    /// Cancela ação administrativa pendente.
    ///
    /// Requer proposta de cancelamento que também satisfaça I_gov.
    pub fn cancel_admin_action(
        &self,
        proposal_id: &str,
        cancellation_proposal: &GovernanceProposal,
    ) -> Result<(), NexusGovernanceError> {
        self.guard.cancel(proposal_id, cancellation_proposal)
            .map_err(NexusGovernanceError::GovernanceError)
    }

    /// Lista ações pendentes.
    pub fn pending_actions(&self) -> Vec<GovernanceProposal> {
        self.guard.pending_proposals()
    }

    /// Lista ações executadas.
    pub fn executed_actions(&self) -> Vec<arkhe_governance::ExecutedProposal> {
        self.guard.executed_proposals()
    }

    /// Hash do audit trail (para anchoring no WormGraph).
    pub fn audit_hash(&self) -> [u8; 32] {
        self.guard.audit_hash()
    }

    /// Verifica se uma ação específica está no audit trail.
    pub fn is_action_audited(&self, proposal_id: &str) -> bool {
        self.guard.executed_proposals()
            .iter()
            .any(|ep| ep.proposal.id == proposal_id)
    }
}
/// Erros da bridge NEXUS ↔ Governance.
#[derive(Debug, thiserror::Error)]
pub enum NexusGovernanceError {
    #[error("Governance invariant violated: {0}")]
    GovernanceViolation(#[from] GovernanceViolation),
    #[error("Governance operation failed: {0}")]
    GovernanceError(#[from] GovernanceError),

    #[error("NEXUS action failed: {0}")]
    NexusActionFailed(String),

    #[error("Action not found in audit trail: {0}")]
    ActionNotAudited(String),
}
