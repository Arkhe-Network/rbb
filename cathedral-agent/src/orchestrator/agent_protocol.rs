//! Cathedral ARKHE v28.3 — Agent Communication Protocol (ACP)//! Structured message passing between agents with provenance and attestation.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-ACP-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// ACP Message — all inter-agent communication uses this envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub header: AcpHeader,
    pub payload: AcpPayload,
    pub provenance: MessageProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpHeader {
    pub message_id: String,           // UUID v4
    pub correlation_id: String,       // Links request-response pairs
    pub sender: super::AgentId,
    pub recipient: AcpRecipient,
    pub message_type: AcpMessageType,
    pub priority: super::TaskPriority,
    pub timestamp: u64,
    pub ttl_seconds: u32,            // Time-to-live
    pub cathedral_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcpRecipient {
    Agent(super::AgentId),
    Coalition(String),               // Broadcast to coalition
    Role(super::AgentRole),          // Any agent with role
    All,                            // Broadcast all
    Orchestrator,                   // To orchestrator only
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcpMessageType {
    Request,        // Ask for action/information
    Response,       // Reply to request
    Notification,   // Fire-and-forget
    Proposal,       // Suggest action/plan
    Vote,           // Cast vote in consensus
    Challenge,      // Dispute a claim/action
    Attestation,    // Cryptographic proof
    Heartbeat,      // Liveness check
    Emergency,      // Emergency broadcast
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcpPayload {
    // Task delegation
    TaskRequest { task: super::DelegatedTask },
    TaskResponse { task_id: String, result: TaskResult, status: TaskStatus },
    TaskStatusUpdate { task_id: String, progress: f32, status: TaskStatus },

    // Consensus
    ConsensusProposal { coalition_id: String, topic: String, position: String, reasoning: String },
    ConsensusVote { coalition_id: String, position: String, confidence: f32 },
    ConsensusResult(super::ConsensusRecord),

    // Knowledge sharing
    KnowledgeShare { topic: String, content: serde_json::Value, confidence: f32 },
    KnowledgeQuery { topic: String, query: String },
    KnowledgeResponse { topic: String, results: Vec<KnowledgeEntry> },

    // Security
    AttestationRequest { nonce: [u8; 32] },
    AttestationResponse { nonce: [u8; 32], signature: Vec<u8>, public_key: [u8; 32] },
    PolicyUpdate { policy_hash: String, changes: Vec<String> },
    AuditLog { events: Vec<AuditEvent> },

    // Coordination
    Heartbeat { load: f32, memory_usage: f32, active_tasks: u32 },
    Emergency { level: EmergencyLevel, message: String, affected_systems: Vec<String> },
    StatusQuery,
    StatusResponse { agent_status: super::AgentStatus, performance: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub latency_ms: u64,
    pub tokens_used: Option<super::Usage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending, Running, Completed, Failed, Cancelled, Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub source: super::AgentId,
    pub content: serde_json::Value,
    pub confidence: f32,
    pub timestamp: u64,
    pub verification_status: VerificationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified, Verified, Disputed, Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub agent_id: super::AgentId,
    pub action: String,
    pub result: String,
    pub policy_compliant: bool,
    pub merkle_root: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyLevel {
    Info, Warning, Critical, Catastrophic,
}

/// Message provenance — cryptographic chain of custody.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProvenance {
    pub previous_message_id: Option<String>,
    pub origin_message_id: String,
    pub hop_count: u32,
    pub signatures: Vec<MessageSignature>,
    pub merkle_path: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSignature {
    pub agent_id: super::AgentId,
    pub signature: Vec<u8>,
    pub algorithm: String, // "SPHINCS+-SHA256-128s"
    pub timestamp: u64,
}

impl AcpMessage {
    /// Create a new ACP message.
    pub fn new(
        sender: super::AgentId,
        recipient: AcpRecipient,
        message_type: AcpMessageType,
        payload: AcpPayload,
        priority: super::TaskPriority,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let message_id = format!("msg_{}_{}", sender.0, timestamp);

        Self {
            header: AcpHeader {
                message_id: message_id.clone(),
                correlation_id: message_id,
                sender,
                recipient,
                message_type,
                priority,
                timestamp,
                ttl_seconds: 300,
                cathedral_version: "28.3".to_string(),
            },
            payload,
            provenance: MessageProvenance {
                previous_message_id: None,
                origin_message_id: message_id.clone(),
                hop_count: 0,
                signatures: Vec::new(),
                merkle_path: None,
            },
        }
    }

    /// Sign the message with agent's SPHINCS+ key.
    pub fn sign(&mut self, agent_id: &super::AgentId, secret_key: &[u8]) {
        let message_bytes = serde_json::to_vec(&self.payload).unwrap_or_default();
        // In production: use sphincsplus crate
        let signature = blake3::hash(&message_bytes).as_bytes().to_vec();

        self.provenance.signatures.push(MessageSignature {
            agent_id: agent_id.clone(),
            signature,
            algorithm: "BLAKE3-STUB".to_string(),
            timestamp: self.header.timestamp,
        });
    }

    /// Verify message signatures.
    pub fn verify(&self) -> bool {
        // In production: verify each SPHINCS+ signature
        !self.provenance.signatures.is_empty()
    }

    /// Forward message to another agent (increments hop count).
    pub fn forward(mut self, new_recipient: AcpRecipient) -> Self {
        self.provenance.previous_message_id = Some(self.header.message_id.clone());
        self.provenance.hop_count += 1;
        self.header.recipient = new_recipient;
        self
    }
}