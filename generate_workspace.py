import os
import shutil

base = "safe-core-os-v3.0"
if os.path.exists(base):
    shutil.rmtree(base)
os.makedirs(base, exist_ok=True)

crates = [
    "safe-core-core", "safe-core-crypto", "safe-core-policy",
    "safe-core-mcp", "safe-core-a2a",
    "safe-core-parallax-bridge", "safe-core-model-runtime"
]
for c in crates:
    os.makedirs(os.path.join(base, "crates", c, "src"), exist_ok=True)

workspace_toml = '''[workspace]
resolver = "2"
members = [
    "crates/safe-core-core",
    "crates/safe-core-crypto",
    "crates/safe-core-policy",
    "crates/safe-core-mcp",
    "crates/safe-core-a2a",
    "crates/safe-core-parallax-bridge",
    "crates/safe-core-model-runtime",
]

[workspace.package]
version = "3.0.0"
edition = "2021"
authors = ["Arkhe Research Group <research@arkhe-os.org>"]
license = "MIT OR Apache-2.0"
rust-version = "1.85.0"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio = { version = "1.43", features = ["full"] }
async-trait = "0.1"
blake3 = "1.5"
ed25519-dalek = { version = "2.1", features = ["rand_core", "zeroize"] }
zeroize = { version = "1.8", features = ["derive"] }
rand_core = { version = "0.6", features = ["getrandom"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.12", features = ["v4", "serde"] }
hex = "0.4"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
rmcp = { version = "0.1.4", features = ["server", "transport-io"] }

safe-core-core = { path = "crates/safe-core-core" }
safe-core-crypto = { path = "crates/safe-core-crypto" }
safe-core-policy = { path = "crates/safe-core-policy" }
safe-core-mcp = { path = "crates/safe-core-mcp" }
safe-core-a2a = { path = "crates/safe-core-a2a" }
safe-core-parallax-bridge = { path = "crates/safe-core-parallax-bridge" }
safe-core-model-runtime = { path = "crates/safe-core-model-runtime" }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
'''

with open(os.path.join(base, "Cargo.toml"), "w") as f:
    f.write(workspace_toml)

def write_crate(name, cargo_toml, lib_rs, extra_files=None, proto=None, build=None):
    crate_path = os.path.join(base, "crates", name)
    with open(os.path.join(crate_path, "Cargo.toml"), "w") as f:
        f.write(cargo_toml)
    with open(os.path.join(crate_path, "src", "lib.rs"), "w") as f:
        f.write(lib_rs)
    if extra_files:
        for fname, content in extra_files.items():
            with open(os.path.join(crate_path, "src", fname), "w") as f:
                f.write(content)
    if proto:
        os.makedirs(os.path.join(crate_path, "proto"), exist_ok=True)
        with open(os.path.join(crate_path, "proto", "parallax.proto"), "w") as f:
            f.write(proto)
    if build:
        with open(os.path.join(crate_path, "build.rs"), "w") as f:
            f.write(build)

write_crate("safe-core-core", '''
[package]
name = "safe-core-core"
version.workspace = true
edition.workspace = true
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
blake3 = { workspace = true }
hex = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
[lints]
workspace = true
''', '''
pub mod error;
pub mod hash;
pub mod id;
pub mod traits;

pub use error::{CoreError, CoreResult};
pub use hash::Blake3Hash;
pub use id::AgentId;
pub use traits::{Signer, Verifier};
''', {
    "error.rs": '''
use thiserror::Error;
#[derive(Debug, Error, Clone)]
pub enum CoreError {
    #[error("Invalid input: {0}")] InvalidInput(String),
    #[error("Unauthorized: {0}")] Unauthorized(String),
    #[error("Internal error: {0}")] Internal(String),
}
pub type CoreResult<T> = Result<T, CoreError>;
''',
    "hash.rs": '''
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blake3Hash(pub [u8; 32]);
impl Blake3Hash {
    pub fn from_data(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(*hash.as_bytes())
    }
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}
impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.to_hex()) }
}
''',
    "id.rs": '''
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);
impl AgentId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn generate() -> Self { Self(uuid::Uuid::new_v4().to_string()) }
}
impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
''',
    "traits.rs": '''
use crate::error::CoreResult;
pub trait Signer: Send + Sync { fn sign(&self, data: &[u8]) -> CoreResult<Vec<u8>>; }
pub trait Verifier: Send + Sync { fn verify(&self, data: &[u8], signature: &[u8]) -> CoreResult<bool>; }
'''
})

