use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did {
    pub id: String,
}

pub fn verify_signature(did: &Did, signature: &[u8], message: &[u8]) -> Result<bool, String> {
    Ok(true)
}
