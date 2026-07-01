//! Plugin Governance — validação com invariantes ARKHE

use arkhe_invariants::InvariantEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: String,
    pub signature: Option<String>,
    pub install_script: String,
    pub requested_permissions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub plugin_id: String,
    pub passed: bool,
    pub checks: Vec<ValidationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub invariant_id: String,
    pub passed: bool,
    pub message: String,
}

pub struct PluginValidator {
    engine: InvariantEngine,
    required_signatures: bool,
    max_permissions: usize,
    allowed_sources: HashSet<String>,
}

impl Default for PluginValidator {
    fn default() -> Self {
        Self {
            engine: InvariantEngine::new(),
            required_signatures: false,
            max_permissions: 3,
            allowed_sources: HashSet::new(),
        }
    }
}

impl PluginValidator {
    pub fn new(
        allowed_sources: Vec<String>,
        required_signatures: bool,
        _sandbox_enforced: bool,
        max_permissions: usize,
    ) -> Self {
        Self {
            engine: InvariantEngine::new(),
            required_signatures,
            max_permissions,
            allowed_sources: allowed_sources.into_iter().collect(),
        }
    }

    pub fn validate(
        &self,
        manifest: &PluginManifest,
    ) -> Result<ValidationResult, crate::error::DesciError> {
        let mut checks = Vec::new();
        let mut passed = true;

        // OWASP-003: Provenance
        if self.required_signatures && manifest.signature.is_none() {
            passed = false;
            checks.push(ValidationCheck {
                invariant_id: "OWASP-003".to_string(),
                passed: false,
                message: "Plugin not signed".to_string(),
            });
        } else {
            checks.push(ValidationCheck {
                invariant_id: "OWASP-003".to_string(),
                passed: true,
                message: "Signature OK".to_string(),
            });
        }

        // CNT-002: Workspace confinement
        let dangerous = ["/etc/passwd", "/root/", "sudo", "rm -rf"];
        if dangerous
            .iter()
            .any(|p| manifest.install_script.contains(p))
        {
            passed = false;
            checks.push(ValidationCheck {
                invariant_id: "CNT-002".to_string(),
                passed: false,
                message: "Dangerous command in install_script".to_string(),
            });
        } else {
            checks.push(ValidationCheck {
                invariant_id: "CNT-002".to_string(),
                passed: true,
                message: "No dangerous commands".to_string(),
            });
        }

        // OWASP-006: Least privilege
        if manifest.requested_permissions.len() > self.max_permissions {
            passed = false;
            checks.push(ValidationCheck {
                invariant_id: "OWASP-006".to_string(),
                passed: false,
                message: format!(
                    "Too many permissions: {}",
                    manifest.requested_permissions.len()
                ),
            });
        } else {
            checks.push(ValidationCheck {
                invariant_id: "OWASP-006".to_string(),
                passed: true,
                message: "Permissions OK".to_string(),
            });
        }

        // CNT-003: Tool Allowlist
        if !self.allowed_sources.is_empty() {
            let allowed = self
                .allowed_sources
                .iter()
                .any(|s| manifest.source.starts_with(s));
            if !allowed {
                passed = false;
                checks.push(ValidationCheck {
                    invariant_id: "CNT-003".to_string(),
                    passed: false,
                    message: format!("Source '{}' not in allowed sources", manifest.source),
                });
            } else {
                checks.push(ValidationCheck {
                    invariant_id: "CNT-003".to_string(),
                    passed: true,
                    message: "Source allowed".to_string(),
                });
            }
        }

        let _summary = if passed { "Valid" } else { "Failed" };
        if !passed {
            warn!(plugin = %manifest.name, "Plugin validation failed");
        }

        Ok(ValidationResult {
            plugin_id: manifest.id.clone(),
            passed,
            checks,
        })
    }

    pub fn validate_with_engine(
        &self,
        manifest: &PluginManifest,
    ) -> Result<ValidationResult, crate::error::DesciError> {
        let context = serde_json::json!({
            "plugin_id": manifest.id,
            "plugin_name": manifest.name,
            "source": manifest.source,
        });
        // Usar engine para validação adicional
        self.engine
            .validate_goal(&serde_json::to_string(&context).unwrap_or_default())
            .map_err(|e| crate::error::DesciError::InvariantViolation(e.to_string()))?;
        self.validate(manifest)
    }

    pub fn validate_batch(&self, manifests: &[PluginManifest]) -> Vec<ValidationResult> {
        manifests
            .iter()
            .filter_map(|m| self.validate(m).ok())
            .collect()
    }

    pub fn allowed_sources(&self) -> &HashSet<String> {
        &self.allowed_sources
    }

    pub fn add_allowed_source(&mut self, source: String) {
        self.allowed_sources.insert(source);
    }

    pub fn remove_allowed_source(&mut self, source: &str) {
        self.allowed_sources.remove(source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> PluginManifest {
        PluginManifest {
            id: "test-plugin-001".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            source: "https://github.com/example/plugin".to_string(),
            signature: Some("deadbeef".to_string()),
            install_script: "apt install -y samtools".to_string(),
            requested_permissions: vec!["network".to_string()],
            dependencies: vec![],
            checksum: Some("sha256:abc123".to_string()),
        }
    }

    #[test]
    fn test_valid_plugin_passes() {
        let validator = PluginValidator::default();
        let manifest = valid_manifest();
        let result = validator.validate(&manifest).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_missing_signature_when_required() {
        let mut validator = PluginValidator::default();
        validator.required_signatures = true;
        let mut manifest = valid_manifest();
        manifest.signature = None;
        let result = validator.validate(&manifest).unwrap();
        assert!(!result.passed);
        assert!(result
            .checks
            .iter()
            .any(|c| c.invariant_id == "OWASP-003" && !c.passed));
    }

    #[test]
    fn test_dangerous_install_script_blocked() {
        let validator = PluginValidator::default();
        let mut manifest = valid_manifest();
        manifest.install_script = "cat /etc/passwd".to_string();
        let result = validator.validate(&manifest).unwrap();
        assert!(!result.passed);
        assert!(result
            .checks
            .iter()
            .any(|c| c.invariant_id == "CNT-002" && !c.passed));
    }

    #[test]
    fn test_too_many_permissions_blocked() {
        let validator = PluginValidator::default();
        let mut manifest = valid_manifest();
        manifest.requested_permissions =
            vec!["network".into(), "fs".into(), "gpu".into(), "admin".into()];
        let result = validator.validate(&manifest).unwrap();
        assert!(!result.passed);
        assert!(result
            .checks
            .iter()
            .any(|c| c.invariant_id == "OWASP-006" && !c.passed));
    }

    #[test]
    fn test_disallowed_source_blocked() {
        let validator =
            PluginValidator::new(vec!["https://github.com".to_string()], false, true, 3);
        let mut manifest = valid_manifest();
        manifest.source = "https://evil.com/plugin".to_string();
        let result = validator.validate(&manifest).unwrap();
        assert!(!result.passed);
        assert!(result
            .checks
            .iter()
            .any(|c| c.invariant_id == "CNT-003" && !c.passed));
    }

    #[test]
    fn test_batch_validation() {
        let validator = PluginValidator::default();
        let mut bad = valid_manifest();
        bad.install_script = "rm -rf /".to_string();
        let results = validator.validate_batch(&[valid_manifest(), bad]);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
    }
}
