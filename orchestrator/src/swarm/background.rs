use crate::skill::manager::SkillManager;
use crate::skill::types::SkillType;
use crate::swarm::types::SwarmSpec;
use tracing::{info, warn};

pub enum Trigger {
    Schedule(String),
}

pub struct BackgroundSwarm {
    pub scheduled_specs: Vec<(SwarmSpec, Trigger)>,
    pub scheduled_skills: Vec<(String, String, SwarmSpec)>,
}

impl BackgroundSwarm {
    pub fn new() -> Self {
        Self {
            scheduled_specs: Vec::new(),
            scheduled_skills: Vec::new(),
        }
    }

    pub async fn schedule_skill(
        &mut self,
        skill_name: &str,
        cron_expr: &str,
        _orchestrator: &mut crate::swarm::orchestrator::SwarmOrchestrator,
        skill_manager: &mut SkillManager,
    ) -> Result<(), String> {
        let skill = skill_manager.load_skill(skill_name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?;

        if skill.skill_type != SkillType::Background {
            warn!("Skill '{}' não é do tipo Background, mas será agendada mesmo assim", skill_name);
        }

        let trigger = Trigger::Schedule(cron_expr.to_string());
        let spec = skill.to_swarm_spec();

        self.add_scheduled_spec(spec.clone(), trigger)?;

        self.scheduled_skills.push((
            skill_name.to_string(),
            cron_expr.to_string(),
            spec,
        ));

        info!("📅 Skill '{}' agendada com cron '{}'", skill_name, cron_expr);
        Ok(())
    }

    pub fn add_scheduled_spec(&mut self, spec: SwarmSpec, trigger: Trigger) -> Result<(), String> {
        self.scheduled_specs.push((spec, trigger));
        Ok(())
    }

    pub async fn run(&mut self) {
        info!("🔄 BackgroundSwarm run loop started");
        // Em um sistema real, aqui existiria um loop que verifica os cron_expr
        // associados aos scheduled_specs e scheduled_skills e chama o orchestrator
        // no tempo correto. Por simplicidade do exemplo, apenas fazemos return.
    }
}
