use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoAdsRecommendation {
    pub id: String,
    pub url: String,
    pub hub: String,
}

pub struct PicoAdsClient {
    api_key: String,
    backend_url: Option<String>,
}

impl PicoAdsClient {
    pub fn new(api_key: String, backend_url: Option<String>) -> Self {
        Self {
            api_key,
            backend_url,
        }
    }

    pub async fn get_recommendations(
        &self,
        _query: &str,
        _hub: Option<&str>,
        _max_results: Option<u32>,
    ) -> Result<Vec<PicoAdsRecommendation>, String> {
        Ok(Vec::new()) // Dummy implementation
    }
}
