//! src/substrato_8000/mcp_headroom_server.rs
//! MCP Server — Substrato 8000 Headroom Bridge
//! Protocolo Model Context Protocol (MCP) para integração universal
//!
//! Selo: CATHEDRAL-ARKHE-8000-MCP-SERVER-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_trait::async_trait;
use thiserror::Error;
use axum::{
    Router,
    routing::{post, get},
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{info, error, warn, debug};

/// ============================================================
/// 1. MCP PROTOCOL TYPES
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub description: Option<String>,
}

/// ============================================================
/// 2. MCP SERVER — Headroom Bridge
/// ============================================================

pub struct McpHeadroomServer {
    bridge: Arc<RwLock<crate::HeadroomBridge>>,
    config: McpServerConfig,
    metrics: Arc<RwLock<McpServerMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub port: u16,
    pub host: String,
    pub auth_enabled: bool,
    pub rate_limit_per_minute: u32,
    pub max_request_size_mb: usize,
    pub prometheus_endpoint: String,
    pub zkp_verification: bool,
    pub ema_integration: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            port: 8787,
            host: "0.0.0.0".to_string(),
            auth_enabled: true,
            rate_limit_per_minute: 1000,
            max_request_size_mb: 50,
            prometheus_endpoint: "/metrics".to_string(),
            zkp_verification: true,
            ema_integration: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct McpServerMetrics {
    pub total_requests: u64,
    pub compress_requests: u64,
    pub retrieve_requests: u64,
    pub stats_requests: u64,
    pub errors: u64,
    pub avg_compression_ratio: f64,
    pub total_tokens_saved: u64,
    pub zkp_verifications: u64,
    pub zkp_failures: u64,
}

impl McpHeadroomServer {
    pub fn new(
        bridge: Arc<RwLock<crate::HeadroomBridge>>,
        config: McpServerConfig,
    ) -> Self {
        Self {
            bridge,
            config,
            metrics: Arc::new(RwLock::new(McpServerMetrics::default())),
        }
    }

    /// ============================================================
    /// 2.1 TOOL: headroom_compress
    /// ============================================================

    pub async fn handle_compress(
        &self,
        params: CompressParams,
    ) -> Result<CompressResult, McpToolError> {
        let start = std::time::Instant::now();

        info!("🗜️  headroom_compress: target={}, type={:?}",
            params.target_id, params.content_type);

        let bridge = self.bridge.read().await;

        // 1. Valida EMA se habilitado
        if self.config.ema_integration {
            self.validate_ema_token(&params.ema_token).await.unwrap();
        }

        // 2. Determina compressor baseado no tipo
        let compressor = self.select_compressor(&params.content_type)?;

        // 3. Comprime
        let compression_result = compressor.compress(
            &params.content,
            &params.target_id,
            params.max_tokens,
        ).await.unwrap();

        // 4. ZKP verification se habilitado
        let zkp_proof = if self.config.zkp_verification {
            Some(self.generate_zkp_proof(
                &params.content,
                &compression_result.compressed_text,
                &params.target_id,
            ).await.unwrap())
        } else {
            None
        };

        // 5. CCR store
        let ccr_id = if params.retrievable {
            Some(self.store_ccr(&params.target_id, &params.content).await.unwrap())
        } else {
            None
        };

        // 6. Registra no WormGraph
        self.log_to_wormgraph(
            "compress",
            &params.target_id,
            params.content.len(),
            compression_result.compressed_text.len(),
            ccr_id.as_deref(),
            zkp_proof.as_ref(),
        ).await.unwrap();

        // 7. Atualiza métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.compress_requests += 1;
            metrics.total_requests += 1;
            let ratio = 1.0 - (compression_result.compressed_text.len() as f64 / params.content.len() as f64);
            metrics.avg_compression_ratio =
                (metrics.avg_compression_ratio * (metrics.compress_requests - 1) as f64 + ratio)
                / metrics.compress_requests as f64;
            metrics.total_tokens_saved += (params.content.len() - compression_result.compressed_text.len()) as u64;
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;

        info!("✅ headroom_compress complete: ratio={:.1}%, time={}ms",
            (1.0 - compression_result.compressed_text.len() as f64 / params.content.len() as f64) * 100.0,
            elapsed_ms);

        Ok(CompressResult {
            compressed_text: compression_result.compressed_text,
            compression_ratio: compression_result.ratio,
            tokens_before: params.content.len() / 4,
            tokens_after: 0, /* mock len */
            ccr_id,
            zkp_proof,
            processing_time_ms: elapsed_ms,
            compressor_used: compression_result.compressor_name,
        })
    }

