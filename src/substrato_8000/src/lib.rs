//! src/substrato_8000/mod.rs
//! Substrato 8000 — Headroom Bridge
//! Integração da camada de compressão Headroom ao ecossistema Cathedral ARKHE
//!
//! Selo: CATHEDRAL-ARKHE-8000-HEADROOM-BRIDGE-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

pub mod mcp_headroom_server;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// ============================================================
/// 1. CONFIGURAÇÃO DO SUBSTRATO 8000
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadroomBridgeConfig {
    /// Modo de operação: Library, Proxy, MCP
    pub mode: HeadroomMode,
    /// Compressão ativada por padrão
    pub compression_enabled: bool,
    /// Threshold de compressão (ratio mínimo para comprimir)
    pub compression_threshold: f64,
    /// Máximo de tokens antes de forçar compressão
    pub max_tokens_before_compress: usize,
    /// CCR (reversible compression) ativado
    pub ccr_enabled: bool,
    /// TTL do CCR cache (segundos)
    pub ccr_ttl_seconds: u64,
    /// Cross-agent memory ativado
    pub cross_agent_memory: bool,
    /// CacheAligner para KV cache optimization
    pub cache_aligner_enabled: bool,
    /// Métricas exportadas para Prometheus
    pub metrics_export: bool,
    /// ZKP proofs para compressão semântica
    pub zkp_verification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HeadroomMode {
    /// Biblioteca inline (compress() chamado explicitamente)
    Library,
    /// Proxy transparente (intercepta requests LLM)
    Proxy { port: u16 },
    /// MCP Server (headroom_compress, headroom_retrieve)
    McpServer,
    /// Modo híbrido (Library + MCP)
    Hybrid,
}

impl Default for HeadroomBridgeConfig {
    fn default() -> Self {
        Self {
            mode: HeadroomMode::Library,
            compression_enabled: true,
            compression_threshold: 0.3,
            max_tokens_before_compress: 4000,
            ccr_enabled: true,
            ccr_ttl_seconds: 3600,
            cross_agent_memory: true,
            cache_aligner_enabled: true,
            metrics_export: true,
            zkp_verification: false, // Desativado por padrão (custo computacional)
        }
    }
}

/// ============================================================
/// 2. HEADROOM BRIDGE — Interface Principal
/// ============================================================

pub struct HeadroomBridge {
    config: HeadroomBridgeConfig,
    compressor: Arc<HeadroomCompressor>,
    adapter: Arc<CathedralHeadroomAdapter>,
    metrics: Arc<RwLock<HeadroomMetricsCollector>>,
    ccr_cache: Arc<CcrCache>,
    cross_agent_store: Arc<CrossAgentMemoryStore>,
}

impl HeadroomBridge {
    pub fn new(
        config: HeadroomBridgeConfig,
        compressor: Arc<HeadroomCompressor>,
        adapter: Arc<CathedralHeadroomAdapter>,
        ccr_cache: Arc<CcrCache>,
        cross_agent_store: Arc<CrossAgentMemoryStore>,
    ) -> Self {
        Self {
            config,
            compressor,
            adapter,
            metrics: Arc::new(RwLock::new(HeadroomMetricsCollector::new())),
            ccr_cache,
            cross_agent_store,
        }
    }

    /// ============================================================
    /// 2.1 COMPRESSÃO DE CONTEXTO IDT
    /// ============================================================

    /// Comprime contexto de Immersion-Driven Thinking antes de enviar ao LLM
    pub async fn compress_idt_context(
        &self,
        session_id: &str,
        branches: &[serde_json::Value],
        anchor_objective: &str,
    ) -> Result<CompressedIdtContext, HeadroomBridgeError> {
        if !self.config.compression_enabled {
            return Ok(CompressedIdtContext {
                text: serde_json::to_string(branches).unwrap(),
                compression_ratio: 0.0,
                tokens_saved: 0,
                ccr_id: None,
                was_compressed: false,
            });
        }

        let start_time = std::time::Instant::now();

        // 1. Serializa branches
        let raw_json = serde_json::to_string(branches).unwrap();
        let raw_tokens = raw_json.len() / 4; // Estimativa aproximada

        // 2. Verifica se precisa comprimir
        if raw_tokens < self.config.max_tokens_before_compress {
            tracing::info!("📦 Contexto IDT pequeno ({} tokens), pulando compressão", raw_tokens);
            return Ok(CompressedIdtContext {
                text: raw_json,
                compression_ratio: 0.0,
                tokens_saved: 0,
                ccr_id: None,
                was_compressed: false,
            });
        }

        // 3. Comprime via Headroom
        let compressed = self.compressor.compress(
            &raw_json,
            CompressionTarget::IdtContext {
                session_id: session_id.to_string(),
                branch_count: branches.len(),
            }
        ).await.unwrap();

        // 4. Armazena original no CCR se habilitado
        let ccr_id = if self.config.ccr_enabled {
            Some(self.ccr_cache.store(
                session_id,
                &raw_json,
                self.config.ccr_ttl_seconds,
            ).await.unwrap())
        } else {
            None
        };

        // 5. Registra métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_compression(
                "idt_context",
                raw_tokens,
                compressed.tokens_after,
                start_time.elapsed().as_millis() as u64,
            );
        }

        // 6. Log no WormGraph via adapter
        self.adapter.log_compression_event(
            session_id,
            "idt_context",
            raw_tokens,
            compressed.tokens_after,
            ccr_id.as_deref(),
        ).await.unwrap();

        tracing::info!(
            "🗜️  IDT context compressed: {} → {} tokens ({}% reduction)",
            raw_tokens, compressed.tokens_after,
            ((1.0 - compressed.tokens_after as f64 / raw_tokens as f64) * 100.0) as u32
        );

        Ok(CompressedIdtContext {
            text: compressed.text,
            compression_ratio: 1.0 - (compressed.tokens_after as f64 / raw_tokens as f64),
            tokens_saved: raw_tokens - compressed.tokens_after,
            ccr_id,
            was_compressed: true,
        })
    }

    /// ============================================================
    /// 2.2 RETRIEVE CCR
    /// ============================================================

    /// Recupera contexto original do CCR quando LLM solicita
    pub async fn retrieve_ccr(
        &self,
        ccr_id: &str,
    ) -> Result<String, HeadroomBridgeError> {
        if !self.config.ccr_enabled {
            return Err(HeadroomBridgeError::CcrDisabled);
        }

        let original = self.ccr_cache.retrieve(ccr_id).await
            .map_err(|e| HeadroomBridgeError::CcrRetrieveFailed(e.to_string()))?;

        tracing::info!("📤 CCR retrieve: {} ({} bytes)", ccr_id, original.len());

        // Registra métricas de retrieve
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_ccr_retrieve(ccr_id);
        }

        Ok(original)
    }

    /// ============================================================
    /// 2.3 CROSS-AGENT MEMORY
    /// ============================================================

    /// Armazena contexto comprimido para compartilhamento entre agents
    pub async fn store_cross_agent_context(
        &self,
        agent_id: &str,
        task_id: &str,
        context: &CompressedIdtContext,
    ) -> Result<String, HeadroomBridgeError> {
        if !self.config.cross_agent_memory {
            return Err(HeadroomBridgeError::CrossAgentMemoryDisabled);
        }

        let shared_context = SharedContext {
            agent_id: agent_id.to_string(),
            task_id: task_id.to_string(),
            compressed_text: context.text.clone(),
            ccr_id: context.ccr_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            compression_ratio: context.compression_ratio,
        };

        let context_id = self.cross_agent_store.put(shared_context).await.unwrap();

        tracing::info!(
            "🔄 Cross-agent context stored: {} (agent={}, task={})",
            context_id, agent_id, task_id
        );

        Ok(context_id)
    }

    /// Recupera contexto compartilhado de outro agent
    pub async fn get_cross_agent_context(
        &self,
        context_id: &str,
    ) -> Result<SharedContext, HeadroomBridgeError> {
        let context = self.cross_agent_store.get(context_id).await
            .map_err(|e| HeadroomBridgeError::CrossAgentRetrieveFailed(e.to_string()))?;

        Ok(context)
    }

    /// ============================================================
    /// 2.4 CACHE ALIGNER
    /// ============================================================

    /// Otimiza prefixos para KV cache hit no provider
    pub async fn align_cache(
        &self,
        messages: &[LlmMessage],
    ) -> Result<Vec<LlmMessage>, HeadroomBridgeError> {
        if !self.config.cache_aligner_enabled {
            return Ok(messages.to_vec());
        }

        let aligned = self.compressor.align_cache_prefixes(messages).await.unwrap();

        tracing::debug!("🎯 Cache aligned: {} messages → {} stable prefixes",
            messages.len(), aligned.len());

        Ok(aligned)
    }

    /// ============================================================
    /// 2.5 MÉTRICAS E RELATÓRIOS
    /// ============================================================

    pub async fn get_metrics_report(&self) -> HeadroomMetricsReport {
        let metrics = self.metrics.read().await;
        metrics.generate_report()
    }

    pub async fn export_prometheus_metrics(&self) -> String {
        let metrics = self.metrics.read().await;
        metrics.to_prometheus_format()
    }
}

