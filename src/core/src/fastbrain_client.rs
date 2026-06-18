use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

pub struct FastBrainClient {
    client: Client,
    base_url: String,
}

impl FastBrainClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn generate_fix(&self, finding: &Value, code: &str, language: &str) -> Result<String> {
        let response = self.client
            .post(&format!("{}/fix", self.base_url))
            .json(&json!({
                "finding": finding,
                "code": code,
                "language": language,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            let fixed_code = result["fixed_code"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing fixed_code in response"))?;
            Ok(fixed_code.to_string())
        } else {
            let error = response.text().await?;
            Err(anyhow::anyhow!("FastBrain error: {}", error))
        }
    }
}
