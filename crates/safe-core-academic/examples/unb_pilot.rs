//! Exemplo de uso do piloto UnB
//!
//! Este exemplo demonstra a integração do Safe-Core Academic
//! com o SIGAA da UnB, processando dados de discentes.

use safe_core_academic::adapter::AcademicAdapter;
use safe_core_academic::pilots::unb::{UnbPilotAdapter, UnbPilotConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuração do piloto
    let config = UnbPilotConfig::default();
    let adapter = UnbPilotAdapter::new(config);

    // Simular dados do SIGAA UnB
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

    // Traduzir para o schema Safe-Core
    let record = adapter.inner.translate(&raw_data).await?;
    println!("✅ Registro traduzido:");
    println!("  Programa: {}", record.course_program);
    println!("  Tipo: {:?}", record.record_type);
    println!("  Hash da pessoa: {}...", &record.person_hash[..16]);

    // Validar regras CAPES
    let is_valid = safe_core_academic::pilots::unb::validate_unb_record(&record).await?;
    println!("  Válido para CAPES: {}", is_valid);

    Ok(())
}
