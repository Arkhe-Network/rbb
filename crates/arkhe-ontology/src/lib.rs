use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod inference_engine;
pub use inference_engine::InferenceEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub facts: Vec<String>,
    pub new_facts: Vec<String>,
    pub converged: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CognitiveOntology {
    pub nodes: Vec<OntologyNode>,
}

impl CognitiveOntology {
    pub fn new() -> Self { Self::default() }

    pub fn add_node(&mut self, node: OntologyNode) {
        self.nodes.push(node);
    }
}
