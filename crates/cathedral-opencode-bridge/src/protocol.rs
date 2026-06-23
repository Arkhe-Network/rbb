use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeRequest {
    pub agent: String,
    pub session_id: String,
    pub prompt: String,
    pub context: serde_json::Value,
    pub permissions: Vec<String>,
    pub did: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub proof: Option<String>,
}
