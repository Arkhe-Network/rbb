//! src/substrato_8000/cross_agent_memory.rs
//! Cross-Agent Memory — Memória compartilhada inteligente entre agents
//! Indexação semântica via zVEC, recuperação por similaridade
//!
//! Selo: CATHEDRAL-ARKHE-8000-CROSS-AGENT-MEMORY-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use thiserror::Error;

/// ============================================================
/// 1. TIPOS DE MEMÓRIA COMPARTILHADA
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryEntry {
    pub entry_id: String,
    pub agent_id: String,
    pub task_id: String,
    pub memory_type: SharedMemoryType,
    pub content: String,
    pub compressed_content: Option<String>,
    pub ccr_id: Option<String>,
    /// Embedding semântico (zVEC)
    pub embedding: Option<Vec<f32>>,
    /// Metadados contextuais
    pub metadata: MemoryMetadata,
    /// Timestamp de criação
    pub created_at: i64,
    /// Timestamp de último acesso
    pub last_accessed: i64,
    /// Contador de acessos
    pub access_count: u64,
    /// TTL (time-to-live) em segundos
    pub ttl_seconds: u64,
    /// Se foi deduplicado
    pub is_deduplicated: bool,
    /// IDs de entradas duplicadas (se deduplicado)
    pub duplicate_of: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharedMemoryType {
    /// Contexto de conversação
    ConversationContext,
    /// Resultado de ferramenta
    ToolResult,
    /// Estado de imersão IDT
    IdtBranchState,
    /// Memória episódica
    EpisodicMemory,
    /// Memória semântica
    SemanticMemory,
    /// Checkpoint de agent
    AgentCheckpoint,
    /// Mensagem de consenso multi-agent
    ConsensusMessage,
    /// Custom
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub priority: f64,
    pub relevance_score: f64,
    pub source_agent: String,
    pub target_agents: Vec<String>,
    pub tags: Vec<String>,
    pub compression_ratio: f64,
    pub original_size_bytes: usize,
    pub compressed_size_bytes: usize,
}

/// ============================================================
/// 2. CROSS-AGENT MEMORY STORE
/// ============================================================

pub struct CrossAgentMemoryStore {
    /// Armazenamento principal (entry_id → entry)
    store: Arc<RwLock<HashMap<String, SharedMemoryEntry>>>,
    /// Índice por agente (agent_id → [entry_ids])
    agent_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Índice por tarefa (task_id → [entry_ids])
    task_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Índice por tipo (memory_type → [entry_ids])
    type_index: Arc<RwLock<HashMap<SharedMemoryType, Vec<String>>>>,
    /// Índice semântico (zVEC stub — em produção, integrar com zVEC real)
    semantic_index: Arc<RwLock<SemanticIndex>>,
    /// Configuração
    config: CrossAgentMemoryConfig,
    /// Métricas
    metrics: Arc<RwLock<MemoryStoreMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossAgentMemoryConfig {
    /// Tamanho máximo do store (entradas)
    pub max_entries: usize,
    /// TTL padrão (segundos)
    pub default_ttl: u64,
    /// Threshold de similaridade para deduplicação (0.0-1.0)
    pub dedup_similarity_threshold: f64,
    /// Se habilita deduplicação automática
    pub auto_dedup: bool,
    /// Se habilita indexação semântica
    pub semantic_indexing: bool,
    /// Número de dimensões do embedding
    pub embedding_dimensions: usize,
    /// Fator de decaimento de relevância (por hora)
    pub relevance_decay_factor: f64,
}

impl Default for CrossAgentMemoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 100_000,
            default_ttl: 3600 * 24, // 24 horas
            dedup_similarity_threshold: 0.92,
            auto_dedup: true,
            semantic_indexing: true,
            embedding_dimensions: 384, // all-MiniLM-L6-v2
            relevance_decay_factor: 0.95,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStoreMetrics {
    pub total_entries: u64,
    pub total_deduplicated: u64,
    pub total_bytes_stored: u64,
    pub total_bytes_saved: u64,
    pub semantic_queries: u64,
    pub exact_queries: u64,
    pub avg_query_latency_ms: f64,
    pub cache_hit_rate: f64,
}

/// Índice semântico simplificado (stub para zVEC real)
#[derive(Debug, Clone)]
struct SemanticIndex {
    /// entry_id → embedding
    embeddings: HashMap<String, Vec<f32>>,
}

impl SemanticIndex {
    fn new() -> Self {
        Self { embeddings: HashMap::new() }
    }

    /// Computa similaridade de cosseno
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() { return 0.0; }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        (dot / (norm_a * norm_b)) as f64
    }

    /// Busca por similaridade
    fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(String, f64)> {
        let mut results: Vec<(String, f64)>: Vec<(String, f64)> = self.embeddings.iter()
            .map(|(id, emb)| (id.clone(), Self::cosine_similarity(query_embedding, emb)))
            .filter(|(_, sim)| *sim > 0.5)
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter().take(top_k).collect()
    }

    /// Adiciona embedding
    fn insert(&mut self, entry_id: String, embedding: Vec<f32>) {
        self.embeddings.insert(entry_id, embedding);
    }

    /// Remove embedding
    fn remove(&mut self, entry_id: &str) {
        self.embeddings.remove(entry_id);
    }
}

/// ============================================================
/// 3. IMPLEMENTAÇÃO
/// ============================================================

impl CrossAgentMemoryStore {
    pub fn new(config: CrossAgentMemoryConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            agent_index: Arc::new(RwLock::new(HashMap::new())),
            task_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            semantic_index: Arc::new(RwLock::new(SemanticIndex::new())),
            config,
            metrics: Arc::new(RwLock::new(MemoryStoreMetrics::default())),
        }
    }

    /// ============================================================
    /// 3.1 STORE
    /// ============================================================

    pub async fn store(
        &self,
        entry: SharedMemoryEntry,
    ) -> Result<String, MemoryStoreError> {
        let mut store = self.store.write().await;
        let mut agent_idx = self.agent_index.write().await;
        let mut task_idx = self.task_index.write().await;
        let mut type_idx = self.type_index.write().await;

        // Verifica limite de entradas
        if store.len() >= self.config.max_entries {
            self.evict_oldest(&mut store).await?;
        }

        let entry_id = entry.entry_id.clone();

        // Deduplicação
        let final_entry = if self.config.auto_dedup {
            self.deduplicate(&mut store, entry).await?
        } else {
            entry
        };

        // Indexa semanticamente
        if self.config.semantic_indexing && final_entry.embedding.is_some() {
            let mut sem_idx = self.semantic_index.write().await;
            if let Some(ref emb) = final_entry.embedding {
                sem_idx.insert(final_entry.entry_id.clone(), emb.clone());
            }
        }

        // Atualiza índices
        agent_idx.entry(final_entry.agent_id.clone())
            .or_default()
            .push(final_entry.entry_id.clone());

        task_idx.entry(final_entry.task_id.clone())
            .or_default()
            .push(final_entry.entry_id.clone());

        type_idx.entry(final_entry.memory_type.clone())
            .or_default()
            .push(final_entry.entry_id.clone());

        // Armazena
        let size = final_entry.content.len();
        let compressed_size = final_entry.compressed_content.as_ref().map(|c| c.len()).unwrap_or(size);

        store.insert(final_entry.entry_id.clone(), final_entry);

        // Atualiza métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_entries += 1;
            metrics.total_bytes_stored += compressed_size as u64;
            metrics.total_bytes_saved += (size - compressed_size) as u64;
        }

        Ok(entry_id)
    }

    /// ============================================================
    /// 3.2 RETRIEVE
    /// ============================================================

    pub async fn get(&self, entry_id: &str) -> Result<SharedMemoryEntry, MemoryStoreError> {
        let mut store = self.store.write().await;

        let mut entry = store.get_mut(entry_id)
            .ok_or(MemoryStoreError::EntryNotFound(entry_id.to_string()))?
            .clone();

        // Atualiza métricas de acesso
        entry.last_accessed = Utc::now().timestamp();
        entry.access_count += 1;

        // Reinsere com dados atualizados
        store.insert(entry_id.to_string(), entry.clone());

        // Atualiza métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.exact_queries += 1;
        }

        Ok(entry)
    }

    /// ============================================================
    /// 3.3 SEMANTIC SEARCH
    /// ============================================================

    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<SharedMemoryEntry>, MemoryStoreError> {
        let start = std::time::Instant::now();

        let sem_idx = self.semantic_index.read().await;
        let similarities = sem_idx.search(query_embedding, top_k * 2); // Busca extra para filtragem
        drop(sem_idx);

        let store = self.store.read().await;
        let mut results = vec![];

        for (entry_id, similarity) in similarities {
            if let Some(entry) = store.get(&entry_id) {
                // Aplica filtros
                if let Some(ref f) = filter {
                    if !self.matches_filter(entry, f) {
                        continue;
                    }
                }

                // Verifica TTL
                if self.is_expired(entry) {
                    continue;
                }

                let mut result = entry.clone();
                result.metadata.relevance_score = similarity as f64;
                results.push(result);

                if results.len() >= top_k {
                    break;
                }
            }
        }

        // Ordena por relevância
        results.sort_by(|a, b| {
            b.metadata.relevance_score.partial_cmp(&a.metadata.relevance_score).unwrap()
        });

        // Atualiza métricas
        {
            let mut metrics = self.metrics.write().await;
            metrics.semantic_queries += 1;
            let latency = start.elapsed().as_millis() as f64;
            metrics.avg_query_latency_ms =
                (metrics.avg_query_latency_ms * (metrics.semantic_queries - 1) as f64 + latency)
                / metrics.semantic_queries as f64;
        }

        Ok(results)
    }

    /// ============================================================
    /// 3.4 QUERY BY AGENT / TASK / TYPE
    /// ============================================================

    pub async fn query_by_agent(
        &self,
        agent_id: &str,
        memory_type: Option<SharedMemoryType>,
        limit: usize,
    ) -> Result<Vec<SharedMemoryEntry>, MemoryStoreError> {
        let agent_idx = self.agent_index.read().await;
        let store = self.store.read().await;

        let entry_ids = agent_idx.get(agent_id)
            .ok_or(MemoryStoreError::AgentNotFound(agent_id.to_string()))?;

        let mut results = vec![];
        for id in entry_ids {
            if let Some(entry) = store.get(id) {
                if let Some(ref mt) = memory_type {
                    if entry.memory_type != *mt {
                        continue;
                    }
                }
                if !self.is_expired(entry) {
                    results.push(entry.clone());
                }
                if results.len() >= limit {
                    break;
                }
            }
        }

        // Ordena por relevância decrescente
        results.sort_by(|a, b| {
            b.metadata.relevance_score.partial_cmp(&a.metadata.relevance_score).unwrap()
        });

        Ok(results)
    }

    pub async fn query_by_task(
        &self,
        task_id: &str,
        limit: usize,
    ) -> Result<Vec<SharedMemoryEntry>, MemoryStoreError> {
        let task_idx = self.task_index.read().await;
        let store = self.store.read().await;

        let entry_ids = task_idx.get(task_id)
            .ok_or(MemoryStoreError::TaskNotFound(task_id.to_string()))?;

        let mut results = vec![];
        for id in entry_ids.iter().take(limit) {
            if let Some(entry) = store.get(id) {
                if !self.is_expired(entry) {
                    results.push(entry.clone());
                }
            }
        }

        Ok(results)
    }

    /// ============================================================
    /// 3.5 DEDUPLICAÇÃO
    /// ============================================================

    async fn deduplicate(
        &self,
        store: &mut HashMap<String, SharedMemoryEntry>,
        entry: SharedMemoryEntry,
    ) -> Result<SharedMemoryEntry, MemoryStoreError> {
        if entry.embedding.is_none() {
            return Ok(entry);
        }

        let query_emb = entry.embedding.as_ref().unwrap();
        let sem_idx = self.semantic_index.read().await;
        let similar = sem_idx.search(query_emb, 5);
        drop(sem_idx);

        for (existing_id, similarity) in similar {
            if similarity >= self.config.dedup_similarity_threshold {
                // Encontrou duplicata
                if let Some(existing) = store.get_mut(&existing_id) {
                    existing.access_count += 1;
                    existing.last_accessed = Utc::now().timestamp();

                    let mut deduped = entry.clone();
                    deduped.is_deduplicated = true;
                    deduped.duplicate_of = Some(existing_id);

                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.total_deduplicated += 1;
                    }

                    return Ok(deduped);
                }
            }
        }

        Ok(entry)
    }

    /// ============================================================
    /// 3.6 EVICTION E TTL
    /// ============================================================

    async fn evict_oldest(
        &self,
        store: &mut HashMap<String, SharedMemoryEntry>,
    ) -> Result<(), MemoryStoreError> {
        // Evict least recently used
        let mut entries: Vec<_> = store.iter().collect();
        entries.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

        let to_evict = entries.len() / 10; // Evict 10%
        for (id, _) in entries.into_iter().take(to_evict) {
            store.remove(&id.to_string());

            let mut sem_idx = self.semantic_index.write().await;
            sem_idx.remove(&id.to_string());
        }

        Ok(())
    }

    fn is_expired(&self, entry: &SharedMemoryEntry) -> bool {
        let age = (Utc::now().timestamp() - entry.created_at) as u64;
        age > entry.ttl_seconds
    }

    fn matches_filter(&self, entry: &SharedMemoryEntry, filter: &MemoryFilter) -> bool {
        if let Some(ref agent) = filter.agent_id {
            if entry.agent_id != *agent { return false; }
        }
        if let Some(ref task) = filter.task_id {
            if entry.task_id != *task { return false; }
        }
        if let Some(ref mem_type) = filter.memory_type {
            if entry.memory_type != *mem_type { return false; }
        }
        if let Some(min_relevance) = filter.min_relevance {
            if entry.metadata.relevance_score < min_relevance { return false; }
        }
        if let Some(ref tags) = filter.tags {
            for tag in tags {
                if !entry.metadata.tags.contains(tag) { return false; }
            }
        }
        true
    }

    /// ============================================================
    /// 3.7 MÉTRICAS
    /// ============================================================

    pub async fn get_metrics(&self) -> MemoryStoreMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn get_stats(&self) -> MemoryStoreStats {
        let store = self.store.read().await;
        let agent_idx = self.agent_index.read().await;
        let type_idx = self.type_index.read().await;

        MemoryStoreStats {
            total_entries: store.len(),
            total_agents: agent_idx.len(),
            total_tasks: self.task_index.read().await.len(),
            memory_type_distribution: type_idx.iter()
                .map(|(k, v)| (format!("{:?}", k), v.len()))
                .collect(),
            avg_entry_size: if !store.is_empty() {
                store.values().map(|e| e.content.len()).sum::<usize>() / store.len()
            } else { 0 },
            dedup_ratio: if !store.is_empty() {
                store.values().filter(|e| e.is_deduplicated).count() as f64 / store.len() as f64
            } else { 0.0 },
        }
    }
}

