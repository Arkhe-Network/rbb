//! Adapter para o SIGAA (Sistema Integrado de Gestão de Atividades Acadêmicas)
//!
//! O SIGAA é utilizado pela maioria das universidades federais brasileiras.
//! Este adapter traduz o JSON exportado pelo SIGAA para o schema `AcademicRecord`.
//!
//! # Mapeamento SIGAA → Safe-Core
//!
//! | SIGAA Campo             | Safe-Core AcademicRecord      |
//! |-------------------------|-------------------------------|
//! | idPessoa                | person_hash                   |
//! | nome                    | payload.nome                  |
//! | matricula               | payload.matricula              |
//! | idCurso                 | course_program                |
//! | idSituacao              | payload.status                |
//! | idNivel                 | record_type (Discente/Docente) |

use crate::adapter::{AcademicAdapter, AcademicRecord, AcademicRecordType, AdapterError};
use async_trait::async_trait;
use serde_json::Value;
use tracing::{info, warn};

/// Adapter para o SIGAA da UnB (Universidade de Brasília)
///
/// Exemplo de JSON esperado:
/// ```json
/// {
///   "idPessoa": "12345678",
///   "nome": "João da Silva",
///   "matricula": "2023012345",
///   "cpf": "123.456.789-00",
///   "idCurso": "PPGEC",
///   "idNivel": "D",
///   "idSituacao": "ATIVO",
///   "programa": "Programa de Pós-Graduação em Engenharia Civil"
/// }
/// ```
pub struct SigaaAdapter {
    /// Instituição de ensino (ex: "UnB")
    institution: String,
    /// Prefixo para hashing de identificadores
    salt: Option<String>,
}

impl SigaaAdapter {
    pub fn new(institution: impl Into<String>) -> Self {
        Self {
            institution: institution.into(),
            salt: None,
        }
    }

    pub fn with_salt(mut self, salt: impl Into<String>) -> Self {
        self.salt = Some(salt.into());
        self
    }
}

#[async_trait]
impl AcademicAdapter for SigaaAdapter {
    fn id(&self) -> &str {
        "adapter.sigaa.unb.v1"
    }

    fn source_system(&self) -> &str {
        "SIGAA"
    }

    async fn translate(&self, raw_data: &Value) -> Result<AcademicRecord, AdapterError> {
        // Extrair campos obrigatórios
        let id_pessoa = raw_data
            .get("idPessoa")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdapterError::MissingField("idPessoa".into()))?;

