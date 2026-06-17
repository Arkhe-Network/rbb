use serde::{Serialize, Deserialize};

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub msg_type: MessageType,
    pub agent_id: String,
    pub payload: Option<String>,
    pub timestamp: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    AgentRegistered,
    TaskAssigned,
    TaskCompleted,
    ConsensusRequired,
}
