use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did {
    pub id: String,
}

pub fn verify_signature(_did: &Did, _signature: &[u8], _message: &[u8]) -> Result<bool, String> {
    Ok(true)
}