write_crate("safe-core-policy", '''
[package]
name = "safe-core-policy"
version.workspace = true
edition.workspace = true
[dependencies]
safe-core-core = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
[lints]
workspace = true
''', '''
pub mod consensus_guard;
pub use consensus_guard::ConsensusGuard;
''', {
    "consensus_guard.rs": '''
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolicyError { #[error("Tool not allowed: {0}")] NotAllowed(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal { pub tool: String, pub payload: serde_json::Value }

pub struct ConsensusGuard { allowed_tools: HashSet<String> }
impl ConsensusGuard {
    pub fn new() -> Self { Self { allowed_tools: ["infer", "read"].iter().map(|s| s.to_string()).collect() } }
    pub fn evaluate(&self, proposal: &Proposal) -> Result<bool, PolicyError> {
        if !self.allowed_tools.contains(&proposal.tool) {
            return Err(PolicyError::NotAllowed(proposal.tool.clone()));
        }
        Ok(true)
    }
}
impl Default for ConsensusGuard { fn default() -> Self { Self::new() } }
'''
})

write_crate("safe-core-crypto", '''
[package]
name = "safe-core-crypto"
version.workspace = true
edition.workspace = true
[dependencies]
safe-core-core = { workspace = true }
[lints]
workspace = true
''', 'pub struct DummyCrypto;')

write_crate("safe-core-mcp", '''
[package]
name = "safe-core-mcp"
version.workspace = true
edition.workspace = true
[dependencies]
safe-core-core = { workspace = true }
rmcp = { workspace = true }
[lints]
workspace = true
''', 'pub struct McpServer;')

write_crate("safe-core-a2a", '''
[package]
name = "safe-core-a2a"
version.workspace = true
edition.workspace = true
[dependencies]
safe-core-core = { workspace = true }
[lints]
workspace = true
''', 'pub struct A2AClient;')

# Now parallax bridge and model runtime

parallax_cargo = '''
[package]
name = "safe-core-parallax-bridge"
version.workspace = true
edition.workspace = true
description = "Cliente gRPC para o scheduler do Parallax (Lattica P2P, SGLang, MLX LM)"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
tonic = { version = "0.12", features = ["tls", "tls-roots"] }
prost = "0.13"
uuid = { workspace = true }

[build-dependencies]
tonic-build = "0.12"

[lints]
workspace = true
'''

parallax_lib = '''
pub mod client;
pub mod error;
pub mod types;

pub use client::ParallaxClient;
pub use error::ParallaxError;
pub use types::*;
'''

parallax_error = '''
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ParallaxError {
    #[error("gRPC error: {0}")]
    Grpc(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Node unavailable")]
    NodeUnavailable,
    #[error("Not supported: {0}")]
    NotSupported(String),
}

impl From<tonic::Status> for ParallaxError {
    fn from(status: tonic::Status) -> Self {
        ParallaxError::Grpc(status.message().to_string())
    }
}

impl From<serde_json::Error> for ParallaxError {
    fn from(err: serde_json::Error) -> Self {
        ParallaxError::Serialization(err.to_string())
    }
}
'''

parallax_types = '''
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InferRequest {
    pub model_name: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub params: SamplingParams,
    pub tools: Vec<ToolDefinition>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SamplingParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<usize>,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: None,
            max_tokens: 2048,
            stop_sequences: Vec::new(),
            seed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct InferResponse {
    pub id: String,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
    pub finish_reason: String,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct HealthResponse {
    pub ready: bool,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub values: Vec<f32>,
}
'''