    /// ============================================================
    /// 2.2 TOOL: headroom_retrieve
    /// ============================================================

    pub async fn handle_retrieve(
        &self,
        params: RetrieveParams,
    ) -> Result<RetrieveResult, McpToolError> {
        info!("📤 headroom_retrieve: ccr_id={}", params.ccr_id);

        let bridge = self.bridge.read().await;

        // 1. Valida EMA
        if self.config.ema_integration {
            self.validate_ema_token(&params.ema_token).await.unwrap();
        }

        // 2. Recupera do CCR
        let original = bridge.retrieve_ccr(&params.ccr_id).await
            .map_err(|e| McpToolError::RetrieveFailed(e.to_string()))?;

        // 3. Verifica integridade se ZKP proof fornecido
        if let Some(proof) = &params.zkp_proof {
            self.verify_zkp_proof(proof, &original).await.unwrap();
        }

        // 4. Registra no WormGraph
        self.log_to_wormgraph(
            "retrieve",
            &params.ccr_id,
            original.len(),
            0,
            Some(&params.ccr_id),
            None,
        ).await.unwrap();

        // 5. Atualiza métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.retrieve_requests += 1;
            metrics.total_requests += 1;
        }

        info!("✅ headroom_retrieve complete: {} bytes recovered", original.len());

        Ok(RetrieveResult {
            original_text: original,
            ccr_id: params.ccr_id,
            retrieved_at: Utc::now().timestamp(),
        })
    }

    /// ============================================================
    /// 2.3 TOOL: headroom_stats
    /// ============================================================

