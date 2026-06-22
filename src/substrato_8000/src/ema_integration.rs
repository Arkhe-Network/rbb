//! src/substrato_8000/ema_integration.rs
//! Enterprise-Managed Authorization (EMA) — Headroom Bridge
//! Autenticação e autorização enterprise para operações de compressão
//!
//! Selo: CATHEDRAL-ARKHE-8000-EMA-INTEGRATION-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use rand::Rng;
use rand::RngExt;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use thiserror::Error;
use sha2::{Sha256, Digest};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng;

/// ============================================================
/// 1. EMA TOKEN SYSTEM
/// ============================================================

/// Token EMA para autenticação de operações Headroom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaToken {
    /// ID único do token
    pub token_id: String,
    /// ID do holder (ORCID, email, ou DID)
    pub holder_id: String,
    /// Escopos de permissão
    pub scopes: Vec<EmaScope>,
    /// Timestamp de emissão
    pub issued_at: i64,
    /// Timestamp de expiração
    pub expires_at: i64,
    /// Emissor do token (HSM ou autoridade enterprise)
    pub issuer: String,
    /// Assinatura Ed25519
    pub signature: Vec<u8>,
    /// Nível de classificação (para dados sensíveis)
    pub classification_level: ClassificationLevel,
    /// Domínios permitidos
    pub allowed_domains: Vec<String>,
    /// Quota de compressão (bytes/mês)
    pub compression_quota_bytes: u64,
    /// Quota usada
    pub quota_used_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EmaScope {
    /// Comprimir contexto
    HeadroomCompress,
    /// Recuperar via CCR
    HeadroomRetrieve,
    /// Ler estatísticas
    HeadroomStats,
    /// Administrar configurações
    HeadroomAdmin,
    /// Acesso a métricas Prometheus
    HeadroomMetrics,
    /// Cross-agent memory read
    CrossAgentRead,
    /// Cross-agent memory write
    CrossAgentWrite,
    /// ZKP proof generation
    ZkpGenerate,
    /// ZKP proof verification
    ZkpVerify,
    /// Proxy mode access
    ProxyAccess,
    /// Custom scope
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClassificationLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
    Secret,
}

/// ============================================================
/// 2. EMA VERIFIER
/// ============================================================

pub struct EmaVerifier {
    /// Chaves públicas de emissores confiáveis
    trusted_issuers: Arc<RwLock<HashMap<String, VerifyingKey>>>,
    /// Tokens revogados (CRL)
    revocation_list: Arc<RwLock<HashMap<String, i64>>>,
    /// Quotas por token
    quota_tracker: Arc<RwLock<HashMap<String, u64>>>,
    /// Configuração
    config: EmaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaConfig {
    /// Tempo máximo de vida do token (segundos)
    pub max_token_lifetime: i64,
    /// Se verifica CRL
    pub check_revocation: bool,
    /// Se verifica quota
    pub enforce_quota: bool,
    /// Se verifica classificação
    pub enforce_classification: bool,
    /// Domínios permitidos por padrão
    pub default_allowed_domains: Vec<String>,
    /// Buffer de quota (%) antes de warning
    pub quota_warning_threshold: f64,
}

impl Default for EmaConfig {
    fn default() -> Self {
        Self {
            max_token_lifetime: 3600 * 24, // 24 horas
            check_revocation: true,
            enforce_quota: true,
            enforce_classification: true,
            default_allowed_domains: vec!["cathedral.local".to_string()],
            quota_warning_threshold: 0.8,
        }
    }
}

/// Resultado da verificação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaVerificationResult {
    pub valid: bool,
    pub token_id: String,
    pub holder_id: String,
    pub granted_scopes: Vec<EmaScope>,
    pub classification: ClassificationLevel,
    pub quota_remaining: u64,
    pub quota_warning: bool,
    pub expires_in_seconds: i64,
    pub verification_time_ms: u64,
}

