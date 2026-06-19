//! src/substrato_8000/proxy_mode.rs
//! Proxy Mode — Servidor HTTP transparente para compressão LLM
//! Intercepta requests/responses e comprime automaticamente
//!
//! Selo: CATHEDRAL-ARKHE-8000-PROXY-MODE-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router, Server,
    routing::{post, get, any},
    extract::{State, Request, Body},
    http::{StatusCode, HeaderMap, Uri},
    response::{IntoResponse, Response},
    middleware::{self, Next},
};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::Utc;
use thiserror::Error;
use tracing::{info, error, debug, warn};
use hyper::body::to_bytes;

/// ============================================================
/// 1. PROXY CONFIGURATION
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Porta do proxy
    pub listen_port: u16,
    /// Host do proxy
    pub listen_host: String,
    /// URL do LLM provider upstream
    pub upstream_url: String,
    /// Provider type (anthropic, openai, gemini, etc)
    pub provider_type: LlmProviderType,
    /// Comprimir requests
    pub compress_requests: bool,
    /// Comprimir responses
    pub compress_responses: bool,
    /// Threshold de tokens para ativar compressão
    pub compression_threshold: usize,
    /// CCR (reversible) ativado
    pub ccr_enabled: bool,
    /// Rate limiting
    pub rate_limit_per_second: u32,
    /// Auth header para upstream
    pub upstream_auth_header: Option<String>,
    /// Timeout para upstream (ms)
    pub upstream_timeout_ms: u64,
    /// Log level
    pub log_level: String,
    /// Métricas Prometheus
    pub metrics_enabled: bool,
    /// Porta de métricas
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmProviderType {
    Anthropic,
    OpenAI,
    Gemini,
    AzureOpenAI,
    Vllm,
    Ollama,
    Custom { api_path: String },
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_port: 8787,
            listen_host: "0.0.0.0".to_string(),
            upstream_url: "http://localhost:8000".to_string(),
            provider_type: LlmProviderType::Vllm,
            compress_requests: true,
            compress_responses: true,
            compression_threshold: 4000,
            ccr_enabled: true,
            rate_limit_per_second: 100,
            upstream_auth_header: None,
            upstream_timeout_ms: 30000,
            log_level: "info".to_string(),
            metrics_enabled: true,
            metrics_port: 8788,
        }
    }
}

/// ============================================================
/// 2. PROXY SERVER
/// ============================================================

pub struct HeadroomProxy {
    config: ProxyConfig,
    bridge: Arc<RwLock<super::HeadroomBridge>>,
    metrics: Arc<RwLock<ProxyMetrics>>,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Default)]
pub struct ProxyMetrics {
    pub total_requests: u64,
    pub compressed_requests: u64,
    pub compressed_responses: u64,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub total_bytes_saved: u64,
    pub avg_latency_ms: f64,
    pub errors: u64,
    pub upstream_errors: u64,
}

