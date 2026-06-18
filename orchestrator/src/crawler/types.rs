use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlRequest {
    pub request_id: String,
    pub target: CrawlTarget,
    pub content_types: Vec<ContentType>,
    pub max_depth: u8,
    pub max_pages: u32,
    pub filters: CrawlFilters,
    pub timestamp: DateTime<Utc>,
    pub requesting_agent: [u8; 32],
    pub purpose: CrawlPurpose,
    pub retention_policy: RetentionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrawlTarget {
    SingleUrl(String),
    Domain(String),
    UrlPattern(String),
    Sitemap(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Markdown,
    Html,
    StructuredJson,
    Screenshot,
    Metadata,
    Links,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlFilters {
    pub respect_robots_txt: bool,
    pub user_agent: String,
    pub delay_ms: u64,
    pub exclude_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
    pub max_page_size: usize,
}

impl Default for CrawlFilters {
    fn default() -> Self {
        Self {
            respect_robots_txt: true,
            user_agent: "Cathedral-Arkhe-Crawler/30.3 (Sovereign-Agent)".to_string(),
            delay_ms: 1000,
            exclude_patterns: vec![
                "*.pdf".to_string(),
                "*.zip".to_string(),
                "/admin/*".to_string(),
            ],
            include_patterns: vec![],
            max_page_size: 10_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrawlPurpose {
    TrainingData,
    RagContext,
    ReputationMonitoring,
    FactChecking,
    Research,
    ComplianceAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    Ephemeral,
    Temporary(u32),
    Permanent,
    UntilRevoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub request_id: String,
    pub pages: Vec<CrawledPage>,
    pub stats: CrawlStats,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub result_hash: [u8; 32],
    pub provenance: CrawlProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledPage {
    pub url: String,
    pub title: Option<String>,
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub structured_data: Option<serde_json::Value>,
    pub metadata: PageMetadata,
    pub links: Vec<String>,
    pub screenshot: Option<String>,
    pub crawled_at: DateTime<Utc>,
    pub content_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub language: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub author: Option<String>,
    pub publish_date: Option<DateTime<Utc>>,
    pub source_url: String,
    pub status_code: u16,
    pub content_type: String,
    pub robots_meta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlStats {
    pub total_pages: u32,
    pub successful_pages: u32,
    pub failed_pages: u32,
    pub total_bytes: u64,
    pub avg_page_size: u64,
    pub total_links_found: u32,
    pub unique_domains: u32,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlProvenance {
    pub crawler_agent_id: [u8; 32],
    pub crawler_version: String,
    pub crawl_method: String,
    pub consent_policy: String,
    pub ethical_log: Vec<String>,
    pub crawler_signature: Option<Vec<u8>>, // changed to Vec<u8> to avoid serde_big_array for 64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlAttestation {
    pub attestation_id: String,
    pub request_id: String,
    pub result_hash: [u8; 32],
    pub content_commitment: [u8; 32],
    pub attestation_type: CrawlAttestationType,
    pub quality_score: f32,
    pub timestamp: DateTime<Utc>,
    pub validator_signature: Vec<u8>, // changed to Vec<u8>
    pub validator_pubkey: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrawlAttestationType {
    IntegrityValidated,
    ComplianceValidated,
    ZkVerified,
    AuditValidated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagDocument {
    pub doc_id: String,
    pub source_url: String,
    pub title: String,
    pub chunks: Vec<RagChunk>,
    pub embedding: Option<Vec<f32>>,
    pub metadata: RagMetadata,
    pub provenance: CrawlProvenance,
    pub attestation: Option<CrawlAttestation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub chunk_index: u32,
    pub text: String,
    pub embedding: Option<Vec<f32>>,
    pub token_count: u32,
    pub overlap_chars: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagMetadata {
    pub source_url: String,
    pub crawl_timestamp: DateTime<Utc>,
    pub document_type: String,
    pub language: String,
    pub word_count: u32,
    pub confidence_score: f32,
    pub fact_check_status: Option<FactCheckStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactCheckStatus {
    pub checked: bool,
    pub verified_sources: Vec<String>,
    pub contradictions_found: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationMonitorConfig {
    pub target_name: String,
    pub keywords: Vec<String>,
    pub sources: Vec<ReputationSource>,
    pub check_interval_secs: u64,
    pub alert_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationSource {
    Twitter,
    Reddit,
    HackerNews,
    GitHub,
    NewsSites(Vec<String>),
    AcademicPapers,
    CustomApi(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_mentions: u32,
    pub positive_mentions: u32,
    pub negative_mentions: u32,
    pub neutral_mentions: u32,
    pub sentiment_score: f32,
    pub top_sources: Vec<(String, u32)>,
    pub trending_keywords: Vec<(String, u32)>,
    pub alert_triggered: bool,
}
