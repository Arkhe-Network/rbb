use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ZkMemoryProofPolicy {
    pub require_memory_proof_for_recommendations: bool,
}
