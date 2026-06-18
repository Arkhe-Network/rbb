use crate::skill::manager::SkillManager;
use crate::skill::types::{SkillType, ExecutionStatus};
use crate::skill::executor::SkillExecutor;
use crate::hashtree::storage::provider::StorageProvider;
use crate::hashtree::types::{StoreRequest, RetrieveRequest, VisibilityMode};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use cron::Schedule;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSchedule {
    pub skill_name: String,
    pub cron_expr: String,
    pub enabled: bool,
    pub last_run: Option<u64>,
    pub next_run: Option<u64>,
}

pub struct BackgroundSkillScheduler {
    orchestrator: Arc<crate::swarm::orchestrator::SwarmOrchestrator>,
    skill_manager: Arc<Mutex<SkillManager>>,
    storage: Arc<dyn StorageProvider>,
    npub: String,
    schedules: Vec<SkillSchedule>,
    running: bool,
}

impl BackgroundSkillScheduler {
    pub fn new(
        orchestrator: Arc<crate::swarm::orchestrator::SwarmOrchestrator>,
        skill_manager: Arc<Mutex<SkillManager>>,
        storage: Arc<dyn StorageProvider>,
        npub: String,
    ) -> Self {
        Self {
            orchestrator,
            skill_manager,
            storage,
            npub,
            schedules: Vec::new(),
            running: false,
        }
    }

    pub async fn load_schedules(&mut self) -> Result<(), String> {
        let path = "swarm/schedules/background_skills";
        if let Ok(nref) = self.storage.resolve_nostr_ref(&self.npub, path).await {
            if let Ok(resp) = self.storage.retrieve(RetrieveRequest {
                content_hash: nref.current_root,
                path: None,
            }).await {
                if let Ok(schedules) = bincode::deserialize::<Vec<SkillSchedule>>(&resp.data) {
                    self.schedules = schedules;
                    info!("📅 {} agendamentos carregados do HashTree", self.schedules.len());
                }
            }
        }
        Ok(())
    }

    pub async fn save_schedules(&self) -> Result<(), String> {
        let data = bincode::serialize(&self.schedules)
            .map_err(|e| format!("Erro ao serializar schedules: {}", e))?;

        let req = StoreRequest {
            data,
            visibility: VisibilityMode::Public,
            path: Some("swarm/schedules/background_skills".to_string()),
            metadata: None,
        };

        self.storage.store(req).await
            .map_err(|e| format!("Erro ao salvar schedules: {}", e))?;
        Ok(())
    }

    pub async fn add_schedule(&mut self, skill_name: &str, cron_expr: &str) -> Result<(), String> {
        let mut skill_mgr = self.skill_manager.lock().await;
        let skill = skill_mgr.load_skill(skill_name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?;

        if skill.skill_type != SkillType::Background {
            return Err(format!("Skill '{}' não é do tipo Background", skill_name));
        }

        Schedule::from_str(cron_expr)
            .map_err(|e| format!("Cron inválido '{}': {}", cron_expr, e))?;

        let schedule = SkillSchedule {
            skill_name: skill_name.to_string(),
            cron_expr: cron_expr.to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
        };

        self.schedules.push(schedule);
        self.save_schedules().await?;
        info!("📅 Skill '{}' agendada com '{}'", skill_name, cron_expr);
        Ok(())
    }

    pub fn list_schedules(&self) -> &[SkillSchedule] {
        &self.schedules
    }

    pub async fn start(&mut self) {
        if self.running { return; }
        self.running = true;
        info!("🔄 BackgroundSkillScheduler iniciado");

        let schedules = self.schedules.clone();
        let skill_manager = self.skill_manager.clone();
        let orchestrator = self.orchestrator.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                ticker.tick().await;
                let now = chrono::Utc::now().timestamp() as u64;

                for schedule in &schedules {
                    if !schedule.enabled { continue; }

                    let cron = match Schedule::from_str(&schedule.cron_expr) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let next = cron.after(&chrono::DateTime::from_timestamp(schedule.last_run.unwrap_or(0) as i64, 0).unwrap_or_else(|| chrono::Utc::now())).next();
                    if let Some(next_time) = next {
                        let next_ts = next_time.timestamp() as u64;
                        if next_ts <= now {
                            let mut mgr = skill_manager.lock().await;
                            let mut executor = SkillExecutor::new(
                                (*orchestrator).clone(),
                                &mut *mgr,
                            );

                            info!("⏰ Executando skill agendada: {}", schedule.skill_name);
                            let _ = executor.execute_skill_background(&schedule.skill_name).await;
                        }
                    }
                }
            }
        });
    }
}
