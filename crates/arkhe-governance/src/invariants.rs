use chrono::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: String,
    pub description: String,
    pub class: AdministrativeAction,
    pub total_voters: u64,
    pub delay: std::time::Duration,
}

impl GovernanceProposal {
    pub fn new(id: String, description: String, class: AdministrativeAction, total_voters: u64, delay: Duration) -> Self {
        Self {
            id,
            description,
            class,
            total_voters,
            delay: delay.to_std().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedProposal {
    pub proposal: GovernanceProposal,
    pub result: crate::guard::ExecutionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdministrativeAction {
    SafeCoreUpdate,
    CapsulePrivilegeChange,
    ComplianceRuleChange,
    FlockParameterChange,
    WormGraphOperation,
    BundleHashtreeChange,
    Other,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum GovernanceViolation {
    #[error("Generic violation: {0}")]
    Generic(String),
}

#[derive(Default)]
pub struct GovernanceInvariantChecker;
