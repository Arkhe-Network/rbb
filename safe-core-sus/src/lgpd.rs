use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpoContact {
    pub nome: String,
    pub email: String,
    pub telefone: String,
}

pub struct LgpdCompliance {
    #[allow(dead_code)]
    dpo: DpoContact,
    measures: Vec<String>,
}

impl LgpdCompliance {
    pub fn new(dpo: DpoContact) -> Self {
        Self {
            dpo,
            measures: vec![
                "Criptografia".to_string(),
                "RBAC".to_string(),
                "Anonimização".to_string(),
                "Audit Log".to_string(),
                "Pseudonimização".to_string(),
            ],
        }
    }

    pub fn compliance_score(&self) -> u8 {
        // Base score dependendo das medidas implementadas.
        let base = (self.measures.len() * 20).min(100) as u8;
        base
    }
}
