use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormGraphRecord {
    pub id: String,
    pub data: serde_json::Value,
    pub tags: HashMap<String, String>,
    pub timestamp: u64,
}

pub struct WormGraph {
    client: Client,
    turbo_url: String,
    arweave_gateway: String,
    wallet_key: Option<String>,
}

impl WormGraph {
    pub fn new(turbo_url: &str, arweave_gateway: &str) -> Self {
        Self {
            client: Client::new(),
            turbo_url: turbo_url.to_string(),
            arweave_gateway: arweave_gateway.to_string(),
            wallet_key: None,
        }
    }

    pub fn with_wallet(mut self, key: String) -> Self {
        self.wallet_key = Some(key);
        self
    }

    fn compute_id(data: &serde_json::Value) -> String {
        let json = data.to_string();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        BASE64.encode(result)
    }

    pub async fn store(&self, data: serde_json::Value, tags: HashMap<String, String>) -> Result<String> {
        let json_data = data.to_string();
        let mut tag_headers = Vec::new();
        for (k, v) in &tags {
            tag_headers.push(format!("{}: {}", k, v));
        }

        let url = format!("{}/v1/upload", self.turbo_url);
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_data)
            .query(&[("tags", tag_headers.join(","))])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Turbo upload failed: {}", error_text));
        }

        let json_response: serde_json::Value = response.json().await?;
        let tx_id = json_response["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' in response"))?
            .to_string();

        Ok(tx_id)
    }

    pub async fn get(&self, tx_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}", self.arweave_gateway, tx_id);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch tx {}", tx_id));
        }
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    }

    pub async fn query(&self, tags: Vec<(&str, &str)>) -> Result<Vec<String>> {
        let tag_filters = tags.iter()
            .map(|(k, v)| format!(r#"{{ name: "{}", values: ["{}"] }}"#, k, v))
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            r#"{{ transactions(tags: [{}], first: 100) {{ edges {{ node {{ id }} }} }} }}"#,
            tag_filters
        );
        let gql_url = format!("{}/graphql", self.arweave_gateway);
        let response = self.client
            .post(&gql_url)
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await?;
        let json: serde_json::Value = response.json().await?;
        let ids = json["data"]["transactions"]["edges"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|edge| edge["node"]["id"].as_str().map(|s| s.to_string()))
            .collect();
        Ok(ids)
    }

    pub async fn store_vulnerability(&self, vuln: &serde_json::Value) -> Result<String> {
        let data = serde_json::to_value(vuln)?;
        let mut tags = HashMap::new();
        tags.insert("App-Name".to_string(), "Cathedral-WormGraph".to_string());
        tags.insert("Type".to_string(), "vulnerability".to_string());
        self.store(data, tags).await
    }

    pub async fn store_proposal(&self, proposal: &serde_json::Value) -> Result<String> {
        let data = serde_json::to_value(proposal)?;
        let mut tags = HashMap::new();
        tags.insert("App-Name".to_string(), "Cathedral-WormGraph".to_string());
        tags.insert("Type".to_string(), "proposal".to_string());
        self.store(data, tags).await
    }
}
