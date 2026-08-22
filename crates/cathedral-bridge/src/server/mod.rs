use cathedral_nostr::NostrReplicator;
use cathedral_wormgraph::Wormgraph;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct VerificationKey {
    pub hash: Vec<u8>,
    pub elf: Vec<u8>,
}

pub struct BridgeState {
    pub verification_keys: RwLock<HashMap<String, VerificationKey>>,
    pub wormgraph: Wormgraph,
    pub nostr_replicator: Option<NostrReplicator>,
}
