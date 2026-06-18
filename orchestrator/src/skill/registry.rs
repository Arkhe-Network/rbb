use crate::skill::types::{Skill, SkillType};
use crate::skill::manager::SkillManager;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct SkillRegistry {
    npub: String,
    cache: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new(npub: String, _relays: Vec<String>) -> Self {
        Self {
            npub,
            cache: HashMap::new(),
        }
    }

    pub async fn publish_skill(&mut self, skill: &Skill) -> Result<String, String> {
        self.cache.insert(skill.name.clone(), skill.clone());
        info!("📡 Skill '{}' publicada no registro", skill.name);
        Ok(format!("hash-{}", skill.name))
    }

    pub async fn fetch_skill(&mut self, name: &str) -> Option<Skill> {
        self.cache.get(name).cloned()
    }

    pub async fn list_skills(&self) -> Vec<String> {
        vec![
            "grill-me".to_string(),
            "to-prd".to_string(),
            "diagnose".to_string(),
            "tdd".to_string(),
            "improve-architecture".to_string(),
            "triage".to_string(),
        ]
    }

    pub async fn import_skill(&mut self, name: &str, manager: &mut SkillManager) -> Result<(), String> {
        let skill = self.fetch_skill(name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada no registro", name))?;

        manager.save_skill(&skill)?;
        info!("✅ Skill '{}' importada do registro", name);
        Ok(())
    }
}
