//! Publicação descentralizada de datasets científicos

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{DesciError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub name: String,
    pub description: String,
    pub format: String,
    pub version: String,
    pub author_did: String,
    pub license: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub checksum_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsPublishResult {
    pub cid: String,
    pub gateway_url: String,
    pub size_bytes: u64,
}

pub struct IpfsClient {
    api_url: String,
    gateway_url: String,
    #[cfg(feature = "ipfs")]
    http_client: reqwest::Client,
}

impl IpfsClient {
    pub fn local() -> Self {
        Self {
            api_url: "http://127.0.0.1:5001/api/v0".to_string(),
            gateway_url: "http://127.0.0.1:8080/ipfs".to_string(),
            #[cfg(feature = "ipfs")]
            http_client: reqwest::Client::new(),
        }
    }

    pub fn new(api_url: &str, gateway_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            gateway_url: gateway_url.to_string(),
            #[cfg(feature = "ipfs")]
            http_client: reqwest::Client::new(),
        }
    }

    #[cfg(feature = "ipfs")]
    pub async fn add_file(&self, path: &str) -> Result<IpfsPublishResult> {
        let file_bytes = tokio::fs::read(path).await.map_err(|e| DesciError::Io(e))?;

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(file_bytes.clone()).file_name(
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("data")
                    .to_string(),
            ),
        );

        let response = self
            .http_client
            .post(format!("{}/add", self.api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| DesciError::IpfsError(format!("Request failed: {}", e)))?
            .error_for_status()
            .map_err(|e| DesciError::IpfsError(format!("API error: {}", e)))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| DesciError::IpfsError(format!("JSON parse error: {}", e)))?;

        let cid = response["Hash"]
            .as_str()
            .ok_or_else(|| DesciError::IpfsError("No CID in response".to_string()))?
            .to_string();

        let size = response["Size"].as_u64().unwrap_or(file_bytes.len() as u64);

        info!(cid = %cid, size = size, "File added to IPFS");

        Ok(IpfsPublishResult {
            cid: cid.clone(),
            gateway_url: format!("{}/{}", self.gateway_url, cid),
            size_bytes: size,
        })
    }

    #[cfg(feature = "ipfs")]
    pub async fn add_bytes(&self, data: &[u8], filename: &str) -> Result<IpfsPublishResult> {
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(data.to_vec()).file_name(filename.to_string()),
        );

        let response = self
            .http_client
            .post(format!("{}/add", self.api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| DesciError::IpfsError(format!("Request failed: {}", e)))?
            .error_for_status()
            .map_err(|e| DesciError::IpfsError(format!("API error: {}", e)))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| DesciError::IpfsError(format!("JSON parse error: {}", e)))?;

        let cid = response["Hash"]
            .as_str()
            .ok_or_else(|| DesciError::IpfsError("No CID in response".to_string()))?
            .to_string();

        let size = response["Size"].as_u64().unwrap_or(data.len() as u64);

        Ok(IpfsPublishResult {
            cid: cid.clone(),
            gateway_url: format!("{}/{}", self.gateway_url, cid),
            size_bytes: size,
        })
    }

    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    pub fn gateway_url(&self) -> &str {
        &self.gateway_url
    }
}

pub struct WormGraphNotifier {
    endpoint: String,
}

impl WormGraphNotifier {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn notify_publication(
        &self,
        cid: &str,
        metadata: &DatasetMetadata,
    ) -> Result<String> {
        let notification_id = blake3::hash(
            format!(
                "{}:{}:{}",
                cid,
                metadata.name,
                chrono::Utc::now().timestamp_millis()
            )
            .as_bytes(),
        )
        .to_string();

        info!(
            notification_id = %notification_id,
            cid = %cid,
            dataset = %metadata.name,
            endpoint = %self.endpoint,
            "WormGraph notification sent (stub)"
        );

        Ok(notification_id)
    }
}

pub struct DeSciPublisher {
    ipfs_client: IpfsClient,
    wormgraph: WormGraphNotifier,
}

impl DeSciPublisher {
    pub fn local() -> Self {
        Self {
            ipfs_client: IpfsClient::local(),
            wormgraph: WormGraphNotifier::new("http://localhost:50051"),
        }
    }

    pub fn new(ipfs_api: &str, ipfs_gateway: &str, wormgraph_endpoint: &str) -> Self {
        Self {
            ipfs_client: IpfsClient::new(ipfs_api, ipfs_gateway),
            wormgraph: WormGraphNotifier::new(wormgraph_endpoint),
        }
    }

