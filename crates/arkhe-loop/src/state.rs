use serde::{Deserialize, Serialize};

pub type LoopResult = Result<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub max_iterations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopPhase {
    Reasoning,
    Action,
    Reflection,
    Execution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStep {
    pub phase: LoopPhase,
    pub iteration: u64,
    pub input: String,
    pub output: String,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub success: bool,
}

pub struct LoopState {
    _config: LoopConfig,
}

impl LoopState {
    pub fn new(_config: LoopConfig) -> Self {
        Self { _config }
    }

    pub fn check_limits(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn snapshot(&self) -> LoopSnapshot {
        LoopSnapshot {}
    }
}

pub struct LoopSnapshot {}