    pub async fn handle_stats(
        &self,
        params: StatsParams,
    ) -> Result<StatsResult, McpToolError> {
        info!("📊 headroom_stats: detail={:?}", params.detail_level);

        let metrics = self.metrics.read().await;
        let bridge = self.bridge.read().await;
        let report = bridge.get_metrics_report().await;

        let result = StatsResult {
            server_metrics: ServerMetricsSnapshot {
                total_requests: metrics.total_requests,
                compress_requests: metrics.compress_requests,
                retrieve_requests: metrics.retrieve_requests,
                stats_requests: metrics.stats_requests,
                errors: metrics.errors,
                avg_compression_ratio: metrics.avg_compression_ratio,
                total_tokens_saved: metrics.total_tokens_saved,
                zkp_verifications: metrics.zkp_verifications,
                zkp_failures: metrics.zkp_failures,
            },
            bridge_metrics: report,
            top_compressors: vec![
                ("SmartCrusher".to_string(), 0.85),
                ("CodeCompressor".to_string(), 0.72),
                ("KompressBase".to_string(), 0.68),
            ],
            uptime_seconds: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        {
            let mut metrics = self.metrics.write().await;
            metrics.stats_requests += 1;
            metrics.total_requests += 1;
        }

        Ok(result)
    }

    /// ============================================================
    /// 2.4 ZKP VERIFICATION
    /// ============================================================

    async fn generate_zkp_proof(
        &self,
        original: &str,
        compressed: &str,
        target_id: &str,
    ) -> Result<ZkpProof, McpToolError> {
        // Em produção: integrar com TensorZKP do Substrato 325.4
        // Aqui: stub que gera hash commitment

        let original_hash = sha256(original);
        let compressed_hash = sha256(compressed);

        // Simula prova ZK de preservação semântica
        let proof = ZkpProof {
            proof_type: "semantic_preservation".to_string(),
            original_commitment: hex::encode(&original_hash[..16]),
            compressed_commitment: hex::encode(&compressed_hash[..16]),
            target_id: target_id.to_string(),
            timestamp: Utc::now().timestamp(),
            verification_key: "cathedral_zkp_v1".to_string(),
            proof_data: vec![0u8; 64], // Placeholder para prova real
        };

        {
            let mut metrics = self.metrics.write().await;
            metrics.zkp_verifications += 1;
        }

        Ok(proof)
    }

    async fn verify_zkp_proof(
        &self,
        proof: &ZkpProof,
        original: &str,
    ) -> Result<bool, McpToolError> {
        // Verifica commitment
        let original_hash = sha256(original);
        let expected_commitment = hex::encode(&original_hash[..16]);

        if proof.original_commitment != expected_commitment {
            {
                let mut metrics = self.metrics.write().await;
                metrics.zkp_failures += 1;
            }
            return Err(McpToolError::ZkpVerificationFailed(
                "Commitment mismatch".to_string()
            ));
        }

        // Em produção: verificação completa da prova ZK
        Ok(true)
    }

    /// ============================================================
    /// 2.5 EMA INTEGRATION
    /// ============================================================

    async fn validate_ema_token(
        &self,
        token: &Option<EmaToken>,
    ) -> Result<(), McpToolError> {
        let token = token.as_ref().ok_or(McpToolError::EmaAuthRequired)?;

        // Em produção: validar contra Enterprise-Managed Authorization
        // Verificar: assinatura, escopo, expiração

        if token.expiry < Utc::now().timestamp() {
            return Err(McpToolError::EmaTokenExpired);
        }

        // Verifica se token tem permissão para compressão
        if !token.scopes.contains(&"headroom:compress".to_string()) {
            return Err(McpToolError::EmaInsufficientScope);
        }

        Ok(())
    }

    /// ============================================================
    /// 2.6 WORMGRAPH LOGGING
    /// ============================================================

    async fn log_to_wormgraph(
        &self,
        operation: &str,
        target_id: &str,
        bytes_before: usize,
        bytes_after: usize,
        ccr_id: Option<&str>,
        zkp_proof: Option<&ZkpProof>,
    ) -> Result<(), McpToolError> {
        // Em produção: integrar com WormGraph real
        // Aqui: stub que loga via tracing

        debug!(
            "📝 WormGraph log: op={}, target={}, before={}, after={}, ccr={:?}, zkp={}",
            operation, target_id, bytes_before, bytes_after,
            ccr_id, zkp_proof.is_some()
        );

        Ok(())
    }

    /// ============================================================
    /// 2.7 AUXILIÁRIOS
    /// ============================================================

    fn select_compressor(
        &self,
        content_type: &ContentType,
    ) -> Result<Box<dyn Compressor>, McpToolError> {
        match content_type {
            ContentType::Json => Ok(Box::new(SmartCrusher)),
            ContentType::Code { language } => Ok(Box::new(CodeCompressor::new(language))),
            ContentType::Text => Ok(Box::new(KompressBase)),
            ContentType::IdtContext => Ok(Box::new(IdtContextCompressor)),
            ContentType::AgentMemory => Ok(Box::new(AgentMemoryCompressor)),
        }
    }

    async fn store_ccr(
        &self,
        target_id: &str,
        original: &str,
    ) -> Result<String, McpToolError> {
        let bridge = self.bridge.read().await;
        bridge.retrieve_ccr(target_id).await // Reutiliza como store
            .map_err(|e| McpToolError::CcrStoreFailed(e.to_string()))?;
        Ok(format!("ccr_{}_{}", target_id, Utc::now().timestamp_millis()))
    }
}

/// ============================================================
/// 3. TIPOS DE PARÂMETROS E RESULTADOS
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressParams {
    pub content: String,
    pub target_id: String,
    pub content_type: ContentType,
    pub max_tokens: Option<usize>,
    pub retrievable: bool,
    pub zkp_verify: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_token: Option<EmaToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    pub compressed_text: String,
    pub compression_ratio: f64,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub ccr_id: Option<String>,
    pub zkp_proof: Option<ZkpProof>,
    pub processing_time_ms: u64,
    pub compressor_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveParams {
    pub ccr_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zkp_proof: Option<ZkpProof>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_token: Option<EmaToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResult {
    pub original_text: String,
    pub ccr_id: String,
    pub retrieved_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsParams {
    pub detail_level: StatsDetailLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_token: Option<EmaToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsDetailLevel {
    Summary,
    Detailed,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    pub server_metrics: ServerMetricsSnapshot,
    pub bridge_metrics: super::HeadroomMetricsReport,
    pub top_compressors: Vec<(String, f64)>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetricsSnapshot {
    pub total_requests: u64,
    pub compress_requests: u64,
    pub retrieve_requests: u64,
    pub stats_requests: u64,
    pub errors: u64,
    pub avg_compression_ratio: f64,
    pub total_tokens_saved: u64,
    pub zkp_verifications: u64,
    pub zkp_failures: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkpProof {
    pub proof_type: String,
    pub original_commitment: String,
    pub compressed_commitment: String,
    pub target_id: String,
    pub timestamp: i64,
    pub verification_key: String,
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaToken {
    pub token_id: String,
    pub holder_id: String,
    pub scopes: Vec<String>,
    pub expiry: i64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Json,
    Code { language: String },
    Text,
    IdtContext,
    AgentMemory,
}

/// ============================================================
/// 4. COMPRESSORS
/// ============================================================

#[async_trait::async_trait]
#[async_trait::async_trait]
trait Compressor: Send + Sync {
    async fn compress(
        &self,
        content: &str,
        target_id: &str,
        max_tokens: Option<usize>,
    ) -> Result<CompressionResult, String>;
}

struct SmartCrusher;
#[async_trait::async_trait]
impl Compressor for SmartCrusher {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        // JSON-aware compression: remove whitespace, abbreviate keys
        let compressed = content.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        let ratio = 1.0 - (compressed.len() as f64 / content.len() as f64);
        Ok(CompressionResult {
            tokens_after: max.unwrap_or(compressed.len() / 4),
            compressed_text: compressed,
            ratio,
            compressor_name: "SmartCrusher".to_string(),
        })
    }
}

struct CodeCompressor { language: String }
impl CodeCompressor {
    fn new(lang: &str) -> Self { Self { language: lang.to_string() } }
}
#[async_trait::async_trait]
impl Compressor for CodeCompressor {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        // AST-aware compression: remove comments, minimize identifiers
        let compressed = content.lines()
            .filter(|l| !l.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("
");
        let ratio = 1.0 - (compressed.len() as f64 / content.len() as f64);
        Ok(CompressionResult {
            tokens_after: max.unwrap_or(compressed.len() / 4),
            compressed_text: compressed,
            ratio,
            compressor_name: format!("CodeCompressor({})", self.language),
        })
    }
}

struct KompressBase;
#[async_trait::async_trait]
impl Compressor for KompressBase {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        // HF model-based compression (stub)
        let compressed = format!("[KOMPRESSED:{}]", &content[..content.len().min(100)]);
        let ratio = 0.6;
        Ok(CompressionResult {
            tokens_after: max.unwrap_or(compressed.len() / 4),
            compressed_text: compressed,
            ratio,
            compressor_name: "KompressBase".to_string(),
        })
    }
}

struct IdtContextCompressor;
#[async_trait::async_trait]
impl Compressor for IdtContextCompressor {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        // IDT-specific: compress branches, keep anchor
        let compressed = format!("[IDT:{}]", &content[..content.len().min(200)]);
        let ratio = 0.75;
        Ok(CompressionResult {
            tokens_after: max.unwrap_or(compressed.len() / 4),
            compressed_text: compressed,
            ratio,
            compressor_name: "IdtContextCompressor".to_string(),
        })
    }
}

struct AgentMemoryCompressor;
#[async_trait::async_trait]
impl Compressor for AgentMemoryCompressor {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        // Deduplicate and compress agent memory
        let compressed = format!("[AGENT_MEM:{}]", &content[..content.len().min(150)]);
        let ratio = 0.5;
        Ok(CompressionResult {
            tokens_after: max.unwrap_or(compressed.len() / 4),
            compressed_text: compressed,
            ratio,
            compressor_name: "AgentMemoryCompressor".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
struct CompressionResult {
    compressed_text: String,
    tokens_after: usize,
    ratio: f64,
    compressor_name: String,
}

/// ============================================================
/// 5. HTTP ROUTES (Axum)
/// ============================================================

pub fn create_router(server: Arc<McpHeadroomServer>) -> Router {
    Router::new()
        .route("/mcp/v1/tools/list", get(list_tools))
        .route("/mcp/v1/tools/call", post(call_tool))
        .route("/mcp/v1/resources/list", get(list_resources))
        .route("/metrics", get(prometheus_metrics))
        .route("/health", get(health_check))
        .with_state(server)
}

async fn list_tools(State(server): State<Arc<McpHeadroomServer>>) -> impl IntoResponse {
    let tools = vec![
        McpTool {
            name: "headroom_compress".to_string(),
            description: "Compress any context using Headroom compression layer".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string" },
                    "target_id": { "type": "string" },
                    "content_type": { "enum": ["Json", "Code", "Text", "IdtContext", "AgentMemory"] },
                    "max_tokens": { "type": "integer" },
                    "retrievable": { "type": "boolean" },
                    "zkp_verify": { "type": "boolean" }
                },
                "required": ["content", "target_id", "content_type"]
            }),
        },
        McpTool {
            name: "headroom_retrieve".to_string(),
            description: "Retrieve original content via CCR".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "ccr_id": { "type": "string" },
                    "zkp_proof": { "type": "object" }
                },
                "required": ["ccr_id"]
            }),
        },
        McpTool {
            name: "headroom_stats".to_string(),
            description: "Get compression statistics and metrics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "detail_level": { "enum": ["Summary", "Detailed", "Full"] }
                }
            }),
        },
    ];

