use crate::skill::types::{ExecutionStatus};
use crate::skill::manager::SkillManager;
use crate::swarm::orchestrator::SwarmOrchestrator;
use crate::swarm::types::SwarmResult;
use tracing::{info, error};

pub struct SkillExecutor<'a> {
    orchestrator: SwarmOrchestrator,
    skill_manager: &'a mut SkillManager,
}

impl<'a> SkillExecutor<'a> {
    pub fn new(orchestrator: SwarmOrchestrator, skill_manager: &'a mut SkillManager) -> Self {
        Self { orchestrator, skill_manager }
    }

    pub async fn execute_skill(&mut self, skill_name: &str) -> Result<SwarmResult, String> {
        info!("⚡ Executando skill '{}' como SwarmSpec", skill_name);

        let skill = self.skill_manager.load_skill(skill_name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?
            .clone();

        let spec = skill.to_swarm_spec();

        let result = self.orchestrator.run_spec(spec).await?;

        self.skill_manager.record_execution(
            skill_name,
            ExecutionStatus::Completed,
            Some(format!("{:?}", result).into_bytes()),
            None,
        );

        info!("✅ Skill '{}' executada com sucesso ({} agentes)", skill_name, result.agent_count);
        Ok(result)
    }

    pub async fn execute_skill_background(&mut self, skill_name: &str) -> Result<(), String> {
        match self.execute_skill(skill_name).await {
            Ok(_) => {
                info!("✅ Background skill '{}' concluída", skill_name);
                Ok(())
            }
            Err(e) => {
                error!("❌ Background skill '{}' falhou: {}", skill_name, e);
                self.skill_manager.record_execution(
                    skill_name,
                    ExecutionStatus::Failed,
                    None,
                    Some(e.clone()),
                );
                Err(e)
            }
        }
    }
}
