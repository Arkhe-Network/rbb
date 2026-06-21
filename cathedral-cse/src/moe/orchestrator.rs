use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};

use crate::agent::AgentMessage;
use crate::thinking::Thought;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsciousnessState {
    Dormant,
    Aware,
    Reflective,
    MetaCognitiva,
    Autopoiética,
}

impl ConsciousnessState {
    pub fn top_k(&self) -> usize {
        match self {
            ConsciousnessState::Dormant => 1,
            ConsciousnessState::Aware => 2,
            ConsciousnessState::Reflective => 3,
            ConsciousnessState::MetaCognitiva => 4,
            ConsciousnessState::Autopoiética => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CognitiveCapability {
    Reactive,
    Symbolic,
    Planning,
    Episodic,
    Causal,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct CognitiveContext {
    pub prompt: String,
    pub consciousness: ConsciousnessState,
    pub eac_metrics: [f64; 5],
    pub history: Vec<AgentMessage>,
    pub available_tools: Vec<String>,
    pub constraints: Vec<String>,
    pub thinking_trace: Option<Vec<Thought>>,
}

impl CognitiveContext {
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            consciousness: ConsciousnessState::Aware,
            eac_metrics: [0.5, 0.5, 0.5, 0.5, 0.5],
            history: Vec::new(),
            available_tools: Vec::new(),
            constraints: Vec::new(),
            thinking_trace: None,
        }
    }

    pub fn with_consciousness(mut self, state: ConsciousnessState) -> Self {
        self.consciousness = state;
        self
    }

    pub fn with_eac(mut self, metrics: [f64; 5]) -> Self {
        self.eac_metrics = metrics;
        self
    }

    pub fn with_thinking_trace(mut self, trace: Vec<Thought>) -> Self {
        self.thinking_trace = Some(trace);
        self
    }
}

#[derive(Debug, Clone)]
pub struct CognitiveOutput {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub confidence: f64,
    pub thinking_trace: Option<String>,
    pub source_expert: String,
}

#[async_trait]
pub trait CognitiveExpert: Send + Sync {
    fn id(&self) -> String;
    fn capability(&self) -> CognitiveCapability;
    fn activation_score(&self, ctx: &CognitiveContext) -> f64;
    async fn process(&self, ctx: &CognitiveContext) -> Result<CognitiveOutput, String>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouterStrategy {
    MaxConfidence,
    WeightedVoting,
    Concatenate,
    Adaptive,
}

impl Default for RouterStrategy {
    fn default() -> Self {
        RouterStrategy::Adaptive
    }
}

pub struct CognitiveRouter {
    pub strategy: RouterStrategy,
}

impl CognitiveRouter {
    pub fn new() -> Self {
        Self { strategy: RouterStrategy::default() }
    }

    pub async fn combine(
        &self,
        outputs: Vec<(String, CognitiveOutput)>,
        ctx: &CognitiveContext,
    ) -> Result<Vec<CognitiveOutput>, String> {
        if outputs.is_empty() {
            return Err("Nenhum output para combinar".to_string());
        }

        let strategy = match self.strategy {
            RouterStrategy::Adaptive => match ctx.consciousness {
                ConsciousnessState::Dormant | ConsciousnessState::Aware => RouterStrategy::MaxConfidence,
                ConsciousnessState::Reflective => RouterStrategy::WeightedVoting,
                ConsciousnessState::MetaCognitiva | ConsciousnessState::Autopoiética => RouterStrategy::Concatenate,
            },
            _ => self.strategy,
        };

        match strategy {
            RouterStrategy::MaxConfidence => {
                let best = outputs
                    .into_iter()
                    .max_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap())
                    .map(|(_, out)| out)
                    .ok_or("Não foi possível seleccionar o melhor output")?;
                Ok(vec![best])
            }
            RouterStrategy::WeightedVoting => {
                let mut combined_content = String::new();
                let mut combined_tools = Vec::new();
                let mut total_confidence = 0.0;

                for (expert_id, output) in &outputs {
                    total_confidence += output.confidence;
                    if !output.content.is_empty() {
                        combined_content.push_str(&format!("[{}] {}\n", expert_id, output.content));
                    }
                    combined_tools.extend(output.tool_calls.clone());
                }

                Ok(vec![CognitiveOutput {
                    content: combined_content,
                    tool_calls: combined_tools,
                    confidence: total_confidence / outputs.len() as f64,
                    thinking_trace: None,
                    source_expert: "combined".to_string(),
                }])
            }
            RouterStrategy::Concatenate => {
                let mut combined = CognitiveOutput {
                    content: String::new(),
                    tool_calls: Vec::new(),
                    confidence: 0.0,
                    thinking_trace: None,
                    source_expert: "combined".to_string(),
                };

                let mut total_conf = 0.0;
                for (expert_id, output) in &outputs {
                    combined.content.push_str(&format!("[{}]\n{}\n\n", expert_id, output.content));
                    combined.tool_calls.extend(output.tool_calls.clone());
                    total_conf += output.confidence;
                }
                combined.confidence = total_conf / outputs.len() as f64;

                let trace_str = outputs
                    .iter()
                    .filter_map(|(_, o)| o.thinking_trace.clone())
                    .collect::<Vec<_>>()
                    .join("\n---\n");
                if !trace_str.is_empty() {
                    combined.thinking_trace = Some(trace_str);
                }

                Ok(vec![combined])
            }
        }
    }
}

pub struct MoeCognitiveOrchestrator {
    experts: Vec<Arc<dyn CognitiveExpert>>,
    router: CognitiveRouter,
    expert_capacity: HashMap<String, u32>,
}

impl MoeCognitiveOrchestrator {
    pub fn new() -> Self {
        Self {
            experts: Vec::new(),
            router: CognitiveRouter::new(),
            expert_capacity: HashMap::new(),
        }
    }

    pub fn register_expert(&mut self, expert: Arc<dyn CognitiveExpert>, capacity: u32) {
        let id = expert.id();
        self.experts.push(expert);
        self.expert_capacity.insert(id, capacity);
    }

    pub async fn route_and_process(&self, ctx: &CognitiveContext) -> Result<Vec<CognitiveOutput>, String> {
        let mut scores: Vec<_> = self.experts
            .iter()
            .map(|e| (e.id(), e.activation_score(ctx)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let k = ctx.consciousness.top_k();
        let selected: Vec<_> = scores.into_iter().take(k).collect();

        let mut handles = tokio::task::JoinSet::new();
        for (expert_id, _) in selected {
            if let Some(expert) = self.experts.iter().find(|e| e.id() == expert_id) {
                let expert = Arc::clone(expert);
                let ctx_clone = ctx.clone();
                handles.spawn(async move {
                    (expert_id.clone(), expert.process(&ctx_clone).await)
                });
            }
        }

        let mut outputs = Vec::new();
        while let Some(result) = handles.join_next().await {
            if let Ok((expert_id, Ok(output))) = result {
                outputs.push((expert_id, output));
            }
        }

        if outputs.is_empty() {
            return Err("Nenhum expert conseguiu processar a requisição".to_string());
        }

        self.router.combine(outputs, ctx).await
    }
}