/// ============================================================
/// 3. TIPOS DE DOMÍNIO
/// ============================================================

#[derive(Debug, Clone)]
pub struct CompressedIdtContext {
    pub text: String,
    pub compression_ratio: f64,
    pub tokens_saved: usize,
    pub ccr_id: Option<String>,
    pub was_compressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    pub agent_id: String,
    pub task_id: String,
    pub compressed_text: String,
    pub ccr_id: Option<String>,
    pub timestamp: u64,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub text: String,
    pub tokens_after: usize,
    pub ratio: f64,
}

#[derive(Debug, Clone)]
pub enum CompressionTarget {
    IdtContext { session_id: String, branch_count: usize },
    AgentContext { model: String, max_tokens: usize },
    MemoryIndex,
    ToolOutput { tool_name: String },
    RagChunk { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadroomMetricsReport {
    pub total_compressions: u64,
    pub total_tokens_saved: u64,
    pub avg_compression_ratio: f64,
    pub ccr_retrieve_count: u64,
    pub cross_agent_stores: u64,
    pub cache_hit_rate: f64,
    pub top_compressors: Vec<(String, f64)>,
}

/// ============================================================
/// 4. ERROS
/// ============================================================

#[derive(Debug, Error)]
pub enum HeadroomBridgeError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("CCR is disabled")]
    CcrDisabled,
    #[error("CCR retrieve failed: {0}")]
    CcrRetrieveFailed(String),
    #[error("Cross-agent memory is disabled")]
    CrossAgentMemoryDisabled,
    #[error("Cross-agent retrieve failed: {0}")]
    CrossAgentRetrieveFailed(String),
    #[error("Cache align failed: {0}")]
    CacheAlignFailed(String),
    #[error("Adapter error: {0}")]
    AdapterError(String),
}

