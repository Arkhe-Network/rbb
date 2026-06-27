pub trait SafeCoreHook: Send + Sync {
    fn pre_submit(&self, action: &crate::invariants::GovernanceProposal) -> Result<(), HookError>;
    fn pre_execute(&self, action: &crate::invariants::GovernanceProposal) -> Result<(), HookError>;
    fn post_execute(&self, action: &crate::invariants::GovernanceProposal, success: bool);
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum HookError {
    #[error("Blocked: {0}")]
    Blocked(String),
    #[error("Internal: {0}")]
    Internal(String),
}
