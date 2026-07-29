//! Configuração para integração Flock.
//!
//! ✅ P4: Definido localmente, não depende de arkhe-flock internals.

use std::path::PathBuf;

/// Configuração do Flock para provas de governança.
#[derive(Debug, Clone)]
pub struct FlockConfig {
    /// Caminho do binário flock_chain (ou None para PATH).
    pub flock_bin: Option<PathBuf>,
    /// Função de hash ("blake3", "sha256", etc).
    pub hash_function: String,
    /// Número de steps para a prova.
    pub steps: u64,
}

impl Default for FlockConfig {
    fn default() -> Self {
        Self {
            flock_bin: None,
            hash_function: "blake3".into(),
            steps: 256,
        }
    }
}

pub struct FlockError(String);

pub type FlockResult<T> = Result<T, FlockError>;

pub fn prove_governance(config: &FlockConfig, proof_data: &[u8]) -> FlockResult<String> {
    let proof_hash = hex::encode(blake3::hash(&proof_data).as_bytes()); // ✅ P5: consistente
    Ok(proof_hash)
}
