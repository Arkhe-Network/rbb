//! ARKHE × DeSciOS — Integração para Ciência Descentralizada v0.2.0
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

pub mod assistant_guardrails;
pub mod error;
pub mod nodes_desci;
pub mod orcid;
pub mod plugin_governance;
pub mod publishing;
pub mod sei_giga;
pub mod workflow_traceability;

// Re-exports principais
pub use assistant_guardrails::{
    AssistantContext, DeSciAssistantGuardrails, GuardrailCategory, GuardrailCheckResult,
    GuardrailConfig, PiiCheckResult, PiiMasker, PiiType, Redaction,
};
pub use error::{DesciError, Result};
pub use plugin_governance::{PluginManifest, PluginValidator, ValidationCheck, ValidationResult};
pub use publishing::{DatasetMetadata, IpfsPublishResult, PublishResult, WormGraphNotifier};
#[cfg(feature = "ipfs")]
pub use publishing::{DeSciPublisher, IpfsClient};
pub use workflow_traceability::{
    ScientificWorkflowTrace, StepId, StepStatus, WorkflowStep, WorkflowType,
};

#[cfg(feature = "ipfs")]
pub use nodes_desci::NodesDesciClient;
pub use nodes_desci::{NodeDataset, NodeInfo, NodeRegistry, NodeSearchResult, NodeStatus};

#[cfg(feature = "orcid")]
pub use orcid::OrcidClient;
pub use orcid::{
    build_did_document, create_attestation, derive_did, verify_attestation, DidDocument,
    OrcidAttestation, OrcidDID, OrcidProfile, DID_ORCID_PREFIX,
};

#[cfg(feature = "sei-giga")]
pub use sei_giga::SeiGigaClient;
pub use sei_giga::{
    compute_anchor_hash, AnchorEvent, AnchorInfo, AnchorMsg, IdentityInfo, RegisterIdentityMsg,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
