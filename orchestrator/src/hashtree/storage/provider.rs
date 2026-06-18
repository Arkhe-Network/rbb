use async_trait::async_trait;
use crate::hashtree::types::{StoreRequest, StoreResponse, RetrieveRequest, RetrieveResponse, ContentHash};
use crate::hashtree::nostr::resolver::NostrReference;
use crate::hashtree::nostr::events::NostrEvent;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn store(&self, req: StoreRequest) -> Result<StoreResponse, String>;
    async fn retrieve(&self, req: RetrieveRequest) -> Result<RetrieveResponse, String>;
    async fn publish_nostr_ref(&self, npub: &str, path: &str, hash: &ContentHash) -> Result<NostrEvent, String>;
    async fn resolve_nostr_ref(&self, npub: &str, path: &str) -> Result<NostrReference, String>;
}

pub struct HashTreeConfig;

impl Default for HashTreeConfig {
    fn default() -> Self {
        Self
    }
}

pub struct HashTreeStorageProvider;

impl HashTreeStorageProvider {
    pub fn new(_config: HashTreeConfig) -> Self {
        Self
    }
}

#[async_trait]
impl StorageProvider for HashTreeStorageProvider {
    async fn store(&self, _req: StoreRequest) -> Result<StoreResponse, String> {
        Ok(StoreResponse { content_hash: ContentHash { hash: [0; 32], hash_type: crate::hashtree::types::HashType::Sha256 } })
    }
    async fn retrieve(&self, _req: RetrieveRequest) -> Result<RetrieveResponse, String> {
        Ok(RetrieveResponse { data: vec![] })
    }
    async fn publish_nostr_ref(&self, _npub: &str, _path: &str, _hash: &ContentHash) -> Result<NostrEvent, String> {
        Ok(NostrEvent { id: "mock".to_string() })
    }
    async fn resolve_nostr_ref(&self, _npub: &str, _path: &str) -> Result<NostrReference, String> {
        Ok(NostrReference { current_root: ContentHash { hash: [0; 32], hash_type: crate::hashtree::types::HashType::Sha256 } })
    }
}
