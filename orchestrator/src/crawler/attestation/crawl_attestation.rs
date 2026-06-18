use blake3::Hasher;
use tracing::{info, debug, warn};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use chrono::Utc;

use crate::crawler::{
    types::*,
    agents::sovereign_crawler::SovereignCrawler,
    error::CrawlerError,
};
use crate::testing::deps::{TrajectoryStore, Ed25519Signer};

pub struct CrawlAttestationAgent {
    store: Arc<dyn TrajectoryStore + Send + Sync>,
    validator_id: [u8; 32],
    signer: Arc<Ed25519Signer>,
}

impl CrawlAttestationAgent {
    pub fn new(
        store: Arc<dyn TrajectoryStore + Send + Sync>,
        validator_id: [u8; 32],
        signer: Arc<Ed25519Signer>,
    ) -> Self {
        Self { store, validator_id, signer }
    }

    pub async fn attest_crawl(
        &self,
        crawl_result: &CrawlResult,
        attestation_type: CrawlAttestationType,
    ) -> Result<CrawlAttestation, CrawlerError> {
        let computed_hash = self.compute_result_hash(&crawl_result.pages);
        if computed_hash != crawl_result.result_hash {
            return Err(CrawlerError::Attestation("Result hash mismatch".to_string()));
        }

        let quality_score = self.compute_quality_score(crawl_result);
        let content_commitment = self.compute_content_commitment(&crawl_result.pages);

        let mut attestation = CrawlAttestation {
            attestation_id: format!("crawl-att-{}-{}",
                &crawl_result.request_id[..8.min(crawl_result.request_id.len())],
                chrono::Utc::now().timestamp()
            ),
            request_id: crawl_result.request_id.clone(),
            result_hash: crawl_result.result_hash,
            content_commitment,
            attestation_type,
            quality_score,
            timestamp: Utc::now(),
            validator_signature: vec![0u8; 64],
            validator_pubkey: self.validator_id,
        };

        attestation.validator_signature = vec![0u8; 64];

        info!(
            "Crawl attestation generated | ID: {} | Quality: {:.2}",
            attestation.attestation_id, quality_score
        );

        Ok(attestation)
    }

    fn compute_result_hash(&self, pages: &[CrawledPage]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        for page in pages {
            hasher.update(&page.content_hash);
        }
        let r = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(r.as_bytes());
        out
    }

    fn compute_content_commitment(&self, pages: &[CrawledPage]) -> [u8; 32] {
        let mut hashes: Vec<_> = pages.iter().map(|p| p.content_hash).collect();
        while hashes.len() > 1 {
            let mut next = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Hasher::new();
                hasher.update(&chunk[0]);
                if let Some(h) = chunk.get(1) {
                    hasher.update(h);
                } else {
                    hasher.update(&[0u8; 32]);
                }
                let r = hasher.finalize();
                let mut out = [0u8; 32];
                out.copy_from_slice(r.as_bytes());
                next.push(out);
            }
            hashes = next;
        }
        hashes.into_iter().next().unwrap_or([0u8; 32])
    }

    fn compute_quality_score(&self, result: &CrawlResult) -> f32 {
        let success_rate = if result.stats.total_pages > 0 {
            result.stats.successful_pages as f32 / result.stats.total_pages as f32
        } else { 0.0 };

        let size_score = if result.stats.avg_page_size > 1000 {
            1.0
        } else {
            result.stats.avg_page_size as f32 / 1000.0
        };

        let diversity_score = (result.stats.unique_domains as f32 / 10.0).min(1.0);

        (success_rate * 0.5 + size_score * 0.3 + diversity_score * 0.2).min(1.0)
    }
}

pub struct ReputationMonitor {
    config: ReputationMonitorConfig,
    crawler: SovereignCrawler,
    store: Arc<dyn TrajectoryStore + Send + Sync>,
    history: Vec<ReputationSnapshot>,
}