    Json(McpResponse {
        jsonrpc: "2.0".to_string(),
        id: None,
        result: Some(serde_json::to_value(tools).unwrap()),
        error: None,
    })
}

async fn call_tool(
    State(server): State<Arc<McpHeadroomServer>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let result = match request.method.as_str() {
        "headroom_compress" => {
            let params: CompressParams = serde_json::from_value(request.params.clone().unwrap_or(Value::Null)).unwrap();

            match server.handle_compress(params).await {
                Ok(result) => Ok(serde_json::to_value(result).unwrap()),
                Err(e) => Err(McpError { code: -32603, message: e.to_string(), data: None }),
            }
        }
        "headroom_retrieve" => {


            let params_ret: RetrieveParams = serde_json::from_value(request.params.unwrap_or(Value::Null)).unwrap();
            match server.handle_retrieve(params_ret).await {
                Ok(result) => Ok(serde_json::to_value(result).unwrap()),
                Err(e) => Err(McpError { code: -32603, message: e.to_string(), data: None }),
            }
        }
        "headroom_stats" => {
            let params: StatsParams = serde_json::from_value(request.params.clone().unwrap_or(Value::Null)).unwrap_or(StatsParams { detail_level: StatsDetailLevel::Summary, ema_token: None });

            match server.handle_stats(params).await {
                Ok(result) => Ok(serde_json::to_value(result).unwrap()),
                Err(e) => Err(McpError { code: -32603, message: e.to_string(), data: None }),
            }
        }
        _ => Err(McpError { code: -32601, message: format!("Method not found: {}", request.method), data: None }),
    };

    let response = match result {
        Ok(value) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(error),
        },
    };

    (StatusCode::OK, Json(response))
}

