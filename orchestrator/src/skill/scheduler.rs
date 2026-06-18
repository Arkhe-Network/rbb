use crate::skill::types::SkillType;
use crate::skill::manager::SkillManager;
use crate::skill::executor::SkillExecutor;
use crate::skill::persistence::{SkillPersistence, PersistentSchedule};
use crate::swarm::orchestrator::SwarmOrchestrator;
use crate::dark_relay::relay::DarkRelay;
use crate::skill::types::ExecutionStatus;
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

pub struct SkillScheduler {
    orchestrator: Arc<SwarmOrchestrator>,
    skill_manager: Arc<tokio::sync::Mutex<SkillManager>>,
    persistence: SkillPersistence,
    dark_relay: Option<DarkRelay>,
    schedules: Vec<PersistentSchedule>,
    running: bool,
}

impl SkillScheduler {
    pub fn new(
        orchestrator: Arc<SwarmOrchestrator>,
        skill_manager: Arc<tokio::sync::Mutex<SkillManager>>,
        persistence: SkillPersistence,
        dark_relay: Option<DarkRelay>,
    ) -> Self {
        Self {
            orchestrator,
            skill_manager,
            persistence,
            dark_relay,
            schedules: Vec::new(),
            running: false,
        }
    }

    pub async fn load_and_start(&mut self) -> Result<(), String> {
        self.schedules = self.persistence.load_schedules().await?;
        info!("📅 {} schedules carregados do HashTree", self.schedules.len());
        self.start().await;
        Ok(())
    }

    pub async fn add_schedule(&mut self, skill_name: &str, cron_expr: &str) -> Result<(), String> {
        let mut mgr = self.skill_manager.lock().await;
        let skill = mgr.load_skill(skill_name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?;

        if skill.skill_type != SkillType::Background {
            return Err(format!("Skill '{}' não é do tipo Background", skill_name));
        }

        Schedule::from_str(cron_expr)
            .map_err(|e| format!("Cron inválido: {}", e))?;

        if self.schedules.iter().any(|s| s.skill_name == skill_name) {
            return Err(format!("Schedule para '{}' já existe", skill_name));
        }

        let schedule = PersistentSchedule {
            skill_name: skill_name.to_string(),
            cron_expr: cron_expr.to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            created_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
        };

        self.schedules.push(schedule);
        self.persistence.save_schedules(&self.schedules).await?;
        info!("📅 Schedule adicionado: {} -> {}", skill_name, cron_expr);

        if let Some(relay) = &self.dark_relay {
            let _ = relay.send_private("npub1system", &format!("Schedule added: {}", skill_name)).await;
        }

        Ok(())
    }

    pub async fn remove_schedule(&mut self, skill_name: &str) -> Result<(), String> {
        let before = self.schedules.len();
        self.schedules.retain(|s| s.skill_name != skill_name);
        if self.schedules.len() == before {
            return Err(format!("Schedule para '{}' não encontrado", skill_name));
        }
        self.persistence.save_schedules(&self.schedules).await?;
        info!("🗑️ Schedule removido: {}", skill_name);
        Ok(())
    }

    pub fn list_schedules(&self) -> Vec<&PersistentSchedule> {
        self.schedules.iter().collect()
    }

    pub async fn toggle_schedule(&mut self, skill_name: &str, enabled: bool) -> Result<(), String> {
        if let Some(schedule) = self.schedules.iter_mut().find(|s| s.skill_name == skill_name) {
            schedule.enabled = enabled;
            schedule.updated_at = chrono::Utc::now().timestamp() as u64;
            self.persistence.save_schedules(&self.schedules).await?;
            info!("🔄 Schedule '{}' {}", skill_name, if enabled { "habilitado" } else { "desabilitado" });
            Ok(())
        } else {
            Err(format!("Schedule '{}' não encontrado", skill_name))
        }
    }

    pub async fn start(&mut self) {
        if self.running {
            warn!("Scheduler já está rodando");
            return;
        }
        self.running = true;
        info!("🔄 SkillScheduler iniciado");

        let schedules = self.schedules.clone();
        let skill_manager = self.skill_manager.clone();
        let orchestrator = self.orchestrator.clone();
        let persistence = self.persistence.clone();
        let dark_relay = self.dark_relay.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                let now = chrono::Utc::now().timestamp() as u64;

                let mut updated_schedules = Vec::new();
                let mut schedules_to_save = false;

                for schedule in &schedules {
                    if !schedule.enabled {
                        continue;
                    }

                    let cron = match Schedule::from_str(&schedule.cron_expr) {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("Erro ao parsear cron '{}': {}", schedule.cron_expr, e);
                            continue;
                        }
                    };

                    let next = cron.after(&chrono::DateTime::from_timestamp(schedule.last_run.unwrap_or(0) as i64, 0).unwrap_or_else(|| chrono::Utc::now())).next();
                    if let Some(next_time) = next {
                        let next_ts = next_time.timestamp() as u64;

                        if next_ts <= now {
                            if let Some(last) = schedule.last_run {
                                if last >= next_ts {
                                    continue;
                                }
                            }

                            info!("⏰ Executando skill agendada: {}", schedule.skill_name);

                            let mut mgr = skill_manager.lock().await;
                            let mut executor = SkillExecutor::new(
                                (*orchestrator).clone(),
                                &mut *mgr,
                            );

                            let result = executor.execute_skill_background(&schedule.skill_name).await;

                            let status = if result.is_ok() {
                                ExecutionStatus::Completed
                            } else {
                                ExecutionStatus::Failed
                            };

                            if let Err(e) = persistence.add_execution_record(
                                &schedule.skill_name,
                                status,
                                None,
                                result.clone().err(),
                            ).await {
                                warn!("Erro ao salvar histórico: {}", e);
                            }

                            if result.is_err() {
                                if let Some(relay) = &dark_relay {
                                    let msg = format!(
                                        "❌ Skill '{}' falhou na execução agendada (cron: {})",
                                        schedule.skill_name, schedule.cron_expr
                                    );
                                    let _ = relay.send_private("npub1system", &msg).await;
                                }
                            }

                            let mut updated = schedule.clone();
                            updated.last_run = Some(now);
                            updated.updated_at = chrono::Utc::now().timestamp() as u64;
                            updated_schedules.push(updated);
                            schedules_to_save = true;
                        }
                    }
                }

                if schedules_to_save {
                    if let Ok(_loaded) = persistence.load_schedules().await {
                        info!("📅 Schedules atualizados no HashTree");
                    }
                }
            }
        });
    }

    pub fn stop(&mut self) {
        self.running = false;
        info!("🛑 SkillScheduler parado");
    }
}
