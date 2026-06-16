use crate::llm_api::{CompletionOptions, LlmBackend};
use crate::task::CognitiveTask;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NeuroSymbolicOrchestrator {
    backends: HashMap<String, Arc<dyn LlmBackend>>,
    prefer_local_llm: bool,
}

impl NeuroSymbolicOrchestrator {
    pub fn new(prefer_local_llm: bool) -> Self {
        let mut backends: HashMap<String, Arc<dyn LlmBackend>> = HashMap::new();
        backends.insert("gemini".to_string(), Arc::new(crate::llm_api::GeminiFlash));
        backends.insert("gpt".to_string(), Arc::new(crate::llm_api::GptInstant));
        backends.insert("claude".to_string(), Arc::new(crate::llm_api::ClaudeOpus));
        backends.insert("grok".to_string(), Arc::new(crate::llm_api::Grok));
        backends.insert("deepseek".to_string(), Arc::new(crate::llm_api::DeepSeekV4));
        Self { backends, prefer_local_llm }
    }

    pub async fn process_task(&self, task: CognitiveTask) -> Result<String, String> {
        let options = CompletionOptions { max_tokens: Some(512), temperature: Some(0.7) };
        match task {
            CognitiveTask::TranslateMultimodal(prompt) => {
                let backend = self.backends.get("gemini").unwrap();
                println!("Route: {:?}", backend.get_name());
                backend.complete(&prompt, &options).await
            }
            CognitiveTask::HumanDialog(prompt) => {
                let backend = self.backends.get("gpt").unwrap();
                println!("Route: {:?}", backend.get_name());
                backend.complete(&prompt, &options).await
            }
            CognitiveTask::SymbolicReasoning(prompt) => {
                let backend_key = if self.prefer_local_llm { "deepseek" } else { "claude" };
                let backend = self.backends.get(backend_key).unwrap();
                println!("Route: {:?}", backend.get_name());
                backend.complete(&prompt, &options).await
            }
            CognitiveTask::ExecuteAction(prompt) => {
                let backend = self.backends.get("grok").unwrap();
                println!("Route: {:?}", backend.get_name());
                backend.complete(&prompt, &options).await
            }
        }
    }
}
