use serde::{Deserialize, Serialize};
use cathedral_identity::Did;
use cathedral_permissions::PermissionEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_did: Did,
    pub signature: Vec<u8>,
    pub metadata: SkillMetadata,
    pub implementation: SkillImplementation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub version: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<PermissionEntry>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillImplementation {
    Rust { code: String, entrypoint: String },
    Python { code: String, entrypoint: String },
    Shell { script: String },
    Wasm { module: Vec<u8> },
}

impl Skill {
    pub fn verify(&self) -> Result<bool, String> {
        cathedral_identity::verify_signature(
            &self.author_did,
            &self.signature,
            &serde_json::to_vec(self).map_err(|e| e.to_string())?,
        ).map_err(|_| "Signature verification failed".to_string())
    }
}
