use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileRequest {
    pub source: String,
    pub version: String,
    pub optimize: bool,
    pub runs: u32,
    pub did: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResponse {
    pub success: bool,
    pub abi: Option<serde_json::Value>,
    pub bytecode: Option<String>,
    pub bytecode_hash: Option<String>,
    pub ast: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSessionRequest {
    pub tx_hash: String,
    pub network: String,
    pub did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStepRequest {
    pub session_id: String,
    pub step: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRequest {
    pub bytecode: String,
    pub abi: serde_json::Value,
    pub network: String,
    pub from: String,
    pub gas_limit: u64,
    pub did: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResponse {
    pub success: bool,
    pub contract_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub error: Option<String>,
}
