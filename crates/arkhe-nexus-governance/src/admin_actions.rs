//! Ações administrativas do NEXUS mapeadas para AdministrativeAction.
//!
//! Esta enum mapeia as ações específicas do NEXUS para os tipos
//! genéricos de AdministrativeAction do arkhe-governance.
use arkhe_governance::AdministrativeAction;
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
    /// Mapeia para AdministrativeAction genérico.
    pub fn to_generic(&self) -> AdministrativeAction {
        match self {
            Self::KernelUpdate => AdministrativeAction::SafeCoreUpdate,
            Self::SecurityPolicyChange => AdministrativeAction::SafeCoreUpdate,
            Self::CapsuleModification => AdministrativeAction::CapsulePrivilegeChange,
            Self::ComplianceRulesUpdate => AdministrativeAction::ComplianceRuleChange,
            Self::FlockConfigUpdate => AdministrativeAction::FlockParameterChange,
            Self::WormGraphOperation => AdministrativeAction::WormGraphOperation,
            Self::BundleHashtreeUpdate => AdministrativeAction::BundleHashtreeChange,
            Self::Other => AdministrativeAction::Other,
        }
    }
    /// Cria proposta de governança para esta ação.
    pub fn to_proposal(
        &self,
        id: String,
        description: String,
        total_voters: u64,
        delay_hours: i64,
    ) -> arkhe_governance::GovernanceProposal {
        arkhe_governance::GovernanceProposal::new(
            id,
            description,
            self.to_generic(),
            total_voters,
            chrono::Duration::hours(delay_hours),
        )
    }
}