/// ============================================================
/// 4. TIPOS AUXILIARES
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
    pub memory_type: Option<SharedMemoryType>,
    pub min_relevance: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStoreStats {
    pub total_entries: usize,
    pub total_agents: usize,
    pub total_tasks: usize,
    pub memory_type_distribution: HashMap<String, usize>,
    pub avg_entry_size: usize,
    pub dedup_ratio: f64,
}

#[derive(Debug, Error)]
pub enum MemoryStoreError {
    #[error("Entry not found: {0}")]
    EntryNotFound(String),
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Store full")]
    StoreFull,
    #[error("Embedding computation failed: {0}")]
    EmbeddingFailed(String),
    #[error("Deduplication failed: {0}")]
    DeduplicationFailed(String),
}

/// ============================================================
/// 5. TESTES
/// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: &str, agent: &str, task: &str, content: &str) -> SharedMemoryEntry {
        SharedMemoryEntry {
            entry_id: id.to_string(),
            agent_id: agent.to_string(),
            task_id: task.to_string(),
            memory_type: SharedMemoryType::ConversationContext,
            content: content.to_string(),
            compressed_content: None,
            ccr_id: None,
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4, 0.5]),
            metadata: MemoryMetadata {
                priority: 0.8,
                relevance_score: 0.9,
                source_agent: agent.to_string(),
                target_agents: vec![],
                tags: vec!["test".to_string()],
                compression_ratio: 0.0,
                original_size_bytes: content.len(),
                compressed_size_bytes: content.len(),
            },
            created_at: Utc::now().timestamp(),
            last_accessed: Utc::now().timestamp(),
            access_count: 0,
            ttl_seconds: 3600,
            is_deduplicated: false,
            duplicate_of: None,
        }
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = CrossAgentMemoryStore::new(CrossAgentMemoryConfig::default());
        let entry = create_test_entry("e1", "agent_a", "task_1", "Hello world");

        let id = store.store(entry.clone()).await.unwrap();
        assert_eq!(id, "e1");

        let retrieved = store.get("e1").await.unwrap();
        assert_eq!(retrieved.content, "Hello world");
        assert_eq!(retrieved.access_count, 1);
    }

    #[tokio::test]
    async fn test_semantic_search() {
        let store = CrossAgentMemoryStore::new(CrossAgentMemoryConfig::default());

        let e1 = create_test_entry("e1", "agent_a", "task_1", "Hello world");
        let e2 = create_test_entry("e2", "agent_b", "task_1", "Goodbye world");
        let e3 = create_test_entry("e3", "agent_c", "task_2", "Rust programming");

        store.store(e1).await.unwrap();
        store.store(e2).await.unwrap();
        store.store(e3).await.unwrap();

        let query = vec![0.1, 0.2, 0.3, 0.4, 0.5]; // Similar ao e1
        let results = store.search_similar(&query, 2, None).await.unwrap();

        assert!(!results.is_empty());
        // e1 deve estar no topo (embedding idêntico)
        assert_eq!(results[0].entry_id, "e1");
    }

    #[tokio::test]
    async fn test_query_by_agent() {
        let store = CrossAgentMemoryStore::new(CrossAgentMemoryConfig::default());

        store.store(create_test_entry("e1", "agent_a", "task_1", "A")).await.unwrap();
        store.store(create_test_entry("e2", "agent_a", "task_2", "B")).await.unwrap();
        store.store(create_test_entry("e3", "agent_b", "task_1", "C")).await.unwrap();

        let results = store.query_by_agent("agent_a", None, 10).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_deduplication() {
        let mut config = CrossAgentMemoryConfig::default();
        config.auto_dedup = true;
        config.dedup_similarity_threshold = 0.99; // Muito alto para teste

        let store = CrossAgentMemoryStore::new(config);

        let e1 = create_test_entry("e1", "agent_a", "task_1", "Duplicate content");
        let mut e2 = create_test_entry("e2", "agent_b", "task_1", "Duplicate content");
        e2.embedding = Some(vec![0.1, 0.2, 0.3, 0.4, 0.5]); // Mesmo embedding

        store.store(e1).await.unwrap();
        let id2 = store.store(e2).await.unwrap();

        let retrieved = store.get(&id2).await.unwrap();
        assert!(retrieved.is_deduplicated);
        assert_eq!(retrieved.duplicate_of, Some("e1".to_string()));
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let mut config = CrossAgentMemoryConfig::default();
        config.default_ttl = 1; // 1 segundo

        let store = CrossAgentMemoryStore::new(config);
        let entry = create_test_entry("e1", "agent_a", "task_1", "Expires soon");

        store.store(entry).await.unwrap();

        // Espera TTL expirar
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let results = store.query_by_agent("agent_a", None, 10).await.unwrap();
        assert!(results.is_empty()); // Expirado
    }
}