        let id_curso = raw_data
            .get("idCurso")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AdapterError::MissingField("idCurso".into()))?;

        let id_nivel = raw_data
            .get("idNivel")
            .and_then(|v| v.as_str())
            .unwrap_or("D"); // Default: Discente

        // Extrair CPF para pseudonimização (se disponível)
        let cpf = raw_data
            .get("cpf")
            .and_then(|v| v.as_str())
            .unwrap_or(id_pessoa);

        // Determinar tipo de registro
        let record_type = match id_nivel {
            "D" | "M" | "G" => AcademicRecordType::Discente,
            "P" | "PROF" => AcademicRecordType::Docente,
            "T" => AcademicRecordType::Tecnico,
            _ => AcademicRecordType::Discente,
        };

        // Extrair programa
        let course_program = raw_data
            .get("idCurso")
            .and_then(|v| v.as_str())
            .unwrap_or(id_curso)
            .to_string();

        // Extrair grande área (se disponível)
        let knowledge_area = raw_data
            .get("grandeArea")
            .and_then(|v| v.as_str())
            .unwrap_or("Não Informada")
            .to_string();

        // Construir AcademicRecord com pseudonimização
        let person_hash = if let Some(salt) = &self.salt {
            // Com salt: mais seguro
            let combined = format!("{}:{}", salt, cpf);
            self.pseudonymize(&combined)
        } else {
            self.pseudonymize(cpf)
        };

        // Hash da instituição (CNPJ ou nome normalizado)
        let institution_hash = self.pseudonymize(&self.institution);

        Ok(AcademicRecord {
            institution_hash,
            person_hash,
            course_program,
            knowledge_area,
            record_type,
            payload: raw_data.clone(),
            source_system: self.source_system().to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    async fn validate_capes_rules(&self, record: &AcademicRecord) -> Result<bool, AdapterError> {
        // Regras específicas do SIGAA UnB
        let payload = &record.payload;

        // Regra 1: Situação deve ser ATIVO
        if let Some(situacao) = payload.get("idSituacao").and_then(|v| v.as_str()) {
            if situacao != "ATIVO" && situacao != "MATRICULADO" {
                warn!(
                    "Discente com situação '{}' não elegível para validação CAPES",
                    situacao
                );
                return Ok(false);
            }
        }

        // Regra 2: Nível deve ser D (Doutorado) ou M (Mestrado)
        if let Some(nivel) = payload.get("idNivel").and_then(|v| v.as_str()) {
            if nivel != "D" && nivel != "M" {
                warn!("Nível '{}' não é elegível para CAPES", nivel);
                return Ok(false);
            }
        }

        // Regra 3: Programa deve ter conceito CAPES ≥ 3 (simulação)
        // Em produção, isso consultaria uma API do MEC/CAPES
        let program = &record.course_program;
        info!(program = %program, "Validando conceito CAPES");

        // Simulação: apenas programas conhecidos passam
        let known_programs = ["PPGEC", "PPGCA", "PPGEE", "PPGM", "PPGQ"];
        if !known_programs.contains(&program.as_str()) {
            warn!(
                "Programa '{}' não encontrado na lista de conhecidos",
                program
            );
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sigaa_translate_discente() {
        let adapter = SigaaAdapter::new("UnB");

        let raw_data = serde_json::json!({
            "idPessoa": "12345678",
            "nome": "João da Silva",
            "matricula": "2023012345",
            "cpf": "123.456.789-00",
            "idCurso": "PPGEC",
            "idNivel": "D",
            "idSituacao": "ATIVO",
            "programa": "Programa de Pós-Graduação em Engenharia Civil"
        });

        let record = adapter.translate(&raw_data).await.unwrap();

        assert_eq!(record.record_type, AcademicRecordType::Discente);
        assert_eq!(record.course_program, "PPGEC");
        assert_ne!(record.person_hash, "123.456.789-00");
        assert!(record.person_hash.len() >= 32);
        assert_eq!(record.source_system, "SIGAA");
        assert!(record.institution_hash.len() >= 32);
    }

    #[tokio::test]
    async fn test_sigaa_translate_docente() {
        let adapter = SigaaAdapter::new("UnB");

        let raw_data = serde_json::json!({
            "idPessoa": "87654321",
            "nome": "Maria Oliveira",
            "cpf": "987.654.321-00",
            "idCurso": "PPGEC",
            "idNivel": "P",
            "idSituacao": "ATIVO"
        });

        let record = adapter.translate(&raw_data).await.unwrap();
        assert_eq!(record.record_type, AcademicRecordType::Docente);
        assert_eq!(record.course_program, "PPGEC");
    }

    #[tokio::test]
    async fn test_sigaa_validate_capes_rules() {
        let adapter = SigaaAdapter::new("UnB");

        let raw_data = serde_json::json!({
            "idPessoa": "12345678",
            "idCurso": "PPGEC",
            "idNivel": "D",
            "idSituacao": "ATIVO"
        });

        let record = adapter.translate(&raw_data).await.unwrap();
        let is_valid = adapter.validate_capes_rules(&record).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_sigaa_inactive_student_fails_validation() {
        let adapter = SigaaAdapter::new("UnB");

        let raw_data = serde_json::json!({
            "idPessoa": "12345678",
            "idCurso": "PPGEC",
            "idNivel": "D",
            "idSituacao": "TRANCADO"
        });

        let record = adapter.translate(&raw_data).await.unwrap();
        let is_valid = adapter.validate_capes_rules(&record).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_sigaa_unknown_program_fails_validation() {
        let adapter = SigaaAdapter::new("UnB");

        let raw_data = serde_json::json!({
            "idPessoa": "12345678",
            "idCurso": "PPGXYZ",
            "idNivel": "D",
            "idSituacao": "ATIVO"
        });

        let record = adapter.translate(&raw_data).await.unwrap();
        let is_valid = adapter.validate_capes_rules(&record).await.unwrap();
        assert!(!is_valid);
    }
}
