use cathedral_identity::Did;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AssetRef {
    AssetId(String),
    GroupKey(String),
}

impl AssetRef {
    pub fn from_string(s: &str) -> Self {
        if s.starts_with("group_key_") {
            AssetRef::GroupKey(s.trim_start_matches("group_key_").to_string())
        } else {
            AssetRef::AssetId(s.to_string())
        }
    }

    pub fn to_did(&self) -> Did {
        match self {
            AssetRef::AssetId(id) => Did { id: format!("did:cathedral:asset:{}", id) },
            AssetRef::GroupKey(key) => Did { id: format!("did:cathedral:group:{}", key) },
        }
    }

    pub fn from_did(did: &Did) -> Option<Self> {
        let s = did.id.as_str();
        if let Some(id) = s.strip_prefix("did:cathedral:asset:") {
            Some(AssetRef::AssetId(id.to_string()))
        } else if let Some(key) = s.strip_prefix("did:cathedral:group:") {
            Some(AssetRef::GroupKey(key.to_string()))
        } else {
            None
        }
    }
}