impl EmaVerifier {
    pub fn new(config: EmaConfig) -> Self {
        Self {
            trusted_issuers: Arc::new(RwLock::new(HashMap::new())),
            revocation_list: Arc::new(RwLock::new(HashMap::new())),
            quota_tracker: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// ============================================================
    /// 2.1 VERIFY TOKEN
    /// ============================================================

    pub async fn verify_token(
        &self,
        token: &EmaToken,
        required_scopes: &[EmaScope],
        operation_bytes: u64,
    ) -> Result<EmaVerificationResult, EmaError> {
        let start = std::time::Instant::now();

        // 1. Verifica expiração
        let now = Utc::now().timestamp();
        if token.expires_at < now {
            return Err(EmaError::TokenExpired {
                expired_at: token.expires_at,
                now,
            });
        }

        // 2. Verifica lifetime
        let lifetime = token.expires_at - token.issued_at;
        if lifetime > self.config.max_token_lifetime {
            return Err(EmaError::TokenLifetimeExceeded {
                lifetime,
                max: self.config.max_token_lifetime,
            });
        }

        // 3. Verifica CRL
        if self.config.check_revocation {
            let crl = self.revocation_list.read().await;
            if crl.contains_key(&token.token_id) {
                return Err(EmaError::TokenRevoked(token.token_id.clone()));
            }
        }

        // 4. Verifica assinatura
        let issuers = self.trusted_issuers.read().await;
        let issuer_key = issuers.get(&token.issuer)
            .ok_or(EmaError::UntrustedIssuer(token.issuer.clone()))?;

        let message = self.token_signing_payload(token);
        let signature = Signature::from_slice(&token.signature)
            .map_err(|e| EmaError::InvalidSignature(e.to_string()))?;

        issuer_key.verify_strict(&message, &signature)
            .map_err(|e| EmaError::SignatureVerificationFailed(e.to_string()))?;

        drop(issuers);

        // 5. Verifica escopos
        let mut granted = vec![];
        for required in required_scopes {
            if token.scopes.contains(required) {
                granted.push(required.clone());
            } else {
                return Err(EmaError::InsufficientScope {
                    required: format!("{:?}", required),
                    held: token.scopes.iter().map(|s| format!("{:?}", s)).collect(),
                });
            }
        }

        // 6. Verifica quota
        let mut quota_remaining = token.compression_quota_bytes;
        let mut quota_warning = false;

        if self.config.enforce_quota {
            let mut tracker = self.quota_tracker.write().await;
            let used = tracker.entry(token.token_id.clone()).or_insert(0);
            *used += operation_bytes;

            if *used > token.compression_quota_bytes {
                return Err(EmaError::QuotaExceeded {
                    used: *used,
                    quota: token.compression_quota_bytes,
                });
            }

            quota_remaining = token.compression_quota_bytes - *used;
            quota_warning = *used as f64 > token.compression_quota_bytes as f64 * self.config.quota_warning_threshold;
        }

        // 7. Verifica classificação
        if self.config.enforce_classification {
            // Em produção: verificar se operação é compatível com classification do token
            // e do conteúdo sendo comprimido
        }

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(EmaVerificationResult {
            valid: true,
            token_id: token.token_id.clone(),
            holder_id: token.holder_id.clone(),
            granted_scopes: granted,
            classification: token.classification_level.clone(),
            quota_remaining,
            quota_warning,
            expires_in_seconds: token.expires_at - now,
            verification_time_ms: elapsed,
        })
    }

    /// ============================================================
    /// 2.2 TOKEN ISSUANCE (HSM)
    /// ============================================================

    /// Emite novo token (requer chave do emissor)
    pub async fn issue_token(
        &self,
        issuer_key: &SigningKey,
        holder_id: &str,
        scopes: Vec<EmaScope>,
        classification: ClassificationLevel,
        lifetime_seconds: i64,
        quota_bytes: u64,
    ) -> Result<EmaToken, EmaError> {
        let token_id = format!("ema_{}_{}", holder_id, Utc::now().timestamp_millis());
        let now = Utc::now().timestamp();

        let mut token = EmaToken {
            token_id: token_id.clone(),
            holder_id: holder_id.to_string(),
            scopes,
            issued_at: now,
            expires_at: now + lifetime_seconds,
            issuer: hex::encode(issuer_key.verifying_key().as_bytes()),
            signature: vec![],
            classification_level: classification,
            allowed_domains: self.config.default_allowed_domains.clone(),
            compression_quota_bytes: quota_bytes,
            quota_used_bytes: 0,
        };

        // Assina
        let payload = self.token_signing_payload(&token);
        let signature = issuer_key.sign(&payload);
        token.signature = signature.to_bytes().to_vec();

        // Registra emissor
        {
            let mut issuers = self.trusted_issuers.write().await;
            issuers.insert(token.issuer.clone(), issuer_key.verifying_key());
        }

        Ok(token)
    }

    /// ============================================================
    /// 2.3 REVOCATION
    /// ============================================================

    /// Revoga token (CRL)
    pub async fn revoke_token(&self, token_id: &str) -> Result<(), EmaError> {
        let mut crl = self.revocation_list.write().await;
        crl.insert(token_id.to_string(), Utc::now().timestamp());

        tracing::info!("🚫 Token revoked: {}", token_id);
        Ok(())
    }

    /// ============================================================
    /// 2.4 UTILITÁRIOS
    /// ============================================================

    fn token_signing_payload(&self, token: &EmaToken) -> Vec<u8> {
        format!(
            "{}:{}:{:?}:{}:{}",
            token.token_id,
            token.holder_id,
            token.scopes,
            token.issued_at,
            token.expires_at
        ).into_bytes()
    }

    pub async fn get_quota_usage(&self, token_id: &str) -> Option<u64> {
        let tracker = self.quota_tracker.read().await;
        tracker.get(token_id).copied()
    }

    pub async fn reset_quota(&self, token_id: &str) {
        let mut tracker = self.quota_tracker.write().await;
        tracker.insert(token_id.to_string(), 0);
    }
}

/// ============================================================
/// 3. EMA MIDDLEWARE (Axum)
/// ============================================================

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn ema_auth_middleware<B>(
    State(verifier): State<Arc<EmaVerifier>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Extrai token do header
    let token_str = request.headers()
        .get("x-ema-token")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token: EmaToken = serde_json::from_str(token_str)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Determina escopos necessários baseado no path
    let required_scopes = determine_required_scopes(request.uri().path());

    // Verifica
    let result = verifier.verify_token(&token, &required_scopes, 0).await
        .map_err(|e| {
            tracing::warn!("EMA verification failed: {}", e);
            StatusCode::FORBIDDEN
        })?;

    if !result.valid {
        return Err(StatusCode::FORBIDDEN);
    }

    // Adiciona informações de verificação aos headers
    let mut request = request;
    request.headers_mut().insert(
        "x-ema-verified",
        "true".parse().unwrap(),
    );
    request.headers_mut().insert(
        "x-ema-holder",
        result.holder_id.parse().unwrap(),
    );

    Ok(next.run(request).await)
}

fn determine_required_scopes(path: &str) -> Vec<EmaScope> {
    match path {
        p if p.starts_with("/mcp/v1/tools/call") => vec![EmaScope::HeadroomCompress, EmaScope::HeadroomRetrieve],
        p if p.starts_with("/mcp/v1/tools/list") => vec![EmaScope::HeadroomStats],
        p if p.starts_with("/metrics") => vec![EmaScope::HeadroomMetrics],
        p if p.starts_with("/proxy") => vec![EmaScope::ProxyAccess],
        _ => vec![EmaScope::HeadroomCompress],
    }
}

/// ============================================================
/// 4. ERROS
/// ============================================================

#[derive(Debug, Error)]
pub enum EmaError {
    #[error("Token expired at {expired_at}, now {now}")]
    TokenExpired { expired_at: i64, now: i64 },
    #[error("Token lifetime {lifetime}s exceeds maximum {max}s")]
    TokenLifetimeExceeded { lifetime: i64, max: i64 },
    #[error("Token revoked: {0}")]
    TokenRevoked(String),
    #[error("Untrusted issuer: {0}")]
    UntrustedIssuer(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    #[error("Insufficient scope: required {required}, held {held:?}")]
    InsufficientScope { required: String, held: Vec<String> },
    #[error("Quota exceeded: used {used}, quota {quota}")]
    QuotaExceeded { used: u64, quota: u64 },
    #[error("Classification mismatch: token {token_level}, content {content_level}")]
    ClassificationMismatch { token_level: String, content_level: String },
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// ============================================================
/// 5. TESTES
/// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_issue_and_verify() {
        let verifier = EmaVerifier::new(EmaConfig::default());
        let mut csprng = OsRng;
        let issuer_key = SigningKey::generate(&mut csprng);

        let token = verifier.issue_token(
            &issuer_key,
            "ORCID_0009-0005-2697-4668",
            vec![EmaScope::HeadroomCompress, EmaScope::HeadroomStats],
            ClassificationLevel::Confidential,
            3600,
            1_000_000,
        ).await.unwrap();

        assert_eq!(token.holder_id, "ORCID_0009-0005-2697-4668");
        assert_eq!(token.scopes.len(), 2);

        // Verifica
        let result = verifier.verify_token(
            &token,
            &[EmaScope::HeadroomCompress],
            100,
        ).await.unwrap();

        assert!(result.valid);
        assert_eq!(result.holder_id, "ORCID_0009-0005-2697-4668");
        assert!(result.quota_remaining < 1_000_000);
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let verifier = EmaVerifier::new(EmaConfig::default());
        let mut csprng = OsRng;
        let issuer_key = SigningKey::generate(&mut csprng);

        let mut token = verifier.issue_token(
            &issuer_key,
            "test",
            vec![EmaScope::HeadroomCompress],
            ClassificationLevel::Public,
            1, // 1 segundo
            1000,
        ).await.unwrap();

        // Expira token
        token.expires_at = Utc::now().timestamp() - 1;

        let result = verifier.verify_token(&token, &[EmaScope::HeadroomCompress], 0).await;
        assert!(matches!(result, Err(EmaError::TokenExpired { .. })));
    }

    #[tokio::test]
    async fn test_insufficient_scope() {
        let verifier = EmaVerifier::new(EmaConfig::default());
        let mut csprng = OsRng;
        let issuer_key = SigningKey::generate(&mut csprng);

        let token = verifier.issue_token(
            &issuer_key,
            "test",
            vec![EmaScope::HeadroomStats], // Sem HeadroomCompress
            ClassificationLevel::Public,
            3600,
            1000,
        ).await.unwrap();

        let result = verifier.verify_token(&token, &[EmaScope::HeadroomCompress], 0).await;
        assert!(matches!(result, Err(EmaError::InsufficientScope { .. })));
    }

    #[tokio::test]
    async fn test_quota_enforcement() {
        let verifier = EmaVerifier::new(EmaConfig::default());
        let mut csprng = OsRng;
        let issuer_key = SigningKey::generate(&mut csprng);

        let token = verifier.issue_token(
            &issuer_key,
            "test",
            vec![EmaScope::HeadroomCompress],
            ClassificationLevel::Public,
            3600,
            100, // Quota muito baixa
        ).await.unwrap();

        // Primeira operação: OK
        let r1 = verifier.verify_token(&token, &[EmaScope::HeadroomCompress], 50).await;
        assert!(r1.is_ok());

        // Segunda operação: excede quota
        let r2 = verifier.verify_token(&token, &[EmaScope::HeadroomCompress], 60).await;
        assert!(matches!(r2, Err(EmaError::QuotaExceeded { .. })));
    }

    #[tokio::test]
    async fn test_revocation() {
        let verifier = EmaVerifier::new(EmaConfig::default());
        let mut csprng = OsRng;
        let issuer_key = SigningKey::generate(&mut csprng);

        let token = verifier.issue_token(
            &issuer_key,
            "test",
            vec![EmaScope::HeadroomCompress],
            ClassificationLevel::Public,
            3600,
            1000,
        ).await.unwrap();

        // Revoga
        verifier.revoke_token(&token.token_id).await.unwrap();

        // Tenta verificar
        let result = verifier.verify_token(&token, &[EmaScope::HeadroomCompress], 0).await;
        assert!(matches!(result, Err(EmaError::TokenRevoked(_))));
    }

    #[tokio::test]
    async fn test_quota_warning() {
        let mut config = EmaConfig::default();
        config.quota_warning_threshold = 0.5; // 50%

        let verifier = EmaVerifier::new(config);
        let mut csprng = OsRng;
        let issuer_key = SigningKey::generate(&mut csprng);

        let token = verifier.issue_token(
            &issuer_key,
            "test",
            vec![EmaScope::HeadroomCompress],
            ClassificationLevel::Public,
            3600,
            100,
        ).await.unwrap();

        // Usa 60% (acima do threshold de 50%)
        let result = verifier.verify_token(&token, &[EmaScope::HeadroomCompress], 60).await.unwrap();
        assert!(result.quota_warning);
    }
}
