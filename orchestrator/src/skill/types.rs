use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillType {
    /// Invocada pelo usuário (ex: /grill-me, /to-prd)
    UserInvoked,
    /// Invocada pelo modelo automaticamente (ex: /tdd, /diagnose)
    ModelInvoked,
    /// Executada em background (ex: /improve-codebase-architecture)
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub skill_type: SkillType,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub triggers: Vec<String>,
    pub instructions: String,
    pub steps: Vec<SkillStep>,
    pub examples: Vec<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub okf_bundle_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub order: usize,
    pub description: String,
    pub expected_output: String,
    pub validation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecution {
    pub skill_name: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub status: ExecutionStatus,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            description: String::new(),
            skill_type: SkillType::UserInvoked,
            version: "1.0.0".to_string(),
            author: None,
            tags: vec![],
            triggers: vec![],
            instructions: String::new(),
            steps: vec![],
            examples: vec![],
            dependencies: vec![],
            metadata: HashMap::new(),
            okf_bundle_id: None,
        }
    }
}
