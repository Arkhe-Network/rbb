use orchestrator::crawler::{
    agents::sovereign_crawler::{SovereignCrawler, SovereignCrawlerConfig},
    attestation::crawl_attestation::{CrawlAttestationAgent, ReputationMonitor},
    pipeline::rag_pipeline::{RagPipeline, RagPipelineConfig},
    types::{
        ContentType, CrawlFilters, CrawlPurpose, CrawlRequest, CrawlTarget,
        ReputationMonitorConfig, ReputationSource, RetentionPolicy,
    },
};
use orchestrator::testing::deps::{DummyTrajectoryStore, Ed25519Signer};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("🏛️ Cathedral ARKHE v30.3 — Sovereign Crawler Demo");
    info!("═══════════════════════════════════════════════════");

    let crawler_config = SovereignCrawlerConfig::default();
    let mut crawler = SovereignCrawler::new(crawler_config);

    let request = CrawlRequest {
        request_id: format!("crawl-demo-{}", chrono::Utc::now().timestamp()),
        target: CrawlTarget::SingleUrl("https://docs.firecrawl.dev/introduction".to_string()),
        content_types: vec![ContentType::Markdown, ContentType::Metadata],
        max_depth: 1,
        max_pages: 5,
        filters: CrawlFilters::default(),
        timestamp: chrono::Utc::now(),
        requesting_agent: [0xAAu8; 32],
        purpose: CrawlPurpose::RagContext,
        retention_policy: RetentionPolicy::Temporary(30),
    };

    info!(
        "Starting crawl | Target: {:?} | Purpose: {:?}",
        request.target, request.purpose
    );

    let crawl_result = crawler.crawl(request).await?;
    info!(
        "✅ Crawl completed | Pages: {} | Success: {} | Duration: {:.2}s",
        crawl_result.stats.total_pages,
        crawl_result.stats.successful_pages,
        crawl_result.stats.duration_seconds
    );

    info!("Processing into RAG pipeline...");
    let rag_config = RagPipelineConfig::default();
    let rag = RagPipeline::new(rag_config).await?;
    let documents = rag.process_crawl_result(&crawl_result).await?;

    info!(
        "✅ RAG pipeline completed | Documents: {} | Total chunks: {}",
        documents.len(),
        documents.iter().map(|d| d.chunks.len()).sum::<usize>()
    );

    info!("Generating crawl attestation...");
    let store = std::sync::Arc::new(DummyTrajectoryStore::new());
    let signer = std::sync::Arc::new(Ed25519Signer::new_random());
    let attestation_agent = CrawlAttestationAgent::new(store.clone(), [0xBBu8; 32], signer.clone());

    let attestation = attestation_agent
        .attest_crawl(
            &crawl_result,
            orchestrator::crawler::types::CrawlAttestationType::IntegrityValidated,
        )
        .await?;

    info!(
        "✅ Attestation generated | ID: {} | Quality: {:.2}",
        attestation.attestation_id, attestation.quality_score
    );

    info!("Retrieving RAG context for query...");
    let query = "What is Firecrawl and how does it work?";
    let context = rag.retrieve_context(query, 3).await?;

    info!("✅ Retrieved {} chunks for query", context.len());
    for (i, chunk) in context.iter().enumerate() {
        info!("  Chunk {}: {} chars", i, chunk.text.len());
    }

    info!("Starting reputation monitor...");
    let rep_config = ReputationMonitorConfig {
        target_name: "Cathedral ARKHE".to_string(),
        keywords: vec!["Cathedral ARKHE".to_string(), "AGI Soberana".to_string()],
        sources: vec![
            ReputationSource::HackerNews,
            ReputationSource::Twitter,
            ReputationSource::GitHub,
        ],
        check_interval_secs: 3600,
        alert_threshold: -0.3,
    };

    let mut monitor = ReputationMonitor::new(rep_config, crawler, store.clone());

    let snapshot = monitor.check_reputation().await?;
    info!(
        "Reputation snapshot | Mentions: {} | Sentiment: {:.2} | Alert: {}",
        snapshot.total_mentions,
        snapshot.sentiment_score,
        if snapshot.alert_triggered {
            "⚠️ YES"
        } else {
            "✅ NO"
        }
    );

    info!("═══════════════════════════════════════════════════");
    info!("🏁 Sovereign Crawler Demo completed!");
    info!("   Crawled: {} pages", crawl_result.stats.successful_pages);
    info!(
        "   RAG Docs: {} | Chunks: {}",
        documents.len(),
        documents.iter().map(|d| d.chunks.len()).sum::<usize>()
    );
    info!("   Attestation: {}", attestation.attestation_id);
    info!("   Context retrieved: {} chunks", context.len());
    info!("═══════════════════════════════════════════════════");

    Ok(())
}
