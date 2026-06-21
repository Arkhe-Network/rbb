use async_trait::async_trait;
use std::sync::Arc;
use crate::agent::AgentMessage;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat_completion(&self, messages: &[AgentMessage], tools: Option<serde_json::Value>) -> Result<String, String>;
    fn clone_arc(&self) -> Arc<dyn LlmClient + Send + Sync>;
}