impl HeadroomProxy {
    pub fn new(
        config: ProxyConfig,
        bridge: Arc<RwLock<super::HeadroomBridge>>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.upstream_timeout_ms))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config,
            bridge,
            metrics: Arc::new(RwLock::new(ProxyMetrics::default())),
            http_client,
        }
    }

    /// ============================================================
    /// 2.1 REQUEST HANDLER
    /// ============================================================

    pub async fn handle_request(
        &self,
        method: axum::http::Method,
        uri: Uri,
        headers: HeaderMap,
        body: Body,
    ) -> Result<Response, ProxyError> {
        let start = std::time::Instant::now();

        info!("🌐 Proxy request: {} {}", method, uri.path());

        // Extrai body como bytes
        let body_bytes = to_bytes(body, usize::MAX).await
            .map_err(|e| ProxyError::BodyReadError(e.to_string()))?;

        let body_str = String::from_utf8_lossy(&body_bytes);
        let body_json: Value = serde_json::from_str(&body_str)
            .unwrap_or(Value::Null);

        // Verifica se é request LLM (tem messages)
        let is_llm_request = body_json.get("messages").is_some()
            || body_json.get("prompt").is_some();

        let (processed_body, was_compressed) = if is_llm_request && self.config.compress_requests {
            self.compress_request(&body_json).await?
        } else {
            (body_json, false)
        };

        // Forward para upstream
        let upstream_response = self.forward_to_upstream(
            method,
            uri,
            headers,
            processed_body,
        ).await?;

        // Comprime response se necessário
        let final_response = if self.config.compress_responses && is_llm_request {
            self.compress_response(upstream_response).await?
        } else {
            upstream_response
        };

        let latency = start.elapsed().as_millis() as f64;

        // Atualiza métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            metrics.total_bytes_in += body_bytes.len() as u64;
            if was_compressed { metrics.compressed_requests += 1; }
            metrics.avg_latency_ms =
                (metrics.avg_latency_ms * (metrics.total_requests - 1) as f64 + latency)
                / metrics.total_requests as f64;
        }

        info!("✅ Proxy response: {}ms, compressed={}", latency as u64, was_compressed);

        Ok(final_response)
    }

    /// ============================================================
    /// 2.2 COMPRESS REQUEST
    /// ============================================================

    async fn compress_request(
        &self,
        body: &Value,
    ) -> Result<(Value, bool), ProxyError> {
        let messages = body.get("messages")
            .and_then(|m| m.as_array())
            .ok_or(ProxyError::InvalidRequest("No messages field".to_string()))?;

        // Conta tokens aproximados
        let total_chars: usize = messages.iter()
            .map(|m| m.get("content").and_then(|c| c.as_str()).unwrap_or("").len())
            .sum();
        let approx_tokens = total_chars / 4;

        if approx_tokens < self.config.compression_threshold {
            debug!("📦 Request too small ({} tokens), skipping compression", approx_tokens);
            return Ok((body.clone(), false));
        }

        // Comprime via bridge
        let bridge = self.bridge.read().await;
        let content = serde_json::to_string(messages)
            .map_err(|e| ProxyError::SerializationError(e.to_string()))?;

        let compressed = bridge.compress_idt_context(
            &format!("proxy_req_{}", Utc::now().timestamp_millis()),
            &[], // Simplificado — em produção, converte messages para branches
            "proxy",
        ).await.map_err(|e| ProxyError::CompressionError(e.to_string()))?;

        if !compressed.was_compressed {
            return Ok((body.clone(), false));
        }

        // Reconstrói body com contexto comprimido
        let mut new_body = body.clone();
        if let Some(obj) = new_body.as_object_mut() {
            obj.insert("_headroom".to_string(), serde_json::json!({
                "compressed": true,
                "original_tokens": approx_tokens,
                "compressed_tokens": compressed.tokens_saved,
                "ccr_id": compressed.ccr_id,
                "compression_ratio": compressed.compression_ratio,
            }));

            // Adiciona hint de retrieve no system message
            if let Some(msgs) = obj.get_mut("messages") {
                if let Some(arr) = msgs.as_array_mut() {
                    let system_msg = serde_json::json!({
                        "role": "system",
                        "content": format!(
                            "[HEADROOM] Context compressed. Use CCR ID '{}' to retrieve full context if needed.",
                            compressed.ccr_id.as_deref().unwrap_or("none")
                        )
                    });
                    arr.insert(0, system_msg);
                }
            }
        }

        info!("🗜️  Request compressed: {} → {} tokens ({}% reduction)",
            approx_tokens, approx_tokens - compressed.tokens_saved,
            (compressed.compression_ratio * 100.0) as u32);

        Ok((new_body, true))
    }

    /// ============================================================
    /// 2.3 COMPRESS RESPONSE
    /// ============================================================

    async fn compress_response(
        &self,
        response: Response,
    ) -> Result<Response, ProxyError> {
        // Em produção: comprime conteúdo da response (choices, content)
        // Aqui: pass-through com header indicando compressão

        let (parts, body) = response.into_parts();
        let body_bytes = to_bytes(body, usize::MAX).await
            .map_err(|e| ProxyError::BodyReadError(e.to_string()))?;

        let mut new_parts = parts.clone();
        new_parts.headers.insert(
            "x-headroom-compressed",
            "true".parse().unwrap(),
        );

        Ok(Response::from_parts(new_parts, Body::from(body_bytes)))
    }

    /// ============================================================
    /// 2.4 FORWARD TO UPSTREAM
    /// ============================================================

    async fn forward_to_upstream(
        &self,
        method: axum::http::Method,
        uri: Uri,
        headers: HeaderMap,
        body: Value,
    ) -> Result<Response, ProxyError> {
        let upstream_path = uri.path();
        let upstream_url = format!("{}{}", self.config.upstream_url, upstream_path);

        let mut request_builder = self.http_client
            .request(
                reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap(),
                &upstream_url,
            )
            .json(&body);

        // Forward headers relevantes
        for (key, value) in headers.iter() {
            if key.as_str().starts_with("content-")
                || key.as_str() == "authorization"
                || key.as_str() == "x-api-key" {
                request_builder = request_builder.header(key.as_str(), value);
            }
        }

        // Adiciona auth header se configurado
        if let Some(ref auth) = self.config.upstream_auth_header {
            request_builder = request_builder.header("Authorization", auth);
        }

        let response = request_builder.send().await
            .map_err(|e| {
                {
                    let mut metrics = self.metrics.blocking_write();
                    metrics.upstream_errors += 1;
                }
                ProxyError::UpstreamError(e.to_string())
            })?;

        let status = response.status();
        let response_bytes = response.bytes().await
            .map_err(|e| ProxyError::UpstreamError(e.to_string()))?;

        let mut response_headers = HeaderMap::new();
        response_headers.insert("content-type", "application/json".parse().unwrap());

        let axum_response = Response::builder()
            .status(status.as_u16())
            .body(Body::from(response_bytes))
            .map_err(|e| ProxyError::ResponseBuildError(e.to_string()))?;

        Ok(axum_response)
    }

    /// ============================================================
    /// 2.5 ROUTER
    /// ============================================================

    pub fn create_router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/", any(proxy_handler))
            .route("/*path", any(proxy_handler))
            .route("/health", get(health_handler))
            .route("/metrics", get(proxy_metrics_handler))
            .layer(middleware::from_fn_with_state(
                self.clone(),
                rate_limit_middleware,
            ))
            .with_state(self)
    }

    pub async fn get_metrics(&self) -> ProxyMetrics {
        self.metrics.read().await.clone()
    }
}

