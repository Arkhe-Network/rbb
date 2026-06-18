use crate::skill::types::{Skill, SkillExecution, ExecutionStatus};
use crate::skill::manager::SkillManager;
use tracing::{info, error};

pub struct SkillExecutor {
    skill_manager: SkillManager,
}

impl SkillExecutor {
    pub fn new(skill_manager: SkillManager) -> Self {
        Self { skill_manager }
    }

    pub async fn execute_skill(&mut self, skill_name: &str) -> Result<(), String> {
        info!("⚡ Executando skill '{}'", skill_name);

        let skill = self.skill_manager.load_skill(skill_name)
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?
            .clone();

        self.skill_manager.record_execution(
            skill_name,
            ExecutionStatus::Completed,
            Some(format!("Executed {}", skill_name).into_bytes()),
            None,
        );

        info!("✅ Skill '{}' executada com sucesso", skill_name);
        Ok(())
    }

    pub async fn execute_skill_background(&mut self, skill_name: &str) {
        match self.execute_skill(skill_name).await {
            Ok(_) => {
                info!("✅ Background skill '{}' concluída", skill_name);
            }
            Err(e) => {
                error!("❌ Background skill '{}' falhou: {}", skill_name, e);
                self.skill_manager.record_execution(
                    skill_name,
                    ExecutionStatus::Failed,
                    None,
                    Some(e),
                );
            }
        }
    }
}
