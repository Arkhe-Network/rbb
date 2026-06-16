use async_trait::async_trait;


pub struct CompletionOptions {
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn complete(&self, prompt: &str, options: &CompletionOptions) -> Result<String, String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String>;
    fn supports_multimodal(&self) -> bool;
    fn get_name(&self) -> &str;
}

pub struct GeminiFlash;
#[async_trait]
impl LlmBackend for GeminiFlash {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String, String> {
        Ok(format!("[Gemini 3.5 Flash] Tradução multimodal completada para: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.1, 0.2, 0.3])
    }
    fn supports_multimodal(&self) -> bool { true }
    fn get_name(&self) -> &str { "Gemini 3.5 Flash" }
}

pub struct ClaudeOpus;
#[async_trait]
impl LlmBackend for ClaudeOpus {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String, String> {
        Ok(format!("[Claude Opus 4.8] Raciocínio simbólico inferido sobre: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.4, 0.5, 0.6])
    }
    fn supports_multimodal(&self) -> bool { false }
    fn get_name(&self) -> &str { "Claude Opus 4.8" }
}

pub struct GptInstant;
#[async_trait]
impl LlmBackend for GptInstant {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String, String> {
        Ok(format!("[GPT-5.5 Instant] Resposta geral/diálogo para: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.7, 0.8, 0.9])
    }
    fn supports_multimodal(&self) -> bool { false }
    fn get_name(&self) -> &str { "GPT-5.5 Instant" }
}

pub struct Grok;
#[async_trait]
impl LlmBackend for Grok {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String, String> {
        Ok(format!("[Grok 4.3] Ação executada com base em: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.2, 0.4, 0.6])
    }
    fn supports_multimodal(&self) -> bool { false }
    fn get_name(&self) -> &str { "Grok 4.3" }
}

pub struct DeepSeekV4;
#[async_trait]
impl LlmBackend for DeepSeekV4 {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String, String> {
        Ok(format!("[DeepSeek V4 Pro] Execução local soberana de: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.9, 0.8, 0.7])
    }
    fn supports_multimodal(&self) -> bool { false }
    fn get_name(&self) -> &str { "DeepSeek V4 Pro" }
}
