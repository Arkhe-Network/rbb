use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct BitsecAnalysisRequest {
    pub code: String,
    pub language: String,
    pub analysis_depth: Option<String>,
    pub include_fixes: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitsecVulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub location: String,
    pub cwe_id: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitsecAnalysisResponse {
    pub vulnerabilities: Vec<BitsecVulnerability>,
    pub summary: BitsecSummary,
    pub suggested_fixes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitsecSummary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

pub struct BitsecClient {
    bittensor: Arc<BittensorClient>,
    subnet_id: u16,
}

impl BitsecClient {
    pub fn new(bittensor: Arc<BittensorClient>) -> Self {
        Self {
            bittensor,
            subnet_id: 60,
        }
    }

    pub async fn analyze_code(
        &self,
        code: &str,
        language: &str,
        include_fixes: bool,
    ) -> Result<BitsecAnalysisResponse> {
        let request = BitsecAnalysisRequest {
            code: code.to_string(),
            language: language.to_string(),
            analysis_depth: Some("standard".to_string()),
            include_fixes: Some(include_fixes),
        };

        let responses = self.bittensor
            .query_subnet_with_fallback::<_, BitsecAnalysisResponse>(
                self.subnet_id,
                "analyze",
                &request,
                3,
                1,
            )
            .await?;

        let best = &responses[0];
        best.data.clone().ok_or_else(|| anyhow!("Resposta vazia da SN60"))
    }

    pub async fn analyze_file(
        &self,
        file_path: &str,
        language: &str,
    ) -> Result<BitsecAnalysisResponse> {
        let code = std::fs::read_to_string(file_path)?;
        self.analyze_code(&code, language, true).await
    }

    pub async fn get_critical_vulnerabilities(
        &self,
        code: &str,
        language: &str,
    ) -> Result<Vec<BitsecVulnerability>> {
        let response = self.analyze_code(code, language, false).await?;
        let critical: Vec<BitsecVulnerability> = response.vulnerabilities
            .into_iter()
            .filter(|v| v.severity == "critical")
            .collect();
        Ok(critical)
    }
}
