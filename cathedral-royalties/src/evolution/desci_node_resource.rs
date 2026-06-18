use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltySplit {
    pub npub: String,
    pub share: f32,
    pub orcid: Option<String>,
    pub eth_address: Option<String>,
    pub pix_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeTier {
    pub max_free_accesses: u32,
    pub reset_interval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltyConfig {
    pub enabled: bool,
    pub price_per_access: String,
    pub currency: String,
    pub chain: String,
    pub royalty_split: Vec<RoyaltySplit>,
    pub free_tier: Option<FreeTier>,
    pub picnic_basket: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
