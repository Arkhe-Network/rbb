//! Piloto UnB — Implementação de integração com a Universidade de Brasília
//!
//! Este módulo contém a configuração específica para o piloto com a UnB,
//! incluindo mapeamento de campos, regras de validação e endpoints de teste.

use crate::adapter::{AcademicAdapter, AcademicRecord, AdapterError};
use crate::adapters::sigaa::SigaaAdapter;
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

/// Configuração do piloto UnB
#[derive(Debug, Clone)]
pub struct UnbPilotConfig {
    /// URL do endpoint de teste do SIGAA UnB
    pub sigaa_test_url: String,
    /// Chave de API para acesso (se necessário)
    pub api_key: Option<String>,
    /// Salt para pseudonimização
    pub salt: String,
    /// Lista de programas CAPES com conceito ≥ 3
    pub capes_programs: Vec<String>,
}

impl Default for UnbPilotConfig {
    fn default() -> Self {
        Self {
            sigaa_test_url: "https://sigaa.unb.br/api/test".to_string(),
            api_key: None,
            salt: "unb-salt-2026".to_string(),
            capes_programs: vec![
                "PPGEC".to_string(),
                "PPGCA".to_string(),
                "PPGEE".to_string(),
                "PPGM".to_string(),
                "PPGQ".to_string(),
            ],
        }
    }
}

/// Adapter específico para o piloto UnB
pub struct UnbPilotAdapter {
    pub inner: SigaaAdapter,
    pub config: UnbPilotConfig,
    /// Cache de programas CAPES (para validação)
    capes_cache: std::collections::HashMap<String, u8>,
}

impl UnbPilotAdapter {
    pub fn new(config: UnbPilotConfig) -> Self {
        let inner = SigaaAdapter::new("UnB").with_salt(&config.salt);
        let mut capes_cache = HashMap::new();
        for program in &config.capes_programs {
            capes_cache.insert(program.clone(), 4); // Simula conceito CAPES
        }
        Self {
            inner,
            config,
            capes_cache,
        }
    }

    /// Valida conceito CAPES de um programa
    pub fn get_capes_concept(&self, program: &str) -> Option<u8> {
        // Em produção, isso consultaria a API do MEC/CAPES
        // Por enquanto, usa cache local
        self.capes_cache.get(program).copied()
    }

    /// Processa um lote de registros do SIGAA UnB
    pub async fn process_batch(
        &self,
        records: Vec<Value>,
    ) -> Vec<Result<AcademicRecord, AdapterError>> {
        let mut results = Vec::new();
        for record in records {
            let result = self.inner.translate(&record).await;
            results.push(result);
        }
        results
    }
}

/// Função para validação específica da UnB
pub async fn validate_unb_record(record: &AcademicRecord) -> Result<bool, AdapterError> {
    // Regras adicionais do piloto UnB:
    // 1. O programa deve estar credenciado na CAPES
    // 2. O discente deve ter matrícula ativa no SIGAA
    // 3. Os dados devem estar consistentes com o sistema acadêmico

    let payload = &record.payload;

    // Verificar matrícula ativa
    if let Some(status) = payload.get("idSituacao").and_then(|v| v.as_str()) {
        if status != "ATIVO" && status != "MATRICULADO" {
            warn!("Matrícula inativa no SIGAA UnB: {}", record.person_hash);
            return Ok(false);
        }
    }

    // Verificar programa credenciado
    // Hack to check the ID in case it's different from the program name.
    let program = record
        .payload
        .get("idCurso")
        .and_then(|v| v.as_str())
        .unwrap_or(&record.course_program);
    let pilot = UnbPilotAdapter::new(UnbPilotConfig::default());
    if pilot.get_capes_concept(program).is_none() {
        warn!("Programa {} não encontrado na lista CAPES da UnB", program);
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_unb_pilot_validation() {
        let config = UnbPilotConfig::default();
        let adapter = UnbPilotAdapter::new(config);

        let raw_data = json!({
            "idPessoa": "12345678",
            "nome": "João da Silva",
            "matricula": "2023012345",
            "cpf": "123.456.789-00",
            "idCurso": "PPGEC",
            "idNivel": "D",
            "idSituacao": "ATIVO",
            "programa": "Programa de Pós-Graduação em Engenharia Civil"
        });

        let record = adapter.inner.translate(&raw_data).await.unwrap();
        assert_eq!(record.course_program, "PPGEC");
        assert_eq!(
            record.record_type,
            super::super::super::adapter::AcademicRecordType::Discente
        );

        let is_valid = validate_unb_record(&record).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_unb_pilot_inactive_fails() {
        let config = UnbPilotConfig::default();
        let adapter = UnbPilotAdapter::new(config);

        let raw_data = json!({
            "idPessoa": "12345678",
            "idCurso": "PPGEC",
            "idNivel": "D",
            "idSituacao": "TRANCADO"
        });

        let record = adapter.inner.translate(&raw_data).await.unwrap();
        let is_valid = validate_unb_record(&record).await.unwrap();
        assert!(!is_valid);
    }
}