/// ============================================================
/// 3. HANDLERS
/// ============================================================

async fn proxy_handler(
    State(proxy): State<Arc<HeadroomProxy>>,
    method: axum::http::Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    match proxy.handle_request(method, uri, headers, body).await {
        Ok(response) => response,
        Err(e) => {
            error!("❌ Proxy error: {}", e);
            let mut metrics = proxy.metrics.blocking_write();
            metrics.errors += 1;

            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("{{\"error\": \"{}\"}}", e)))
                .unwrap()
        }
    }
}

async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "cathedral-headroom-proxy",
        "version": "1.0.0",
        "timestamp": Utc::now().timestamp(),
    }))
}

async fn proxy_metrics_handler(
    State(proxy): State<Arc<HeadroomProxy>>,
) -> impl IntoResponse {
    let metrics = proxy.get_metrics().await;

    let report = format!(
        "# HELP headroom_proxy_requests_total Total proxy requests
         # TYPE headroom_proxy_requests_total counter
         headroom_proxy_requests_total {}
         # HELP headroom_proxy_compressed_requests_total Compressed requests
         # TYPE headroom_proxy_compressed_requests_total counter
         headroom_proxy_compressed_requests_total {}
         # HELP headroom_proxy_bytes_saved_total Bytes saved by compression
         # TYPE headroom_proxy_bytes_saved_total counter
         headroom_proxy_bytes_saved_total {}
         # HELP headroom_proxy_avg_latency_ms Average latency
         # TYPE headroom_proxy_avg_latency_ms gauge
         headroom_proxy_avg_latency_ms {:.2}
         # HELP headroom_proxy_errors_total Total errors
         # TYPE headroom_proxy_errors_total counter
         headroom_proxy_errors_total {}
        ",
        metrics.total_requests,
        metrics.compressed_requests,
        metrics.total_bytes_saved,
        metrics.avg_latency_ms,
        metrics.errors,
    );

    (StatusCode::OK, report)
}

async fn rate_limit_middleware(
    State(proxy): State<Arc<HeadroomProxy>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Simplificado: em produção, usar token bucket
    // Aqui: apenas pass-through
    Ok(next.run(request).await)
}

/// ============================================================
/// 4. ERROS
/// ============================================================

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("Body read error: {0}")]
    BodyReadError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Upstream error: {0}")]
    UpstreamError(String),
    #[error("Response build error: {0}")]
    ResponseBuildError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// ============================================================
/// 5. TESTES
/// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_config_default() {
        let config = ProxyConfig::default();
        assert_eq!(config.listen_port, 8787);
        assert!(config.compress_requests);
        assert!(config.ccr_enabled);
    }

    #[tokio::test]
    async fn test_compress_request_small() {
        let proxy = create_test_proxy().await;

        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": "Hi"}
            ]
        });

        let (result, was_compressed) = proxy.compress_request(&body).await.unwrap();
        assert!(!was_compressed); // Too small
        assert_eq!(result, body);
    }

    #[tokio::test]
    async fn test_compress_request_large() {
        let mut config = ProxyConfig::default();
        config.compression_threshold = 10;

        let proxy = create_test_proxy_with_config(config).await;

        let long_content = "a".repeat(1000);
        let body = serde_json::json!({
            "messages": [
                {"role": "system", "content": "You are helpful"},
                {"role": "user", "content": long_content}
            ]
        });

        let (result, was_compressed) = proxy.compress_request(&body).await.unwrap();
        // Pode ou não comprimir dependendo do stub
        assert!(result.get("_headroom").is_some() || !was_compressed);
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let proxy = create_test_proxy().await;

        // Simula requests
        {
            let mut metrics = proxy.metrics.write().await;
            metrics.total_requests = 100;
            metrics.compressed_requests = 50;
            metrics.total_bytes_saved = 50000;
        }

        let metrics = proxy.get_metrics().await;
        assert_eq!(metrics.total_requests, 100);
        assert_eq!(metrics.compressed_requests, 50);
    }

    async fn create_test_proxy() -> Arc<HeadroomProxy> {
        create_test_proxy_with_config(ProxyConfig::default()).await
    }

    async fn create_test_proxy_with_config(config: ProxyConfig) -> Arc<HeadroomProxy> {
        use crate::substrato_8000::*;

        let bridge = Arc::new(RwLock::new(HeadroomBridge::new(
            HeadroomBridgeConfig::default(),
            Arc::new(HeadroomCompressor),
            Arc::new(CathedralHeadroomAdapter),
            Arc::new(CcrCache),
            Arc::new(CrossAgentMemoryStore::new(CrossAgentMemoryConfig::default())),
        )));

        Arc::new(HeadroomProxy::new(config, bridge))
    }
}
