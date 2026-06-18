// src/evolution/desci_node_resource.rs
//! DeSciNodeResource — Research Objects versionáveis integrados ao HashTree + Open State Repository

use crate::evolution::resource::{Resource, ResourceMetadata, ResourceInterface, ResourceState, ProvenanceEntry};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResearchComponentType {
    Manuscript,
    Dataset,
    Code,
    Model,
    Pipeline,
    Supplementary,
    Custom(String),
}

impl std::fmt::Display for ResearchComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manuscript => write!(f, "manuscript"),
            Self::Dataset => write!(f, "dataset"),
            Self::Code => write!(f, "code"),
            Self::Model => write!(f, "model"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Supplementary => write!(f, "supplementary"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchComponent {
    pub component_type: ResearchComponentType,
    pub name: String,
    pub hash: String,                    // Hash no HashTree
    pub cid: Option<String>,             // IPFS CID (interoperabilidade)
    pub size_bytes: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorCredit {
    pub npub: String,
    pub orcid: Option<String>,
    pub role: String,                    // "author", "data-curator", "reviewer", "maintainer"
    pub contribution_score: f64,         // 0.0 - 1.0
    pub contribution_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVersion {
    pub version: String,
    pub hash: String,
    pub created_at: u64,
    pub created_by: String,
    pub changelog: String,
    pub components: Vec<ResearchComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Draft,
    Submitted,
    UnderReview,
    Published,
    Revised,
    Retracted,
    Archived,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Submitted => write!(f, "submitted"),
            Self::UnderReview => write!(f, "under_review"),
            Self::Published => write!(f, "published"),
            Self::Revised => write!(f, "revised"),
            Self::Retracted => write!(f, "retracted"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReviewRecord {
    pub reviewer_npub: String,
    pub reviewer_orcid: Option<String>,
    pub score: u8,                         // 0-10
    pub comments: String,
    pub is_public: bool,
    pub reviewed_at: u64,
    pub version: String,                   // versão revisada
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeSciNodeResource {
    pub metadata: ResourceMetadata,
    pub node_id: String,                 // ID interno do DeSci
    pub dpid: String,                    // Decentralized Persistent Identifier
    pub title: String,
    pub abstract_text: Option<String>,
    pub components: Vec<ResearchComponent>,
    pub contributors: Vec<ContributorCredit>,
    pub orcid_links: Vec<String>,
    pub versions: Vec<NodeVersion>,
    pub current_version: String,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub status: NodeStatus,
    pub peer_reviews: Vec<PeerReviewRecord>,
    pub external_refs: Vec<String>,
}

impl DeSciNodeResource {
    pub fn new(
        title: &str,
        dpid: &str,
        author_npub: &str,
        author_orcid: Option<&str>,
    ) -> Self {
        let now = Utc::now().timestamp() as u64;
        let node_id = format!("desci:{}", uuid::Uuid::new_v4());

        let mut contributors = vec![ContributorCredit {
            npub: author_npub.to_string(),
            orcid: author_orcid.map(|s| s.to_string()),
            role: "author".to_string(),
            contribution_score: 1.0,
            contribution_description: Some("Initial creation".to_string()),
        }];

        if let Some(orcid) = author_orcid {
            // Registra automaticamente no ORCID via integração futura
        }

        Self {
            metadata: ResourceMetadata {
                id: node_id.clone(),
                version: "1.0.0".to_string(),
                state: ResourceState::Active,
                interface: ResourceInterface {
                    input_schema: serde_json::json!({}),
                    output_schema: serde_json::json!({}),
                    side_effects: vec!["publishes_research".to_string()],
                    dependencies: vec!["hash_tree".to_string()],
                },
                created_at: now,
                updated_at: now,
                author: author_npub.to_string(),
                provenance: Vec::new(),
                tags: vec!["desci".to_string(), "research".to_string()],
                metadata: HashMap::new(),
            },
            node_id,
            dpid: dpid.to_string(),
            title: title.to_string(),
            abstract_text: None,
            components: Vec::new(),
            contributors,
            orcid_links: author_orcid.map(|o| vec![o.to_string()]).unwrap_or_default(),
            versions: vec![NodeVersion {
                version: "v1".to_string(),
                hash: "".to_string(),
                created_at: now,
                created_by: author_npub.to_string(),
                changelog: "Initial version".to_string(),
                components: vec![],
            }],
            current_version: "v1".to_string(),
            license: Some("CC-BY-4.0".to_string()),
            keywords: vec![],
            status: NodeStatus::Draft,
            peer_reviews: Vec::new(),
            external_refs: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: ResearchComponent) {
        self.components.push(component);
        self.metadata.updated_at = Utc::now().timestamp() as u64;
    }

    pub fn add_contributor(&mut self, contributor: ContributorCredit) {
        self.contributors.push(contributor);
        self.metadata.updated_at = Utc::now().timestamp() as u64;
    }

    pub fn create_new_version(&mut self, changelog: &str, author: &str) -> String {
        let new_version = format!("v{}", self.versions.len() + 1);
        let now = Utc::now().timestamp() as u64;

        self.versions.push(NodeVersion {
            version: new_version.clone(),
            hash: "".to_string(), // será preenchido após commit no HashTree
            created_at: now,
            created_by: author.to_string(),
            changelog: changelog.to_string(),
            components: self.components.clone(),
        });

        self.current_version = new_version.clone();
        self.metadata.updated_at = now;
        new_version
    }

    pub fn get_reputation_score(&self) -> f32 {
        let mut score = 0.0;
        if self.status == NodeStatus::Published { score += 20.0; }
        let review_score = (self.peer_reviews.len() as f32).min(10.0) * 3.0;
        score += review_score;
        let contributor_score = (self.contributors.len() as f32).min(5.0) * 2.0;
        score += contributor_score;
        let version_score = (self.versions.len() as f32).min(5.0) * 2.0;
        score += version_score;
        if !self.external_refs.is_empty() { score += 5.0; }
        if !self.dpid.is_empty() { score += 5.0; }
        score.min(100.0)
    }

    pub fn is_fair_compliant(&self) -> bool {
        let has_metadata = self.title.len() > 3 && self.abstract_text.is_some() && !self.keywords.is_empty();
        let has_components = !self.versions.is_empty() && self.versions.iter().any(|v| !v.components.is_empty());
        let has_pid = !self.dpid.is_empty();
        let has_license = self.license.is_some();
        let has_provenance = !self.metadata.provenance.is_empty();
        has_metadata && has_components && has_pid && has_license && has_provenance
    }
}

impl Resource for DeSciNodeResource {
    fn metadata(&self) -> &ResourceMetadata { &self.metadata }
    fn metadata_mut(&mut self) -> &mut ResourceMetadata { &mut self.metadata }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Erro ao serializar DeSciNodeResource: {}", e))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Erro ao deserializar DeSciNodeResource: {}", e))
    }
}
