//! Ações administrativas do NEXUS mapeadas para AdministrativeAction.
//!
//! Esta enum mapeia as ações específicas do NEXUS para os tipos
//! genéricos de AdministrativeAction do arkhe-governance.

use arkhe_governance::{ActionClass, GovernanceAction};
use chrono::Duration;

/// Ações administrativas específicas do NEXUS/Safe Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexusAdminAction {
    /// Atualização do kernel do NEXUS.
    KernelUpdate,
    /// Modificação de políticas de segurança.
    SecurityPolicyChange,
    /// Alteração em Capsules (privilégios, isolamento).
    CapsuleModification,
    /// Atualização de regras de ComplianceReport.
    ComplianceRulesUpdate,
    /// Modificação de parâmetros Flock.
    FlockConfigUpdate,
    /// Operação no WormGraph (routing, anchoring).
    WormGraphOperation,
    /// Alteração no Hashtree de bundles.
    BundleHashtreeUpdate,
    /// Outra ação administrativa.
    Other,
}

impl NexusAdminAction {
    /// Mapeia para ActionClass genérico.
    pub fn to_generic(&self) -> ActionClass {
        match self {
            Self::KernelUpdate => ActionClass::Critical,
            Self::SecurityPolicyChange => ActionClass::Critical,
            Self::CapsuleModification => ActionClass::Operational,
            Self::ComplianceRulesUpdate => ActionClass::Operational,
            Self::FlockConfigUpdate => ActionClass::Critical,
            Self::WormGraphOperation => ActionClass::Operational,
            Self::BundleHashtreeUpdate => ActionClass::Operational,
            Self::Other => ActionClass::Other,
        }
    }

    /// Cria proposta de governança para esta ação.
    pub fn to_proposal(
        &self,
        _id: String,
        description: String,
        _total_voters: u64,
        delay_hours: i64,
    ) -> GovernanceAction {
        GovernanceAction::new(
            self.to_generic(),
            description,
            "did:arkhe:system".to_string(),
            std::time::Duration::from_secs((delay_hours * 3600) as u64),
            [0u8; 32],
        )
    }
}
