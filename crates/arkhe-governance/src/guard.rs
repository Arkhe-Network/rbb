use std::sync::Mutex;
use crate::invariants::{GovernanceProposal, GovernanceInvariantChecker, AdministrativeAction, ExecutedProposal, GovernanceViolation};

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ExecutionResult {
    Success,
    Rejected(String),
    Cancelled,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum GuardError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Cancellation denied: {0}")]
    CancellationDenied(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub struct GovernanceGuard {
    checker: Mutex<GovernanceInvariantChecker>,
    pending: Mutex<Vec<GovernanceProposal>>,
    executed: Mutex<Vec<ExecutedProposal>>,
}

impl GovernanceGuard {
    pub fn new() -> Self {
        Self {
            checker: Mutex::new(GovernanceInvariantChecker::default()),
            pending: Mutex::new(Vec::new()),
            executed: Mutex::new(Vec::new()),
        }
    }

    pub fn submit(&self, proposal: GovernanceProposal) -> Result<(), GovernanceViolation> {
        let mut pending = self.pending.lock().unwrap();
        pending.push(proposal);
        Ok(())
    }

    pub fn execute<F>(&self, proposal_id: &str, action: F) -> Result<ExecutionResult, GuardError>
    where
        F: FnOnce(&GovernanceProposal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        let proposal = {
            let pending = self.pending.lock().unwrap();
            pending.iter().find(|p| p.id == proposal_id).cloned()
        };

        if let Some(prop) = proposal {
            let action_result = action(&prop);
            let success = action_result.is_ok();
            let execution_result = if success {
                ExecutionResult::Success
            } else {
                ExecutionResult::Rejected(action_result.as_ref().unwrap_err().to_string())
            };
            self.executed.lock().unwrap().push(ExecutedProposal {
                proposal: prop,
                result: execution_result.clone()
            });
            action_result.map_err(|e| GuardError::ExecutionFailed(e.to_string()))?;
            return Ok(execution_result);
        }
        Err(GuardError::NotFound(proposal_id.to_string()))
    }

    pub fn cancel(&self, _proposal_id: &str, _cancellation: &GovernanceProposal) -> Result<(), GuardError> {
        Ok(())
    }

    pub fn pending_proposals(&self) -> Vec<GovernanceProposal> {
        self.pending.lock().unwrap().clone()
    }

    pub fn executed_proposals(&self) -> Vec<ExecutedProposal> {
        self.executed.lock().unwrap().clone()
    }

    pub fn audit_hash(&self) -> [u8; 32] {
        [0u8; 32]
    }
}
