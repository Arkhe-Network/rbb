use crate::guard::{GovernanceGuard, GuardError, ExecutionResult};
use crate::invariants::GovernanceProposal;

pub struct AsyncGovernanceGuard {
    inner: tokio::sync::Mutex<GovernanceGuard>,
}

impl AsyncGovernanceGuard {
    pub fn new() -> Self {
        Self {
            inner: tokio::sync::Mutex::new(GovernanceGuard::new()),
        }
    }

    pub async fn submit(&self, proposal: GovernanceProposal) -> Result<(), crate::invariants::GovernanceViolation> {
        let guard = self.inner.lock().await;
        guard.submit(proposal)
    }

    pub async fn execute<F>(&self, proposal_id: &str, action: F) -> Result<ExecutionResult, GuardError>
    where
        F: FnOnce(&GovernanceProposal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        let guard = self.inner.lock().await;
        guard.execute(proposal_id, action)
    }
}
