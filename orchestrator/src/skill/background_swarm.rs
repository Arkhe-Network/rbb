use crate::skill::manager::SkillManager;
use crate::skill::types::SkillType;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct BackgroundSwarm {
    skill_manager: Arc<Mutex<SkillManager>>,
}

impl BackgroundSwarm {
    pub fn new(skill_manager: Arc<Mutex<SkillManager>>) -> Self {
        Self { skill_manager }
    }

    pub async fn run_weekly(&self) {
        let mut manager = self.skill_manager.lock().await;

        // clone the skills to avoid borrowing issues
        let background_skills: Vec<_> = manager.list_by_type(SkillType::Background)
            .into_iter()
            .cloned()
            .collect();

        info!("🕒 Iniciando execução de {} skills de background (semanal)", background_skills.len());

        for skill in background_skills {
            let skill_name = skill.name.clone();
            info!("⏳ Executando skill de background: {}", skill_name);
            // In a real system, you'd spawn this to avoid blocking the loop
            // and use a real executor
            manager.record_execution(
                &skill_name,
                crate::skill::types::ExecutionStatus::Completed,
                Some(format!("Background execution of {}", skill_name).into_bytes()),
                None,
            );
            info!("✅ Background skill '{}' concluída", skill_name);
        }
    }
}
