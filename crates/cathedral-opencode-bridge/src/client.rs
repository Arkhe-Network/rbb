use crate::protocol::{OpenCodeRequest, OpenCodeResponse};
use reqwest::Client;

pub struct OpenCodeClient {
    http: Client,
    base_url: String,
}

impl OpenCodeClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: Client::new(),
            base_url,
        }
    }

    pub async fn run_agent(&self, req: OpenCodeRequest) -> Result<OpenCodeResponse, String> {
        let url = format!("{}/v1/agent/run", self.base_url);
        let resp = self.http.post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<OpenCodeResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp)
    }
}
