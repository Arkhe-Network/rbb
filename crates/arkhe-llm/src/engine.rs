#[async_trait::async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(&self, prompt: &str, temperature: f32, max_tokens: usize) -> Result<String, arkhe_core::ArkheError>;
}

pub struct RemoteEngine {
    endpoint: String,
    model: String,
}

impl RemoteEngine {
    pub fn new(endpoint: &str, model: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl InferenceEngine for RemoteEngine {
    async fn generate(&self, _prompt: &str, _temperature: f32, _max_tokens: usize) -> Result<String, arkhe_core::ArkheError> {
        Ok(String::new())
    }
}
