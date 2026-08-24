use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionHash(pub [u8; 32]);

impl Eq for DecisionHash {}

impl std::hash::Hash for DecisionHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemStateHash(pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionDomain {
    Core,
    Cryptography,
    Networking,
    Storage,
    Hermeneutics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReferenceKind {
    RFC,
    Issue,
    PullRequest,
    Conversation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Verified { by: String, at: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reference {
    pub kind: ReferenceKind,
    pub identifier: String,
    pub verification_status: VerificationStatus,
    pub verification_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rationale {
    pub metaphor: Option<String>,
    pub computational_translation: String,
    pub metaphor_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RejectedAlternative {
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub rejected_reason: String,
    pub refutation_references: Vec<Reference>,
    pub confidence_at_rejection: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Negation {
    pub description: String,
    pub rationale: String,
    pub alternatives: Vec<RejectedAlternative>,
    pub foreclosure_references: Vec<Reference>,
    pub refusal_timestamp: u64,
    pub refusal_hash: DecisionHash,
    pub precondition_for_reopening: Option<String>,
    pub evidence_that_would_reopen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidityWindow {
    pub years: u32,
    pub revisit_trigger: String,
    pub confidence_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitecturalDecision {
    pub id: Uuid,
    pub timestamp: u64,
    pub status: DecisionStatus,
    pub domain: DecisionDomain,
    pub title: String,
    pub problem: String,
    pub decision_outcome: String,
    pub rationale: Rationale,
    pub references: Vec<Reference>,
    pub negation: Negation,
    pub expected_validity: Option<ValidityWindow>,
    pub decision_makers: Vec<String>,
    pub consulted: Vec<String>,
    pub informed: Vec<String>,
    pub system_state_hash: Option<SystemStateHash>,
    pub decision_hash: DecisionHash,
    pub previous_decision_hash: Option<DecisionHash>,
    pub superseded_by: Option<Uuid>,
}