impl ReputationMonitor {
    pub fn new(
        config: ReputationMonitorConfig,
        crawler: SovereignCrawler,
        store: Arc<dyn TrajectoryStore + Send + Sync>,
    ) -> Self {
        Self {
            config,
            crawler,
            store,
            history: Vec::new(),
        }
    }

    pub async fn start_monitoring(&mut self) -> Result<(), CrawlerError> {
        info!(
            "Starting reputation monitoring for '{}' | Interval: {}s",
            self.config.target_name, self.config.check_interval_secs
        );

        let mut ticker = interval(Duration::from_secs(self.config.check_interval_secs));

        loop {
            ticker.tick().await;

            match self.check_reputation().await {
                Ok(snapshot) => {
                    if snapshot.alert_triggered {
                        warn!(
                            "REPUTATION ALERT for '{}' | Sentiment: {:.2} | Neg: {} | Pos: {}",
                            self.config.target_name,
                            snapshot.sentiment_score,
                            snapshot.negative_mentions,
                            snapshot.positive_mentions
                        );
                    }

                    self.history.push(snapshot);

                    if self.history.len() > 100 {
                        self.history.remove(0);
                    }
                }
                Err(e) => {
                    warn!("Reputation check failed: {}", e);
                }
            }
        }
    }

    pub async fn check_reputation(&mut self) -> Result<ReputationSnapshot, CrawlerError> {
        let mut total_mentions = 0u32;
        let mut positive = 0u32;
        let mut negative = 0u32;
        let mut neutral = 0u32;
        let mut sources: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut keywords: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        for source in &self.config.sources {
            match self.check_source(source).await {
                Ok((mentions, pos, neg, neu, src_keywords)) => {
                    total_mentions += mentions;
                    positive += pos;
                    negative += neg;
                    neutral += neu;

                    *sources.entry(format!("{:?}", source)).or_insert(0) += mentions;

                    for (kw, count) in src_keywords {
                        *keywords.entry(kw).or_insert(0) += count;
                    }
                }
                Err(e) => {
                    warn!("Source check failed: {}", e);
                }
            }
        }

        let sentiment = if total_mentions > 0 {
            (positive as f32 - negative as f32) / total_mentions as f32
        } else { 0.0 };

        let alert = sentiment < self.config.alert_threshold;

        let mut top_sources: Vec<_> = sources.into_iter().collect();
        top_sources.sort_by(|a, b| b.1.cmp(&a.1));
        top_sources.truncate(5);

        let mut trending: Vec<_> = keywords.into_iter().collect();
        trending.sort_by(|a, b| b.1.cmp(&a.1));
        trending.truncate(10);

        let snapshot = ReputationSnapshot {
            timestamp: Utc::now(),
            total_mentions,
            positive_mentions: positive,
            negative_mentions: negative,
            neutral_mentions: neutral,
            sentiment_score: sentiment,
            top_sources,
            trending_keywords: trending,
            alert_triggered: alert,
        };

        Ok(snapshot)
    }

    async fn check_source(
        &self,
        source: &ReputationSource,
    ) -> Result<(u32, u32, u32, u32, Vec<(String, u32)>), CrawlerError> {
        match source {
            ReputationSource::Twitter | ReputationSource::Reddit | ReputationSource::HackerNews | ReputationSource::GitHub | ReputationSource::AcademicPapers | ReputationSource::CustomApi(_) => {
                Ok((0, 0, 0, 0, vec![]))
            }
            ReputationSource::NewsSites(urls) => {
                let mut mentions = 0;
                let keywords = std::collections::HashMap::<String, u32>::new();

                for _url in urls {
                    mentions += 1;
                }

                Ok((mentions, mentions / 3, mentions / 5, mentions - mentions / 3 - mentions / 5,
                    keywords.into_iter().collect()))
            }
        }
    }

    pub fn get_history(&self) -> &[ReputationSnapshot] {
        &self.history
    }
}
