use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SusApiUrls {
    pub esus_ab: String,
    pub sisreg: String,
    pub cnes: String,
    pub snt: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Não encontrado")]
    NotFound,
    #[error("Erro de autenticação: {0}")]
    Auth(String),
    #[error("Erro do servidor: {0}")]
    ServerError(String),
    #[error("Erro de rede: {0}")]
    Network(String),
    #[error("Desconhecido: {0}")]
    Unknown(String),
}

pub struct SusApiGateway {
    urls: SusApiUrls,
    client: Client,
}

impl SusApiGateway {
    pub fn new(base_urls: SusApiUrls) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            urls: base_urls,
            client,
        }
    }

    // Simulação simplificada de retry e backoff
    async fn request_with_retry<F, Fut>(&self, operation: F) -> Result<Value, ApiError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let mut delays = vec![0, 1, 2, 4];

        loop {
            let delay = delays.remove(0);
            if delay > 0 {
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }

            match operation().await {
                Ok(resp) => {
                    match resp.status() {
                        StatusCode::OK | StatusCode::CREATED => {
                            return resp
                                .json::<Value>()
                                .await
                                .map_err(|e| ApiError::Unknown(e.to_string()));
                        }
                        StatusCode::NOT_FOUND => return Err(ApiError::NotFound),
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                            return Err(ApiError::Auth("Acesso negado".to_string()));
                        }
                        s if s.is_server_error() => {
                            if delays.is_empty() {
                                return Err(ApiError::ServerError(s.to_string()));
                            }
                            continue; // Retry
                        }
                        s => return Err(ApiError::Unknown(format!("Status não tratado: {}", s))),
                    }
                }
                Err(e) => {
                    if delays.is_empty() {
                        return Err(ApiError::Network(e.to_string()));
                    }
                    continue; // Retry
                }
            }
        }
    }

    pub async fn get_estabelecimento(&self, cnes: &str) -> Result<Value, ApiError> {
        let url = format!("{}/estabelecimentos/{}", self.urls.cnes, cnes);
        self.request_with_retry(|| self.client.get(&url).send())
            .await
    }

    pub async fn register_transplante(&self, data: Value) -> Result<Value, ApiError> {
        let url = format!("{}/transplantes", self.urls.snt);
        self.request_with_retry(|| self.client.post(&url).json(&data).send())
            .await
    }

    pub async fn register_atendimento(&self, data: Value) -> Result<Value, ApiError> {
        let url = format!("{}/atendimentos", self.urls.esus_ab);
        self.request_with_retry(|| self.client.post(&url).json(&data).send())
            .await
    }

    pub async fn register_solicitacao(&self, data: Value) -> Result<Value, ApiError> {
        let url = format!("{}/regulacao/solicitacoes", self.urls.sisreg);
        self.request_with_retry(|| self.client.post(&url).json(&data).send())
            .await
    }
}