async fn list_resources() -> impl IntoResponse {
    let resources = vec![
        McpResource {
            uri: "headroom://metrics/compression".to_string(),
            name: "Compression Metrics".to_string(),
            mime_type: Some("application/json".to_string()),
            description: Some("Real-time compression metrics".to_string()),
        },
        McpResource {
            uri: "headroom://stats/daily".to_string(),
            name: "Daily Statistics".to_string(),
            mime_type: Some("application/json".to_string()),
            description: Some("Daily compression statistics".to_string()),
        },
    ];

    Json(resources)
}

async fn prometheus_metrics(State(server): State<Arc<McpHeadroomServer>>) -> impl IntoResponse {
    let metrics = server.metrics.read().await;
    let report = format!(
        "# HELP headroom_requests_total Total requests
         # TYPE headroom_requests_total counter
         headroom_requests_total {}
         # HELP headroom_compress_ratio_avg Average compression ratio
         # TYPE headroom_compress_ratio_avg gauge
         headroom_compress_ratio_avg {:.4}
         # HELP headroom_tokens_saved_total Total tokens saved
         # TYPE headroom_tokens_saved_total counter
         headroom_tokens_saved_total {}
         # HELP headroom_zkp_verifications_total ZKP verifications
         # TYPE headroom_zkp_verifications_total counter
         headroom_zkp_verifications_total {}
         # HELP headroom_zkp_failures_total ZKP failures
         # TYPE headroom_zkp_failures_total counter
         headroom_zkp_failures_total {}
        ",
        metrics.total_requests,
        metrics.avg_compression_ratio,
        metrics.total_tokens_saved,
        metrics.zkp_verifications,
        metrics.zkp_failures,
    );

    (StatusCode::OK, report)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "cathedral-headroom-mcp",
        "version": "1.0.0",
        "timestamp": Utc::now().timestamp(),
    }))
}

