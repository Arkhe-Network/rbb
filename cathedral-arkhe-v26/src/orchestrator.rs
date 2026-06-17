extern crate alloc;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Oracle,      // General reasoning, coordination
    Coder,       // Code generation, execution
    Analyst,     // Data analysis, research
    Guardian,    // Security, policy enforcement (override authority)
    Executor,    // Tool execution (subordinate)
    Observer,    // Monitoring, telemetry (read-only)
}

impl AgentRole {
    /// Authority level — higher = more override power.
    pub fn authority_level(&self) -> u8 {
        match self {
            AgentRole::Guardian => 255,  // Can halt any operation
            AgentRole::Oracle => 200,    // Can coordinate and delegate
            AgentRole::Analyst => 150,   // Can request data, not execute
            AgentRole::Coder => 150,     // Can write code, not deploy
            AgentRole::Executor => 100,  // Executes only
            AgentRole::Observer => 50,   // Read-only
        }
    }
}

/// Unique agent identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusMode {
    MajorityVote,      // Simple majority
    WeightedVote,      // Weighted by performance_score
    Unanimous,         // All must agree
    Hierarchical,      // Leader decides after hearing all
    Delphi,            // Anonymous rounds until convergence
    KPop,              // Knowledge-Pop consensus
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub position: String, // e.g., "approve", "reject", "abstain"
    pub reasoning: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusResult {
    Reached { agreement_ratio: f32, winning_position: String },
    Failed { reason: String },
    Overridden { by: AgentId, reason: String },
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRecord {
    pub coalition_id: String,
    pub timestamp: u64,
    pub topic: String,
    pub votes: Vec<(AgentId, Vote)>, // Changed from HashMap to Vec of tuples to support no_std serialization
    pub result: ConsensusResult,
    pub confidence: f32,
}
