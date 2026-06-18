use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::crawler::{error::CrawlerError, types::*};

#[derive(Debug, Clone)]
pub struct RagPipelineConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub embedding_model: String,
    pub embedding_dim: usize,
    pub embedding_batch_size: usize,
    pub min_confidence_score: f32,
    pub max_chunks_per_doc: u32,
    pub zvec_endpoint: String,
}

impl Default for RagPipelineConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            embedding_model: "text-embedding-3-small".to_string(),
            embedding_dim: 1536,
            embedding_batch_size: 32,
            min_confidence_score: 0.7,
            max_chunks_per_doc: 100,
            zvec_endpoint: "http://localhost:8080".to_string(),
        }
    }
}

pub struct RagPipeline {
    config: RagPipelineConfig,
}

impl RagPipeline {
    pub async fn new(config: RagPipelineConfig) -> Result<Self, CrawlerError> {
        Ok(Self { config })
    }

    pub async fn process_crawl_result(
        &self,
        crawl_result: &CrawlResult,
    ) -> Result<Vec<RagDocument>, CrawlerError> {
        info!(
            "Processing crawl result into RAG | Pages: {} | Request: {}",
            crawl_result.pages.len(),
            crawl_result.request_id
        );

        let mut rag_documents = Vec::new();

        for page in &crawl_result.pages {
            match self.process_page(page, &crawl_result.provenance).await {
                Ok(doc) => {
                    self.store_in_zvec(&doc).await?;
                    rag_documents.push(doc);
                }
                Err(e) => {
                    warn!("Failed to process page {}: {}", page.url, e);
                }
            }
        }

        info!(
            "RAG pipeline completed | Documents: {} | Chunks: {}",
            rag_documents.len(),
            rag_documents.iter().map(|d| d.chunks.len()).sum::<usize>()
        );

        Ok(rag_documents)
    }

    async fn process_page(
        &self,
        page: &CrawledPage,
        provenance: &CrawlProvenance,
    ) -> Result<RagDocument, CrawlerError> {
        let markdown = page
            .markdown
            .as_ref()
            .ok_or_else(|| CrawlerError::Processing("No markdown content".to_string()))?;

        let chunks = self.chunk_text(markdown);
        debug!("Page {} chunked into {} chunks", page.url, chunks.len());

        let mut rag_chunks = Vec::new();
        for (i, chunk) in chunks.into_iter().enumerate() {
            let emb = vec![0.0; self.config.embedding_dim];
            rag_chunks.push(RagChunk {
                chunk_index: i as u32,
                text: chunk.text,
                embedding: Some(emb),
                token_count: chunk.token_count,
                overlap_chars: chunk.overlap_chars,
            });
        }

        let doc_embedding = self.compute_document_embedding(&rag_chunks);
        let fact_check = self.basic_fact_check(page).await;

        let doc = RagDocument {
            doc_id: format!(
                "rag-{}-{}",
                hex::encode(&page.content_hash[..8]),
                chrono::Utc::now().timestamp()
            ),
            source_url: page.url.clone(),
            title: page.title.clone().unwrap_or_else(|| "Untitled".to_string()),
            chunks: rag_chunks,
            embedding: Some(doc_embedding),
            metadata: RagMetadata {
                source_url: page.url.clone(),
                crawl_timestamp: page.crawled_at,
                document_type: "web_page".to_string(),
                language: page
                    .metadata
                    .language
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                word_count: markdown.split_whitespace().count() as u32,
                confidence_score: fact_check.confidence,
                fact_check_status: Some(fact_check),
            },
            provenance: provenance.clone(),
            attestation: None,
        };

        Ok(doc)
    }

    fn chunk_text(&self, text: &str) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let text_len = text.len();

        while start < text_len {
            let end = (start + self.config.chunk_size).min(text_len);
            let chunk_text = text[start..end].to_string();
            let token_count = self.estimate_tokens(&chunk_text);

            chunks.push(TextChunk {
                text: chunk_text,
                token_count,
                overlap_chars: if start > 0 {
                    self.config.chunk_overlap as u32
                } else {
                    0
                },
            });

            start += self.config.chunk_size - self.config.chunk_overlap;
        }

        chunks
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() / 4) as u32
    }

    fn compute_document_embedding(&self, chunks: &[RagChunk]) -> Vec<f32> {
        if chunks.is_empty() {
            return vec![0.0; self.config.embedding_dim];
        }

        let mut sum = vec![0.0; self.config.embedding_dim];
        let mut count = 0;

        for chunk in chunks {
            if let Some(ref emb) = chunk.embedding {
                for (i, val) in emb.iter().enumerate() {
                    sum[i] += val;
                }
                count += 1;
            }
        }

        if count > 0 {
            sum.iter_mut().for_each(|v| *v /= count as f32);
        }

        sum
    }

    async fn basic_fact_check(&self, page: &CrawledPage) -> FactCheckStatus {
        FactCheckStatus {
            checked: false,
            verified_sources: vec![page.url.clone()],
            contradictions_found: vec![],
            confidence: 0.5,
        }
    }

    async fn store_in_zvec(&self, doc: &RagDocument) -> Result<(), CrawlerError> {
        info!("Stored chunks in mock zVEC for doc {}", doc.doc_id);
        Ok(())
    }

    pub async fn retrieve_context(
        &self,
        _query: &str,
        top_k: usize,
    ) -> Result<Vec<RagChunk>, CrawlerError> {
        let mut chunks = Vec::new();
        for i in 0..top_k {
            chunks.push(RagChunk {
                chunk_index: i as u32,
                text: "Mock text chunk".to_string(),
                embedding: None,
                token_count: 10,
                overlap_chars: 0,
            });
        }
        Ok(chunks)
    }
}

#[derive(Debug, Clone)]
struct TextChunk {
    text: String,
    token_count: u32,
    overlap_chars: u32,
}