// Placeholder types para compilação
#[derive(Debug, Clone)] pub struct HeadroomCompressor;
impl HeadroomCompressor {
    pub async fn compress(&self, _input: &str, _target: CompressionTarget) -> Result<CompressionResult, String> {
        Ok(CompressionResult { text: _input.to_string(), tokens_after: _input.len() / 4, ratio: 0.0 })
    }
    pub async fn align_cache_prefixes(&self, messages: &[LlmMessage]) -> Result<Vec<LlmMessage>, String> {
        Ok(messages.to_vec())
    }
}

#[derive(Debug, Clone)] pub struct CathedralHeadroomAdapter;
impl CathedralHeadroomAdapter {
    pub async fn log_compression_event(
        &self, _session: &str, _type: &str, _before: usize, _after: usize, _ccr: Option<&str>
    ) -> Result<(), HeadroomBridgeError> {
        Ok(())
    }
}

#[derive(Debug, Clone)] pub struct CcrCache;
impl CcrCache {
    pub async fn store(&self, _key: &str, _value: &str, _ttl: u64) -> Result<String, String> {
        Ok(format!("ccr_{}", _key))
    }
    pub async fn retrieve(&self, _id: &str) -> Result<String, String> {
        Ok("original".to_string())
    }
}

#[derive(Debug, Clone)] pub struct CrossAgentMemoryStore;
impl CrossAgentMemoryStore {
    pub async fn put(&self, _ctx: SharedContext) -> Result<String, String> {
        Ok("ctx_123".to_string())
    }
    pub async fn get(&self, _id: &str) -> Result<SharedContext, String> {
        Err("not found".to_string())
    }
}

#[derive(Debug, Clone, Default)] pub struct HeadroomMetricsCollector;
impl HeadroomMetricsCollector {
    pub fn new() -> Self { Self }
    pub fn record_compression(&mut self, _type: &str, _before: usize, _after: usize, _latency_ms: u64) {}
    pub fn record_ccr_retrieve(&mut self, _id: &str) {}
    pub fn generate_report(&self) -> HeadroomMetricsReport {
        HeadroomMetricsReport {
            total_compressions: 0,
            total_tokens_saved: 0,
            avg_compression_ratio: 0.0,
            ccr_retrieve_count: 0,
            cross_agent_stores: 0,
            cache_hit_rate: 0.0,
            top_compressors: vec![],
        }
    }
    pub fn to_prometheus_format(&self) -> String {
        "# headroom metrics".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compression_threshold() {
        let config = HeadroomBridgeConfig {
            max_tokens_before_compress: 100,
            ..Default::default()
        };

        let bridge = create_test_bridge(config).await;

        // Contexto pequeno → não comprime
        let small_branches = vec![];
        let result = bridge.compress_idt_context("test", &small_branches, "obj").await.unwrap();
        assert!(!result.was_compressed);
    }

    #[tokio::test]
    async fn test_ccr_roundtrip() {
        let config = HeadroomBridgeConfig {
            ccr_enabled: true,
            ..Default::default()
        };

        let bridge = create_test_bridge(config).await;

        let branches = vec![];
        let compressed = bridge.compress_idt_context("test", &branches, "obj").await.unwrap();

        if let Some(ccr_id) = compressed.ccr_id {
            let retrieved = bridge.retrieve_ccr(&ccr_id).await.unwrap();
            assert!(!retrieved.is_empty());
        }
    }

    #[tokio::test]
    async fn test_cross_agent_memory() {
        let config = HeadroomBridgeConfig {
            cross_agent_memory: true,
            ..Default::default()
        };

        let bridge = create_test_bridge(config).await;

        let ctx = CompressedIdtContext {
            text: "compressed".to_string(),
            compression_ratio: 0.5,
            tokens_saved: 100,
            ccr_id: Some("ccr_1".to_string()),
            was_compressed: true,
        };

        let id = bridge.store_cross_agent_context("agent_1", "task_1", &ctx).await.unwrap();
        assert!(!id.is_empty());
    }

    async fn create_test_bridge(config: HeadroomBridgeConfig) -> HeadroomBridge {
        HeadroomBridge::new(
            config,
            Arc::new(HeadroomCompressor),
            Arc::new(CathedralHeadroomAdapter),
            Arc::new(CcrCache),
            Arc::new(CrossAgentMemoryStore),
        )
    }
}
