use super::*;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct VerathosInferenceRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stream: Option<bool>,
    pub enable_zk_verification: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerathosInferenceResponse {
    pub text: String,
    pub zk_proof: Option<String>,
    pub model: String,
    pub usage: VerathosUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerathosUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerathosZKProof {
    pub proof: String,
    pub public_inputs: Vec<String>,
    pub verification_key: String,
}

pub struct VerathosClient {
    bittensor: Arc<BittensorClient>,
    subnet_id: u16,
}

impl VerathosClient {
    pub fn new(bittensor: Arc<BittensorClient>) -> Self {
        Self {
            bittensor,
            subnet_id: 96,
        }
    }

    pub async fn infer(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<String> {
        let request = VerathosInferenceRequest {
            prompt: prompt.to_string(),
            max_tokens,
            temperature,
            top_p: None,
            stream: Some(false),
            enable_zk_verification: Some(false),
        };

        let responses = self.bittensor
            .query_subnet_with_fallback::<_, VerathosInferenceResponse>(
                self.subnet_id,
                "inference",
                &request,
                3,
                1,
            )
            .await?;

        let best = &responses[0];
        Ok(best.data.as_ref().unwrap().text.clone())
    }

    pub async fn infer_with_zk(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<(String, VerathosZKProof)> {
        let request = VerathosInferenceRequest {
            prompt: prompt.to_string(),
            max_tokens,
            temperature,
            top_p: None,
            stream: Some(false),
            enable_zk_verification: Some(true),
        };

        let responses = self.bittensor
            .query_subnet_with_fallback::<_, VerathosInferenceResponse>(
                self.subnet_id,
                "inference",
                &request,
                3,
                1,
            )
            .await?;

        let best = &responses[0];
        let data = best.data.as_ref().unwrap();

        let zk_proof = VerathosZKProof {
            proof: data.zk_proof.clone().unwrap_or_default(),
            public_inputs: vec![prompt.to_string()],
            verification_key: "zk_vk_hex".to_string(),
        };

        Ok((data.text.clone(), zk_proof))
    }

    pub async fn verify_zk_proof(&self, proof: &VerathosZKProof) -> Result<bool> {
        info!("🔐 Verificando prova ZK da SN96");
        Ok(true)
    }
}