parallax_client = '''
use crate::error::ParallaxError;
use crate::types::*;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tonic::transport::Endpoint;

pub mod parallax {
    tonic::include_proto!("parallax");
}

use parallax::inference_service_client::InferenceServiceClient;
use parallax::{
    EmbedRequest, HealthRequest, InferRequest as ProtoInferRequest,
    ListModelsRequest,
};

pub struct ParallaxClient {
    inner: Mutex<InferenceServiceClient<tonic::transport::Channel>>,
}

impl ParallaxClient {
    pub async fn connect(addr: &str) -> Result<Self, ParallaxError> {
        let channel = Endpoint::try_from(addr.to_string())
            .map_err(|e| ParallaxError::Connection(format!("Invalid URI: {}", e)))?
            .connect()
            .await
            .map_err(|e| ParallaxError::Connection(format!("Connection failed: {}", e)))?;

        Ok(Self {
            inner: Mutex::new(InferenceServiceClient::new(channel)),
        })
    }

    pub async fn health(&self) -> Result<HealthResponse, ParallaxError> {
        let mut client = self.inner.lock().await;
        let resp = client
            .health(HealthRequest {})
            .await?
            .into_inner();

        Ok(HealthResponse {
            ready: resp.ready,
            version: resp.version,
        })
    }

    pub async fn list_models(&self) -> Result<Vec<String>, ParallaxError> {
        let mut client = self.inner.lock().await;
        let resp = client
            .list_models(ListModelsRequest {})
            .await?
            .into_inner();

        Ok(resp.models)
    }

    pub async fn infer(&self, req: InferRequest) -> Result<InferResponse, ParallaxError> {
        let top_k = req
            .params
            .top_k
            .map(|v| i32::try_from(v).unwrap_or(0))
            .unwrap_or(0);

        let max_tokens = i32::try_from(req.params.max_tokens).unwrap_or(i32::MAX);

        let seed = req
            .params
            .seed
            .map(|v| i64::try_from(v).unwrap_or(0))
            .unwrap_or(0);

        let proto_messages: Vec<parallax::ChatMessage> = req
            .messages
            .into_iter()
            .map(|m| parallax::ChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let proto_req = ProtoInferRequest {
            model_name: req.model_name,
            prompt: req.prompt,
            messages: proto_messages,
            params: Some(parallax::SamplingParams {
                temperature: req.params.temperature,
                top_p: req.params.top_p,
                top_k,
                max_tokens,
                stop: req.params.stop_sequences,
                seed,
            }),
            metadata: req.metadata,
        };

        let mut client = self.inner.lock().await;
        let response = client.infer(proto_req).await?.into_inner();

        let usage = response.usage.as_ref();

        let tool_calls: Vec<ToolCall> = response
            .tool_calls
            .into_iter()
            .map(|tc| ToolCall {
                id: tc.id,
                name: tc.name,
                arguments: match serde_json::from_str(&tc.arguments) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!("Failed to parse tool call arguments: {}", e);
                        serde_json::json!({})
                    }
                },
            })
            .collect();

        let finish_reason = match response.finish_reason.as_str() {
            "stop" => "stop".to_string(),
            "length" => "length".to_string(),
            "tool_calls" | "tool_call" => "tool_calls".to_string(),
            other => {
                tracing::warn!("Unknown finish_reason from Parallax: {}", other);
                "stop".to_string()
            }
        };

        Ok(InferResponse {
            id: response.id,
            content: response.content,
            tool_calls,
            usage: TokenUsage {
                prompt_tokens: usage.map(|u| u.prompt_tokens as u32).unwrap_or(0),
                completion_tokens: usage.map(|u| u.completion_tokens as u32).unwrap_or(0),
                total_tokens: usage.map(|u| u.total_tokens as u32).unwrap_or(0),
            },
            finish_reason,
        })
    }

    pub async fn embed(&self, model_name: &str, texts: Vec<String>) -> Result<Vec<Embedding>, ParallaxError> {
        let req = EmbedRequest {
            model_name: model_name.to_string(),
            texts,
        };

        let mut client = self.inner.lock().await;
        let resp = client.embed(req).await?.into_inner();

        Ok(resp
            .embeddings
            .into_iter()
            .map(|emb| Embedding { values: emb.values })
            .collect())
    }
}
'''

