//! ARKHE × DeSciOS — Integração para ciência descentralizada

pub mod assistant_guardrails;
pub mod error;
pub mod plugin_governance;
pub mod publishing;
pub mod workflow_traceability;

pub use assistant_guardrails::{AssistantContext, DeSciAssistantGuardrails, GuardrailError};
pub use error::{DesciError, Result};
pub use plugin_governance::{PluginManifest, PluginValidator, ValidationResult};
pub use publishing::{DatasetMetadata, DeSciPublisher};
pub use workflow_traceability::{ScientificWorkflowTrace, WorkflowStep};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
