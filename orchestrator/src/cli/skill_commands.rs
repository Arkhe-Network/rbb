use crate::swarm::second_self::SecondSelfOrchestrator;
use crate::skill::manager::SkillManager;
use crate::skill::scheduler::SkillScheduler;
use crate::dark_relay::relay::DarkRelay;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillCommand {
    Schedule { skill_name: String, cron_expr: String },
    Unschedule { skill_name: String },
    ListSchedules,
    Enable { skill_name: String },
    Disable { skill_name: String },
    ListSkills,
    Run { skill_name: String },
    Import { skill_name: String, source: Option<String> },
    Publish { skill_name: String },
    Help,
}

impl SkillCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/schedule" | "schedule" => {
                if parts.len() >= 3 {
                    Some(Self::Schedule {
                        skill_name: parts[1].to_string(),
                        cron_expr: parts[2..].join(" "),
                    })
                } else {
                    None
                }
            }
            "/unschedule" | "unschedule" => {
                if parts.len() >= 2 {
                    Some(Self::Unschedule {
                        skill_name: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "/list-schedules" | "list-schedules" => Some(Self::ListSchedules),
            "/enable" | "enable" => {
                if parts.len() >= 2 {
                    Some(Self::Enable {
                        skill_name: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "/disable" | "disable" => {
                if parts.len() >= 2 {
                    Some(Self::Disable {
                        skill_name: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "/list-skills" | "list-skills" => Some(Self::ListSkills),
            "/run" | "run" => {
                if parts.len() >= 2 {
                    Some(Self::Run {
                        skill_name: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "/import" | "import" => {
                if parts.len() >= 2 {
                    Some(Self::Import {
                        skill_name: parts[1].to_string(),
                        source: if parts.len() >= 3 { Some(parts[2].to_string()) } else { None },
                    })
                } else {
                    None
                }
            }
            "/publish" | "publish" => {
                if parts.len() >= 2 {
                    Some(Self::Publish {
                        skill_name: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "/help" | "help" | "?" => Some(Self::Help),
            _ => None,
        }
    }

    pub async fn execute(
        &self,
        orchestrator: &mut SecondSelfOrchestrator,
        skill_manager: &mut SkillManager,
        scheduler: &mut SkillScheduler,
        registry: &mut crate::skill::registry::SkillRegistry,
        dark_relay: &Option<DarkRelay>,
    ) -> Result<String, String> {
        match self {
            Self::Schedule { skill_name, cron_expr } => {
                scheduler.add_schedule(skill_name, cron_expr).await?;
                let msg = format!("✅ Skill '{}' agendada com cron '{}'", skill_name, cron_expr);
                if let Some(relay) = dark_relay {
                    let _ = relay.send_private("npub1system", &msg).await;
                }
                Ok(msg)
            }
            Self::Unschedule { skill_name } => {
                scheduler.remove_schedule(skill_name).await?;
                Ok(format!("🗑️ Agendamento de '{}' removido", skill_name))
            }
            Self::ListSchedules => {
                let schedules = scheduler.list_schedules();
                if schedules.is_empty() {
                    Ok("📅 Nenhum agendamento ativo".to_string())
                } else {
                    let mut output = String::from("📅 Agendamentos:\n");
                    for s in schedules {
                        output.push_str(&format!(
                            "  - {}: {} (enabled: {}) | last_run: {:?}\n",
                            s.skill_name, s.cron_expr, s.enabled, s.last_run
                        ));
                    }
                    Ok(output)
                }
            }
            Self::Enable { skill_name } => {
                scheduler.toggle_schedule(skill_name, true).await?;
                Ok(format!("✅ Agendamento '{}' habilitado", skill_name))
            }
            Self::Disable { skill_name } => {
                scheduler.toggle_schedule(skill_name, false).await?;
                Ok(format!("⏸️ Agendamento '{}' desabilitado", skill_name))
            }
            Self::ListSkills => {
                let skills = skill_manager.list_skills().await;
                if skills.is_empty() {
                    Ok("📚 Nenhuma skill carregada".to_string())
                } else {
                    let mut output = String::from("📚 Skills disponíveis:\n");
                    for s in skills {
                        output.push_str(&format!("  - {}\n", s));
                    }
                    Ok(output)
                }
            }
            Self::Run { skill_name } => {
                let result = orchestrator.execute_skill(skill_manager, skill_name).await?;
                Ok(format!(
                    "✅ Skill '{}' executada | {} agentes, {} steps, {}s",
                    skill_name, result.agent_count, result.total_steps, result.duration_secs
                ))
            }
            Self::Import { skill_name, source: _ } => {
                let imported = registry.import_skill(skill_name, skill_manager).await?;
                Ok(format!("✅ Skill '{}' importada do registro", imported.name))
            }
            Self::Publish { skill_name } => {
                let mut skill_mgr_clone = skill_manager.clone();
                let skill = skill_mgr_clone.load_skill(skill_name).await
                    .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?;
                let hash = registry.publish_skill(skill).await?;
                Ok(format!("📡 Skill '{}' publicada (hash: {})", skill_name, hash.to_nhash()))
            }
            Self::Help => Ok(
                r#"📖 Comandos disponíveis:
  /schedule <skill> <cron>   - Agenda uma skill
  /unschedule <skill>        - Remove agendamento
  /list-schedules            - Lista agendamentos
  /enable <skill>            - Habilita agendamento
  /disable <skill>           - Desabilita agendamento
  /list-skills               - Lista skills disponíveis
  /run <skill>               - Executa skill imediatamente
  /import <skill>            - Importa skill do registro
  /publish <skill>           - Publica skill no registro
  /help                      - Mostra esta ajuda"#.to_string()
            ),
        }
    }
}