parallax_proto = '''
syntax = "proto3";

package parallax;

service InferenceService {
  rpc Infer (InferRequest) returns (InferResponse);
  rpc Health (HealthRequest) returns (HealthResponse);
  rpc ListModels (ListModelsRequest) returns (ListModelsResponse);
  rpc Embed (EmbedRequest) returns (EmbedResponse);
}

message InferRequest {
  string model_name = 1;
  string prompt = 2;
  repeated ChatMessage messages = 3;
  SamplingParams params = 4;
  map<string, string> metadata = 5;
}

message ChatMessage {
  string role = 1;
  string content = 2;
}

message SamplingParams {
  float temperature = 1;
  float top_p = 2;
  int32 top_k = 3;
  int32 max_tokens = 4;
  repeated string stop = 5;
  int64 seed = 6;
}

message InferResponse {
  string id = 1;
  string content = 2;
  repeated ToolCall tool_calls = 3;
  TokenUsage usage = 4;
  string finish_reason = 5;
}

message ToolCall {
  string id = 1;
  string name = 2;
  string arguments = 3;
}

message TokenUsage {
  int32 prompt_tokens = 1;
  int32 completion_tokens = 2;
  int32 total_tokens = 3;
}

message HealthRequest {}
message HealthResponse {
  bool ready = 1;
  string version = 2;
}

message ListModelsRequest {}
message ListModelsResponse {
  repeated string models = 1;
}

message EmbedRequest {
  string model_name = 1;
  repeated string texts = 2;
}

message EmbedResponse {
  repeated Embedding embeddings = 1;
}

message Embedding {
  repeated float values = 1;
}
'''

parallax_build = '''
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/parallax.proto")?;
    Ok(())
}
'''

write_crate("safe-core-parallax-bridge", parallax_cargo, parallax_lib,
            extra_files={
                "error.rs": parallax_error,
                "types.rs": parallax_types,
                "client.rs": parallax_client,
            },
            proto=parallax_proto,
            build=parallax_build)

model_cargo = '''
[package]
name = "safe-core-model-runtime"
version.workspace = true
edition.workspace = true
description = "Unified model runtime (Candle, Parallax) with Convenção X"

[dependencies]
safe-core-core = { workspace = true }
safe-core-policy = { workspace = true }
safe-core-parallax-bridge = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
tokio = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

[lints]
workspace = true
'''

model_lib = '''
pub mod backends;
pub mod error;
pub mod runtime;
pub mod types;

pub use backends::parallax::ParallaxBackend;
pub use error::RuntimeError;
pub use runtime::{ModelRuntime, RuntimeRegistry, register_parallax};
pub use types::{
    ChatMessage, FinishReason, InferenceRequest, InferenceResponse,
    ModelConfig, SamplingParams, Tensor, TokenUsage, ToolCall, ToolDefinition,
};
'''

model_error = '''
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    #[error("Policy denied: {0}")]
    Policy(String),
    #[error("Backend error: {0}")]
    Backend(String),
    #[error("Model not found: {0}")]
    NotFound(String),
    #[error("Model not ready")]
    NotReady,
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Not supported: {0}")]
    NotSupported(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<safe_core_parallax_bridge::ParallaxError> for RuntimeError {
    fn from(err: safe_core_parallax_bridge::ParallaxError) -> Self {
        match err {
            safe_core_parallax_bridge::ParallaxError::ModelNotFound(msg) => {
                RuntimeError::NotFound(msg)
            }
            safe_core_parallax_bridge::ParallaxError::NotSupported(msg) => {
                RuntimeError::NotSupported(msg)
            }
            other => RuntimeError::Backend(other.to_string()),
        }
    }
}
'''

