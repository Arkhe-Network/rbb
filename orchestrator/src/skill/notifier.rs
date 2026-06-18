use crate::dark_relay::relay::DarkRelay;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNotification {
    pub skill_name: String,
    pub status: NotificationStatus,
    pub timestamp: u64,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    Success,
    Failure,
    Started,
    Scheduled,
}

pub struct SkillNotifier {
    dark_relay: Option<DarkRelay>,
    target_npub: String,
}

impl SkillNotifier {
    pub fn new(dark_relay: Option<DarkRelay>, target_npub: String) -> Self {
        Self {
            dark_relay,
            target_npub,
        }
    }

    pub async fn notify_started(&self, skill_name: &str) {
        let msg = SkillNotification {
            skill_name: skill_name.to_string(),
            status: NotificationStatus::Started,
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: format!("⏰ Skill '{}' iniciou execução", skill_name),
            details: None,
        };
        self.send_notification(msg).await;
    }

    pub async fn notify_success(&self, skill_name: &str, details: Option<String>) {
        let msg = SkillNotification {
            skill_name: skill_name.to_string(),
            status: NotificationStatus::Success,
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: format!("✅ Skill '{}' concluída com sucesso", skill_name),
            details,
        };
        self.send_notification(msg).await;
    }

    pub async fn notify_failure(&self, skill_name: &str, error_msg: &str) {
        let msg = SkillNotification {
            skill_name: skill_name.to_string(),
            status: NotificationStatus::Failure,
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: format!("❌ Skill '{}' falhou", skill_name),
            details: Some(error_msg.to_string()),
        };
        self.send_notification(msg).await;
        error!("❌ Skill '{}' falhou: {}", skill_name, error_msg);
    }

    pub async fn notify_scheduled(&self, skill_name: &str, cron_expr: &str) {
        let msg = SkillNotification {
            skill_name: skill_name.to_string(),
            status: NotificationStatus::Scheduled,
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: format!("📅 Skill '{}' agendada com cron '{}'", skill_name, cron_expr),
            details: None,
        };
        self.send_notification(msg).await;
    }

    async fn send_notification(&self, notification: SkillNotification) {
        if let Some(relay) = &self.dark_relay {
            let json = match serde_json::to_string(&notification) {
                Ok(j) => j,
                Err(e) => {
                    warn!("Erro ao serializar notificação: {}", e);
                    return;
                }
            };

            if let Err(e) = relay.send_private(&self.target_npub, &json).await {
                warn!("Falha ao enviar notificação via Dark Relay: {}", e);
            } else {
                info!("📨 Notificação enviada via Dark Relay para {}", self.target_npub);
            }
        }
    }
}
