use super::{CognitiveOntology, InferenceResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngine;

impl InferenceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn infer(&self, ontology: &CognitiveOntology, query: &str) -> Result<InferenceResult, String> {
        let mut relevant_nodes = Vec::new();
        for node in &ontology.nodes {
            if node.label.to_lowercase().contains(&query.to_lowercase()) {
                relevant_nodes.push(node.clone());
            }
        }

        let new_facts: Vec<String> = relevant_nodes.iter()
            .map(|n| format!("{}: {}", n.label, n.properties.get("definition").unwrap_or(&"".to_string())))
            .collect();

        Ok(InferenceResult {
            facts: new_facts.clone(),
            new_facts,
            converged: false,
        })
    }

    pub fn verify_consistency(&self, _ontology: &CognitiveOntology) -> Result<bool, String> {
        Ok(true)
    }
}