model_types = '''
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub params: SamplingParams,
    pub tools: Vec<ToolDefinition>,
    pub metadata: HashMap<String, String>,
}

impl InferenceRequest {
    pub fn simple(prompt: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: prompt.into(),
            system_prompt: None,
            messages: Vec::new(),
            params: SamplingParams::default(),
            tools: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn chat(messages: Vec<ChatMessage>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: String::new(),
            system_prompt: None,
            messages,
            params: SamplingParams::default(),
            tools: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<usize>,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: None,
            max_tokens: 2048,
            stop_sequences: Vec::new(),
            seed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub id: String,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    Error,
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinishReason::Stop => write!(f, "stop"),
            FinishReason::Length => write!(f, "length"),
            FinishReason::ToolCall => write!(f, "tool_calls"),
            FinishReason::Error => write!(f, "error"),
        }
    }
}

impl From<&str> for FinishReason {
    fn from(s: &str) -> Self {
        match s {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" | "tool_call" => FinishReason::ToolCall,
            _ => {
                tracing::warn!("Unknown finish_reason, defaulting to Stop: {}", s);
                FinishReason::Stop
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
}

impl Tensor {
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Self {
        Self { data, shape }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_name: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "deepseek-r1".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            top_p: 0.9,
        }
    }
}
'''

model_runtime = '''
use crate::error::RuntimeError;
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait ModelRuntime: Send + Sync {
    async fn x_infer(&self, request: InferenceRequest) -> Result<InferenceResponse, RuntimeError>;

    async fn x_infer_chat(&self, messages: Vec<ChatMessage>) -> Result<InferenceResponse, RuntimeError> {
        let request = InferenceRequest::chat(messages);
        self.x_infer(request).await
    }

    async fn x_embed(&self, _texts: Vec<String>) -> Result<Vec<Tensor>, RuntimeError> {
        Err(RuntimeError::NotSupported(
            "Embedding not supported by this backend".into(),
        ))
    }

    fn model_name(&self) -> &str;

    async fn is_ready(&self) -> bool;
}

pub struct RuntimeRegistry {
    backends: tokio::sync::RwLock<HashMap<String, Arc<dyn ModelRuntime>>>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            backends: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, backend: Arc<dyn ModelRuntime>) -> Result<(), RuntimeError> {
        let name = backend.model_name().to_string();
        let mut backends = self.backends.write().await;
        backends.insert(name.clone(), backend);
        tracing::info!("Registered backend: {}", name);
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Result<Arc<dyn ModelRuntime>, RuntimeError> {
        let backends = self.backends.read().await;
        backends
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("Backend '{}' not registered", name)))
    }

    pub async fn list(&self) -> Vec<String> {
        let backends = self.backends.read().await;
        backends.keys().cloned().collect()
    }
}

impl Default for RuntimeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn register_parallax(
    registry: &RuntimeRegistry,
    addr: &str,
    model: &str,
    config: crate::types::ModelConfig,
    guard: safe_core_policy::ConsensusGuard,
) -> Result<(), RuntimeError> {
    let backend = crate::backends::parallax::ParallaxBackend::new(addr, model, config, guard).await?;
    registry.register(std::sync::Arc::new(backend)).await?;
    Ok(())
}
'''

model_backend_mod = '''
pub mod parallax;

pub use parallax::ParallaxBackend;
'''

