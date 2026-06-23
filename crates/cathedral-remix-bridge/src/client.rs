use crate::protocol::*;
use reqwest::Client;
use std::time::Duration;

pub struct RemixClient {
    http: Client,
    base_url: String,
}

impl RemixClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
            base_url,
        }
    }

    pub async fn compile(&self, req: CompileRequest) -> Result<CompileResponse, String> {
        let url = format!("{}/api/compile", self.base_url);
        let resp = self.http
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<CompileResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp)
    }

    pub async fn deploy(&self, req: DeployRequest) -> Result<DeployResponse, String> {
        let url = format!("{}/api/deploy", self.base_url);
        let resp = self.http
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<DeployResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp)
    }
}
