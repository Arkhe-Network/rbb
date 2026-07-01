use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AcademicRecordType {
    Discente,
    Docente,
    Tecnico,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicRecord {
    pub institution_hash: String,
    pub person_hash: String,
    pub course_program: String,
    pub knowledge_area: String,
    pub record_type: AcademicRecordType,
    pub payload: Value,
    pub source_system: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Missing mandatory field: {0}")]
    MissingField(String),
}

#[async_trait]
pub trait AcademicAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn source_system(&self) -> &str;
    async fn translate(&self, raw_data: &Value) -> Result<AcademicRecord, AdapterError>;
    async fn validate_capes_rules(&self, record: &AcademicRecord) -> Result<bool, AdapterError>;

    fn pseudonymize(&self, data: &str) -> String {
        let hash = blake3::hash(data.as_bytes());
        hex::encode(hash.as_bytes())
    }
}