model_backend_parallax = '''
use crate::error::RuntimeError;
use crate::runtime::ModelRuntime;
use crate::types::*;
use async_trait::async_trait;
use safe_core_parallax_bridge::ParallaxClient;
use safe_core_policy::{ConsensusGuard, Proposal};
use tracing::{info, warn};

pub struct ParallaxBackend {
    client: ParallaxClient,
    guard: ConsensusGuard,
    model_name: String,
    config: ModelConfig,
}

impl ParallaxBackend {
    pub async fn new(
        addr: &str,
        model_name: &str,
        config: ModelConfig,
        guard: ConsensusGuard,
    ) -> Result<Self, RuntimeError> {
        let client = ParallaxClient::connect(addr)
            .await
            .map_err(|e| RuntimeError::Backend(format!("Connection failed: {}", e)))?;

        let health = client
            .health()
            .await
            .map_err(|e| RuntimeError::Backend(format!("Health check failed: {}", e)))?;

        if !health.ready {
            return Err(RuntimeError::NotReady);
        }

        info!(
            "Parallax cluster ready (version: {}), checking model availability...",
            health.version
        );

        let available_models = client
            .list_models()
            .await
            .map_err(|e| RuntimeError::Backend(format!("Failed to list models: {}", e)))?;

        if !available_models.contains(&model_name.to_string()) {
            return Err(RuntimeError::NotFound(format!(
                "Model '{}' not found. Available models: {:?}",
                model_name, available_models
            )));
        }

        info!("Model '{}' confirmed available", model_name);

        Ok(Self {
            client,
            guard,
            model_name: model_name.to_string(),
            config,
        })
    }
}

#[async_trait]
impl ModelRuntime for ParallaxBackend {
    async fn x_infer(&self, request: InferenceRequest) -> Result<InferenceResponse, RuntimeError> {
        if request.prompt.is_empty() && request.messages.is_empty() {
            return Err(RuntimeError::InvalidRequest(
                "Either prompt or messages must be provided".into(),
            ));
        }

        let proposal = Proposal {
            tool: "infer".to_string(),
            payload: serde_json::json!({
                "model": self.model_name,
                "prompt_length": request.prompt.len(),
                "messages_count": request.messages.len(),
            }),
        };

        self.guard
            .evaluate(&proposal)
            .map_err(|e| RuntimeError::Policy(e.to_string()))?;

        let params = safe_core_parallax_bridge::SamplingParams {
            temperature: request.params.temperature,
            top_p: request.params.top_p,
            top_k: request.params.top_k,
            max_tokens: request.params.max_tokens,
            stop_sequences: request.params.stop_sequences,
            seed: request.params.seed,
        };

        let bridge_request = safe_core_parallax_bridge::InferRequest {
            model_name: self.model_name.clone(),
            prompt: request.prompt,
            system_prompt: request.system_prompt,
            messages: request
                .messages
                .into_iter()
                .map(|m| safe_core_parallax_bridge::ChatMessage {
                    role: m.role,
                    content: m.content,
                })
                .collect(),
            params,
            tools: request
                .tools
                .into_iter()
                .map(|t| safe_core_parallax_bridge::ToolDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                })
                .collect(),
            metadata: request.metadata,
        };

        let resp = self.client.infer(bridge_request).await?;

        Ok(InferenceResponse {
            id: resp.id,
            content: resp.content,
            tool_calls: resp
                .tool_calls
                .into_iter()
                .map(|tc| ToolCall {
                    id: tc.id,
                    name: tc.name,
                    arguments: tc.arguments,
                })
                .collect(),
            usage: TokenUsage {
                prompt_tokens: resp.usage.prompt_tokens,
                completion_tokens: resp.usage.completion_tokens,
                total_tokens: resp.usage.total_tokens,
            },
            finish_reason: FinishReason::from(resp.finish_reason.as_str()),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn x_embed(&self, texts: Vec<String>) -> Result<Vec<Tensor>, RuntimeError> {
        if texts.is_empty() {
            return Err(RuntimeError::InvalidRequest(
                "texts must not be empty".into(),
            ));
        }

        let embeddings = self
            .client
            .embed(&self.model_name, texts)
            .await
            .map_err(|e| RuntimeError::Backend(format!("Embedding failed: {}", e)))?;

        Ok(embeddings
            .into_iter()
            .map(|emb| Tensor::new(emb.values, vec![emb.values.len()]))
            .collect())
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn is_ready(&self) -> bool {
        match self.client.health().await {
            Ok(health) => health.ready,
            Err(e) => {
                warn!("Health check failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_guard() -> ConsensusGuard {
        ConsensusGuard::new()
    }

    fn make_config() -> ModelConfig {
        ModelConfig {
            model_name: "test-model".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            top_p: 0.9,
        }
    }

    #[test]
    fn test_sampling_params_conversion() {
        let params = safe_core_parallax_bridge::SamplingParams {
            temperature: 0.5,
            top_p: 0.8,
            top_k: Some(50),
            max_tokens: 1024,
            stop_sequences: vec!["\\n".to_string()],
            seed: Some(42),
        };

        assert_eq!(params.temperature, 0.5);
        assert_eq!(params.top_k, Some(50));
        assert_eq!(params.max_tokens, 1024);
    }

    #[test]
    fn test_safe_numeric_conversion_top_k() {
        let top_k: usize = 50;
        let result = i32::try_from(top_k).unwrap_or(0);
        assert_eq!(result, 50);

        let top_k_overflow: usize = i32::MAX as usize + 1;
        let result = i32::try_from(top_k_overflow).unwrap_or(0);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_safe_numeric_conversion_max_tokens() {
        let max_tokens: usize = 2048;
        let result = i32::try_from(max_tokens).unwrap_or(i32::MAX);
        assert_eq!(result, 2048);

        let max_tokens_overflow: usize = i32::MAX as usize + 1000;
        let result = i32::try_from(max_tokens_overflow).unwrap_or(i32::MAX);
        assert_eq!(result, i32::MAX);
    }

    #[test]
    fn test_safe_numeric_conversion_seed() {
        let seed: u64 = 42;
        let result = i64::try_from(seed).unwrap_or(0);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_finish_reason_parsing() {
        assert_eq!(FinishReason::from("stop"), FinishReason::Stop);
        assert_eq!(FinishReason::from("length"), FinishReason::Length);
        assert_eq!(FinishReason::from("tool_calls"), FinishReason::ToolCall);
        assert_eq!(FinishReason::from("tool_call"), FinishReason::ToolCall);
        assert_eq!(FinishReason::from("unknown"), FinishReason::Stop);
    }

    #[test]
    fn test_inference_request_simple() {
        let req = InferenceRequest::simple("Hello world");
        assert_eq!(req.prompt, "Hello world");
        assert!(req.messages.is_empty());
    }

    #[test]
    fn test_inference_request_chat() {
        let messages = vec![
            ChatMessage::system("You are helpful"),
            ChatMessage::user("Hi"),
        ];
        let req = InferenceRequest::chat(messages);
        assert!(req.prompt.is_empty());
        assert_eq!(req.messages.len(), 2);
    }

    #[test]
    fn test_chat_message_constructors() {
        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, "user");

        let assistant = ChatMessage::assistant("Hi there");
        assert_eq!(assistant.role, "assistant");

        let system = ChatMessage::system("Be helpful");
        assert_eq!(system.role, "system");
    }

    #[test]
    fn test_policy_guard_allows_infer() {
        let guard = make_guard();
        let proposal = Proposal {
            tool: "infer".to_string(),
            payload: serde_json::json!({"prompt": "test"}),
        };
        assert!(guard.evaluate(&proposal).is_ok());
    }

    #[test]
    fn test_policy_guard_rejects_unknown_tool() {
        let guard = make_guard();
        let proposal = Proposal {
            tool: "malicious_tool".to_string(),
            payload: serde_json::json!({}),
        };
        assert!(guard.evaluate(&proposal).is_err());
    }

    #[tokio::test]
    async fn test_runtime_registry() {
        let registry = RuntimeRegistry::new();
        assert!(registry.list().await.is_empty());
    }

    #[test]
    fn test_tensor_creation() {
        let data = vec![1.0, 2.0, 3.0];
        let shape = vec![3];
        let tensor = Tensor::new(data.clone(), shape.clone());
        assert_eq!(tensor.data, data);
        assert_eq!(tensor.shape, shape);
    }
}
'''

write_crate("safe-core-model-runtime", model_cargo, model_lib,
            extra_files={
                "error.rs": model_error,
                "types.rs": model_types,
                "runtime.rs": model_runtime,
                "backends/mod.rs": model_backend_mod,
                "backends/parallax.rs": model_backend_parallax,
            })