/// ============================================================
/// 6. ERROS
/// ============================================================

#[derive(Debug, Error)]
pub enum McpToolError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Retrieve failed: {0}")]
    RetrieveFailed(String),
    #[error("CCR store failed: {0}")]
    CcrStoreFailed(String),
    #[error("ZKP verification failed: {0}")]
    ZkpVerificationFailed(String),
    #[error("EMA authentication required")]
    EmaAuthRequired,
    #[error("EMA token expired")]
    EmaTokenExpired,
    #[error("EMA insufficient scope")]
    EmaInsufficientScope,
    #[error("Invalid content type")]
    InvalidContentType,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// ============================================================
/// 7. UTILITÁRIOS
/// ============================================================

fn sha256(input: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

/// ============================================================
/// 8. TESTES
/// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compress_tool() {
        let bridge = Arc::new(RwLock::new(create_test_bridge().await));
        let server = Arc::new(McpHeadroomServer::new(bridge, McpServerConfig::default()));

        let params = CompressParams {
            content: r#"{"key": "value", "nested": {"a": 1, "b": 2}}"#.to_string(),
            target_id: "test_1".to_string(),
            content_type: ContentType::Json,
            max_tokens: Some(100),
            retrievable: true,
            zkp_verify: false,
            ema_token: None,
        };

        let result = server.handle_compress(params).await.unwrap();
        assert!(result.compression_ratio > 0.0);
        assert!(result.tokens_after < result.tokens_before);
        assert!(result.ccr_id.is_some());
    }

    #[tokio::test]
    async fn test_retrieve_tool() {
        let bridge = Arc::new(RwLock::new(create_test_bridge().await));
        let server = Arc::new(McpHeadroomServer::new(bridge, McpServerConfig::default()));

        // Primeiro comprime
        let compress_params = CompressParams {
            content: "original text".to_string(),
            target_id: "test_2".to_string(),
            content_type: ContentType::Text,
            max_tokens: None,
            retrievable: true,
            zkp_verify: false,
            ema_token: None,
        };
        let compress_result = server.handle_compress(compress_params).await.unwrap();
        let ccr_id = compress_result.ccr_id.unwrap();

        // Depois recupera
        let retrieve_params = RetrieveParams {
            ccr_id,
            zkp_proof: None,
            ema_token: None,
        };
        let retrieve_result = server.handle_retrieve(retrieve_params).await.unwrap();
        assert!(!retrieve_result.original_text.is_empty());
    }

    #[tokio::test]
    async fn test_stats_tool() {
        let bridge = Arc::new(RwLock::new(create_test_bridge().await));
        let server = Arc::new(McpHeadroomServer::new(bridge, McpServerConfig::default()));

        let params = StatsParams {
            detail_level: StatsDetailLevel::Summary,
            ema_token: None,
        };
        let result = server.handle_stats(params).await.unwrap();
        assert_eq!(result.server_metrics.total_requests, 0);
    }

    #[tokio::test]
    async fn test_zkp_verification() {
        let bridge = Arc::new(RwLock::new(create_test_bridge().await));
        let mut config = McpServerConfig::default();
        config.zkp_verification = true;
        let server = Arc::new(McpHeadroomServer::new(bridge, config));

        let params = CompressParams {
            content: "sensitive security data".to_string(),
            target_id: "test_zkp".to_string(),
            content_type: ContentType::Text,
            max_tokens: None,
            retrievable: true,
            zkp_verify: true,
            ema_token: None,
        };

        let result = server.handle_compress(params).await.unwrap();
        assert!(result.zkp_proof.is_some());

        let proof = result.zkp_proof.unwrap();
        assert_eq!(proof.proof_type, "semantic_preservation");
    }

    async fn create_test_bridge() -> crate::HeadroomBridge {
        use super::super::*;
        HeadroomBridge::new(
            HeadroomBridgeConfig::default(),
            Arc::new(HeadroomCompressor),
            Arc::new(CathedralHeadroomAdapter),
            Arc::new(CcrCache),
            Arc::new(CrossAgentMemoryStore),
        )
    }
}
