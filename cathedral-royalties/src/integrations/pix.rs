use reqwest::Client;
use serde::{Serialize, Deserialize};
use crate::integrations::pix_openapi::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixPaymentRequest {
    pub amount: f64,
    pub description: String,
    pub payer_name: Option<String>,
    pub payer_document: Option<String>,
    pub expiration_seconds: Option<u32>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixStatus {
    Paid,
    Expired,
    Cancelled,
}

#[derive(Debug, Serialize)]
pub struct PixRequest {
    pub amount: f64,
    pub description: String,
    pub payer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PixResponse {
    pub qr_code: String,
    pub copy_paste: String,
    pub transaction_id: String,
}

pub struct PixGateway {
    client: Client,
    pub base_url: String,
    api_key: String,
    merchant_id: String,
}

impl PixGateway {
    pub fn new(base_url: &str, api_key: &str, merchant_id: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            merchant_id: merchant_id.to_string(),
        }
    }

    pub async fn create_payment(&self, req: &PixPaymentRequest) -> Result<PixResponse, String> {
        let response = self.client
            .post(&format!("{}/pix/qrcode", self.base_url))
            .header("Authorization", &self.api_key)
            .json(&PixRequest { amount: req.amount, description: req.description.to_string(), payer: None })
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let data: PixResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(data)
    }

    pub async fn create_pix_payment(&self, amount: f64, description: &str) -> Result<PixResponse, String> {
        let response = self.client
            .post(&format!("{}/pix/qrcode", self.base_url))
            .header("Authorization", &self.api_key)
            .json(&PixRequest { amount, description: description.to_string(), payer: None })
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let data: PixResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(data)
    }
}

pub struct OpenFinanceClient {
    client: Client,
    base_url: String,
    client_id: String,
    client_secret: String,
}

impl OpenFinanceClient {
    pub fn new(base_url: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    pub async fn transfer_pix(
        &self,
        consent: &OpenFinanceConsent,
        pix_key: &str,
        amount: f64,
        description: &str,
    ) -> Result<OpenFinanceTransferResponse, String> {
        let req = OpenFinanceTransferRequest {
            pix_key: pix_key.to_string(),
            amount,
            description: description.to_string(),
            consent_id: consent.consent_id.clone(),
        };

        let response = self.client
            .post(&format!("{}/transfers", self.base_url))
            .header("Authorization", format!("Bearer {}", consent.access_token))
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: OpenFinanceTransferResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(data)
    }
}
