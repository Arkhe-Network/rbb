// src/integrations/bittensor/sn4_targon.rs
//! Integração com a SN4 (Targon) para geração de provas ZK em TEE.

use super::*;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use std::sync::Arc;

// ─── Tipos ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TargonZKRequest {
    pub circuit_wasm: String,            // Circuito WASM (base64)
    pub proving_key: String,             // Proving key (base64)
    pub public_inputs: Vec<String>,      // Inputs públicos (hex)
    pub private_inputs: Vec<String>,     // Inputs privados (hex)
    pub tee_type: Option<String>,        // "sgx", "sev", "nitro"
    pub require_attestation: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargonZKResponse {
    pub proof: String,                   // Prova ZK (hex)
    pub attestation: Option<String>,     // Atestação TEE (hex)
    pub performance: TargonPerformance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargonPerformance {
    pub setup_time_ms: u64,
    pub proving_time_ms: u64,
    pub verification_time_ms: u64,
    pub memory_usage_mb: u32,
}

// ─── Cliente SN4 ──────────────────────────────────────────────────────────

pub struct TargonClient {
    bittensor: Arc<BittensorClient>,
    subnet_id: u16,
}

impl TargonClient {
    pub fn new(bittensor: Arc<BittensorClient>) -> Self {
        Self {
            bittensor,
            subnet_id: 4,
        }
    }

    /// Gera uma prova ZK em TEE
    pub async fn generate_zk_proof(
        &self,
        circuit_wasm: &[u8],
        proving_key: &[u8],
        public_inputs: &[String],
        private_inputs: &[String],
    ) -> Result<TargonZKResponse> {
        let request = TargonZKRequest {
            circuit_wasm: base64::encode(circuit_wasm),
            proving_key: base64::encode(proving_key),
            public_inputs: public_inputs.to_vec(),
            private_inputs: private_inputs.to_vec(),
            tee_type: Some("sev".to_string()),
            require_attestation: Some(true),
        };

        let responses = self.bittensor
            .query_subnet_with_fallback::<_, TargonZKResponse>(
                self.subnet_id,
                "zk_prove",
                &request,
                2,
                1,
            )
            .await?;

        let best = &responses[0];
        best.data.clone().ok_or_else(|| anyhow!("Resposta vazia da SN4"))
    }
}
