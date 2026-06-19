use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ChutesJobRequest {
    pub image: String,
    pub command: Vec<String>,
    pub gpu_memory: Option<u32>,
    pub cpu_cores: Option<u32>,
    pub ram_mb: Option<u32>,
    pub env_vars: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChutesJobResponse {
    pub job_id: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub metrics: ChutesMetrics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChutesMetrics {
    pub gpu_utilization: f32,
    pub memory_usage_mb: u32,
    pub execution_time_ms: u64,
}

pub struct ChutesClient {
    bittensor: Arc<BittensorClient>,
    subnet_id: u16,
}

impl ChutesClient {
    pub fn new(bittensor: Arc<BittensorClient>) -> Self {
        Self {
            bittensor,
            subnet_id: 64,
        }
    }

    pub async fn execute_job(
        &self,
        request: &ChutesJobRequest,
    ) -> Result<ChutesJobResponse> {
        let responses = self.bittensor
            .query_subnet_with_fallback::<_, ChutesJobResponse>(
                self.subnet_id,
                "execute",
                request,
                2,
                1,
            )
            .await?;

        let best = &responses[0];
        best.data.clone().ok_or_else(|| anyhow!("Resposta vazia da SN64"))
    }

    pub async fn train_model(
        &self,
        model_code: &str,
        dataset_path: &str,
        epochs: u32,
    ) -> Result<String> {
        let request = ChutesJobRequest {
            image: "cathedral/trainer:latest".to_string(),
            command: vec![
                "python".to_string(),
                "train.py".to_string(),
                "--dataset".to_string(),
                dataset_path.to_string(),
                "--epochs".to_string(),
                epochs.to_string(),
            ],
            gpu_memory: Some(8192),
            cpu_cores: Some(4),
            ram_mb: Some(16384),
            env_vars: Some(vec![
                ("MODEL_CODE".to_string(), model_code.to_string()),
            ]),
        };

        let response = self.execute_job(&request).await?;
        Ok(response.output.unwrap_or_default())
    }

    pub async fn batch_infer(
        &self,
        prompts: Vec<String>,
        model_path: &str,
    ) -> Result<Vec<String>> {
        let request = ChutesJobRequest {
            image: "cathedral/inference:latest".to_string(),
            command: vec![
                "python".to_string(),
                "infer.py".to_string(),
                "--model".to_string(),
                model_path.to_string(),
                "--batch".to_string(),
                prompts.len().to_string(),
            ],
            gpu_memory: Some(4096),
            cpu_cores: Some(2),
            ram_mb: Some(8192),
            env_vars: Some(vec![
                ("PROMPTS".to_string(), prompts.join("\n")),
            ]),
        };

        let response = self.execute_job(&request).await?;
        let lines: Vec<String> = response.output
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(lines)
    }
}
