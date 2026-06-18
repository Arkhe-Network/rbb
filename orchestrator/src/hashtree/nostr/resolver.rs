use crate::hashtree::types::ContentHash;

pub struct NostrReference {
    pub current_root: ContentHash,
}

pub struct NostrReferenceResolver {
    _relays: Vec<String>,
}

impl NostrReferenceResolver {
    pub fn new(relays: Vec<String>) -> Self {
        Self { _relays: relays }
    }

    pub async fn resolve(&self, _npub: &str, _path: &str) -> Result<NostrReference, String> {
        Err("Mock resolver".to_string())
    }
}
