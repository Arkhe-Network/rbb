use super::*;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct ApexChallengeRequest {
    pub challenge_id: Option<String>,
    pub challenge_type: Option<String>,
    pub max_solutions: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApexChallenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub category: String,
    pub points: u32,
    pub time_limit_secs: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApexSolution {
    pub challenge_id: String,
    pub solution: String,
    pub proof_of_work: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApexSolutionResult {
    pub accepted: bool,
    pub score: Option<f32>,
    pub feedback: Option<String>,
    pub ranking: Option<u32>,
}

pub struct ApexClient {
    bittensor: Arc<BittensorClient>,
    subnet_id: u16,
}

impl ApexClient {
    pub fn new(bittensor: Arc<BittensorClient>) -> Self {
        Self {
            bittensor,
            subnet_id: 1,
        }
    }

    pub async fn get_challenges(
        &self,
        challenge_type: Option<&str>,
    ) -> Result<Vec<ApexChallenge>> {
        let request = ApexChallengeRequest {
            challenge_id: None,
            challenge_type: challenge_type.map(|s| s.to_string()),
            max_solutions: Some(10),
        };

        let responses = self.bittensor
            .query_subnet_with_fallback::<_, Vec<ApexChallenge>>(
                self.subnet_id,
                "challenges",
                &request,
                2,
                1,
            )
            .await?;

        let best = &responses[0];
        best.data.clone().ok_or_else(|| anyhow!("Resposta vazia da SN1"))
    }

    pub async fn submit_solution(
        &self,
        challenge_id: &str,
        solution: &str,
    ) -> Result<ApexSolutionResult> {
        let payload = ApexSolution {
            challenge_id: challenge_id.to_string(),
            solution: solution.to_string(),
            proof_of_work: None,
        };

        let responses = self.bittensor
            .query_subnet_with_fallback::<_, ApexSolutionResult>(
                self.subnet_id,
                "submit",
                &payload,
                2,
                1,
            )
            .await?;

        let best = &responses[0];
        best.data.clone().ok_or_else(|| anyhow!("Resposta vazia da SN1"))
    }

    // Agent solve is integrated at the orchestrator layer
}