    #[cfg(feature = "ipfs")]
    pub async fn publish(
        &self,
        file_path: &str,
        metadata: DatasetMetadata,
    ) -> Result<PublishResult> {
        let ipfs_result = self.ipfs_client.add_file(file_path).await?;

        let notification_id = self
            .wormgraph
            .notify_publication(&ipfs_result.cid, &metadata)
            .await?;

        info!(
            cid = %ipfs_result.cid,
            notification = %notification_id,
            "Dataset published successfully"
        );

        Ok(PublishResult {
            cid: ipfs_result.cid,
            gateway_url: ipfs_result.gateway_url,
            size_bytes: ipfs_result.size_bytes,
            notification_id,
            metadata,
        })
    }

    #[cfg(feature = "ipfs")]
    pub async fn publish_bytes(
        &self,
        data: &[u8],
        filename: &str,
        metadata: DatasetMetadata,
    ) -> Result<PublishResult> {
        let ipfs_result = self.ipfs_client.add_bytes(data, filename).await?;

        let notification_id = self
            .wormgraph
            .notify_publication(&ipfs_result.cid, &metadata)
            .await?;

        Ok(PublishResult {
            cid: ipfs_result.cid,
            gateway_url: ipfs_result.gateway_url,
            size_bytes: ipfs_result.size_bytes,
            notification_id,
            metadata,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub cid: String,
    pub gateway_url: String,
    pub size_bytes: u64,
    pub notification_id: String,
    pub metadata: DatasetMetadata,
}

#[allow(dead_code)]
pub struct CcipClient {
    _router_address: String,
    _chain_id: u64,
}

#[allow(dead_code)]
impl CcipClient {
    pub fn new(router_address: &str, chain_id: u64) -> Self {
        Self {
            _router_address: router_address.to_string(),
            _chain_id: chain_id,
        }
    }

    pub async fn send_message(&self, _payload: &[u8]) -> Result<String> {
        Err(DesciError::NotImplemented(
            "CCIP integration requires ethers-rs/alloy + smart contract deployment.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata() -> DatasetMetadata {
        DatasetMetadata {
            name: "BRCA1 Variant Dataset".to_string(),
            description: "Curated variants of BRCA1 gene".to_string(),
            format: "vcf".to_string(),
            version: "1.0.0".to_string(),
            author_did: "did:arkhe:researcher-001".to_string(),
            license: "CC-BY-4.0".to_string(),
            tags: vec!["genomics".into(), "brca1".into(), "cancer".into()],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            checksum_sha256: "abc123".to_string(),
        }
    }

    #[test]
    fn test_dataset_metadata_serialization() {
        let meta = sample_metadata();
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: DatasetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, meta.name);
        assert_eq!(deserialized.author_did, meta.author_did);
    }

    #[test]
    fn test_ipfs_client_local_urls() {
        let client = IpfsClient::local();
        assert_eq!(client.api_url(), "http://127.0.0.1:5001/api/v0");
        assert!(client.gateway_url().contains("8080"));
    }

    #[test]
    fn test_ipfs_client_custom_urls() {
        let client = IpfsClient::new(
            "http://10.0.0.1:5001/api/v0",
            "https://gateway.example.com/ipfs",
        );
        assert_eq!(client.api_url(), "http://10.0.0.1:5001/api/v0");
        assert_eq!(client.gateway_url(), "https://gateway.example.com/ipfs");
    }

    #[test]
    fn test_ccip_stub_returns_not_implemented() {
        let client = CcipClient::new("0x...", 1);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(client.send_message(&[]));
        assert!(result.is_err());
        match result.unwrap_err() {
            DesciError::NotImplemented(msg) => {
                assert!(msg.contains("ethers-rs"));
            }
            other => panic!("Expected NotImplemented, got: {}", other),
        }
    }

    #[test]
    fn test_publish_result_serialization() {
        let result = PublishResult {
            cid: "QmTest".to_string(),
            gateway_url: "http://localhost:8080/ipfs/QmTest".to_string(),
            size_bytes: 1024,
            notification_id: "notif-123".to_string(),
            metadata: sample_metadata(),
        };
        let json = serde_json::to_string_pretty(&result).unwrap();
        assert!(json.contains("QmTest"));
        assert!(json.contains("BRCA1"));
    }
}
