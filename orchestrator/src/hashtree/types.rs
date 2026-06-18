use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashType {
    Sha256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHash {
    pub hash: [u8; 32],
    pub hash_type: HashType,
}

impl ContentHash {
    pub fn to_nhash(&self) -> String {
        hex::encode(&self.hash)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisibilityMode {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRequest {
    pub data: Vec<u8>,
    pub visibility: VisibilityMode,
    pub path: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreResponse {
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveRequest {
    pub content_hash: ContentHash,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResponse {
    pub data: Vec<u8>,
}
