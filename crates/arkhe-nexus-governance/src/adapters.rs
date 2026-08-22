//! Adaptadores para integrar GovernanceGuard no NEXUS/Safe Core.
use arkhe_governance::{
    ActionClass, ExecutedAction, ExecutionResult, GovernanceAction, GovernanceGuard, GuardError,
};
use chrono::Duration;

/// Adaptador principal — envolve o NEXUS/Safe Core com governança.
///
/// Uso: Substituir o SafeCoreGuard existente por esta struct.
pub struct NexusGovernanceAdapter {
    guard: std::sync::Arc<GovernanceGuard>,
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
        proposal: GovernanceAction,
        action: F,
    ) -> Result<ExecutionResult, NexusGovernanceError>
    where
        F: FnOnce(&GovernanceAction) -> Result<(), String>,
    {
        // Passo 1: Submit (verifica I_gov no momento da submissão)
        let id_str = self
            .guard
            .submit(proposal.clone())
            .map_err(NexusGovernanceError::GovernanceViolation)?;

        // Passo 2: Execute (re-verifica I_gov + timelock no momento da execução)
        let _ = self
            .guard
            .execute(&id_str, action)
            .map_err(NexusGovernanceError::GovernanceError)?;

        Ok(ExecutionResult::Success)
    }

    /// Cancela ação administrativa pendente.
    ///
    /// Requer proposta de cancelamento que também satisfaça I_gov.
    pub fn cancel_admin_action(
        &self,
        proposal_id: &str,
        cancellation_proposal: &GovernanceAction,
    ) -> Result<(), NexusGovernanceError> {
        self.guard
            .cancel(proposal_id, cancellation_proposal)
            .map_err(NexusGovernanceError::GovernanceError)
    }

    /// Lista ações pendentes.
    pub fn pending_actions(&self) -> Vec<GovernanceAction> {
        Vec::new() // Not fully implemented in mocked guard
    }

    /// Lista ações executadas.
    pub fn executed_actions(&self) -> Vec<ExecutedAction> {
        Vec::new() // Not fully implemented in mocked guard
    }

    /// Hash do audit trail (para anchoring no WormGraph).
    pub fn audit_hash(&self) -> [u8; 32] {
        [0u8; 32] // Not fully implemented in mocked guard
    }

    /// Verifica se uma ação específica está no audit trail.
    pub fn is_action_audited(&self, proposal_id: &str) -> bool {
        false // Not fully implemented in mocked guard
    }
}

/// Erros da bridge NEXUS ↔ Governance.
#[derive(Debug, thiserror::Error)]
pub enum NexusGovernanceError {
    #[error("Governance invariant violated: {0}")]
    GovernanceViolation(#[from] GuardError),

    #[error("Governance operation failed: {0}")]
    GovernanceError(GuardError),

    #[error("NEXUS action failed: {0}")]
    NexusActionFailed(String),

    #[error("Action not found in audit trail: {0}")]
    ActionNotAudited(String),
}
