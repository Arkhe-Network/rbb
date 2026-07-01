use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum InvariantError {
    #[error("Invariant violation: {0}")]
    Violation(String),
}

#[derive(Debug, Default)]
pub struct InvariantEngine {}

impl InvariantEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate_goal(&self, _context_json: &str) -> Result<(), InvariantError> {
        Ok(())
    }
}
