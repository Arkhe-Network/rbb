use crate::hashtree::storage::provider::StorageProvider;
use crate::hashtree::types::{StoreRequest, RetrieveRequest, VisibilityMode, ContentHash};
use crate::skill::types::{SkillExecution, ExecutionStatus};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentSchedule {
    pub skill_name: String,
    pub cron_expr: String,
    pub enabled: bool,
    pub last_run: Option<u64>,
    pub next_run: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStore {
    pub schedules: Vec<PersistentSchedule>,
    pub version: u32,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistory {
    pub executions: Vec<SkillExecution>,
    pub version: u32,
    pub updated_at: u64,
}

#[derive(Clone)]
pub struct SkillPersistence {
    storage: Arc<dyn StorageProvider>,
    npub: String,
    schedules_path: String,
    history_path: String,
}

impl SkillPersistence {
    pub fn new(storage: Arc<dyn StorageProvider>, npub: String) -> Self {
        Self {
            storage,
            npub,
            schedules_path: "swarm/schedules".to_string(),
            history_path: "swarm/skill_history".to_string(),
        }
    }

    pub async fn save_schedules(&self, schedules: &[PersistentSchedule]) -> Result<ContentHash, String> {
        let store = ScheduleStore {
            schedules: schedules.to_vec(),
            version: 1,
            updated_at: chrono::Utc::now().timestamp() as u64,
        };

        let data = bincode::serialize(&store)
            .map_err(|e| format!("Erro ao serializar schedules: {}", e))?;

        let request = StoreRequest {
            data,
            visibility: VisibilityMode::Private,
            path: Some(self.schedules_path.clone()),
            metadata: Some(serde_json::json!({
                "type": "schedule_store",
                "count": schedules.len(),
                "timestamp": store.updated_at,
            })),
        };

        let response = self.storage.store(request)
            .await
            .map_err(|e| format!("Erro ao salvar schedules: {}", e))?;

        let _ = self.storage.publish_nostr_ref(
            &self.npub,
            &format!("{}/latest", self.schedules_path),
            &response.content_hash,
        ).await;

        info!("✅ Schedules salvos no HashTree ({} items)", schedules.len());
        Ok(response.content_hash)
    }

    pub async fn load_schedules(&self) -> Result<Vec<PersistentSchedule>, String> {
        let nref = match self.storage.resolve_nostr_ref(&self.npub, &format!("{}/latest", self.schedules_path)).await {
            Ok(n) => n,
            Err(_) => {
                return self.load_schedules_direct().await;
            }
        };

        let response = self.storage.retrieve(RetrieveRequest {
            content_hash: nref.current_root,
            path: None,
        }).await.map_err(|e| format!("Erro ao recuperar schedules: {}", e))?;

        let store: ScheduleStore = bincode::deserialize(&response.data)
            .map_err(|e| format!("Erro ao desserializar schedules: {}", e))?;

        info!("📅 Schedules carregados ({} items)", store.schedules.len());
        Ok(store.schedules)
    }

    async fn load_schedules_direct(&self) -> Result<Vec<PersistentSchedule>, String> {
        let request = RetrieveRequest {
            content_hash: ContentHash {
                hash: [0u8; 32],
                hash_type: crate::hashtree::types::HashType::Sha256,
            },
            path: Some(self.schedules_path.clone()),
        };
        match self.storage.retrieve(request).await {
            Ok(response) => {
                let store: ScheduleStore = bincode::deserialize(&response.data)
                    .map_err(|e| format!("Erro ao desserializar: {}", e))?;
                Ok(store.schedules)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn save_execution_history(&self, executions: &[SkillExecution]) -> Result<ContentHash, String> {
        let history = ExecutionHistory {
            executions: executions.to_vec(),
            version: 1,
            updated_at: chrono::Utc::now().timestamp() as u64,
        };

        let data = bincode::serialize(&history)
            .map_err(|e| format!("Erro ao serializar histórico: {}", e))?;

        let request = StoreRequest {
            data,
            visibility: VisibilityMode::Private,
            path: Some(self.history_path.clone()),
            metadata: Some(serde_json::json!({
                "type": "execution_history",
                "count": executions.len(),
                "timestamp": history.updated_at,
            })),
        };

        let response = self.storage.store(request)
            .await
            .map_err(|e| format!("Erro ao salvar histórico: {}", e))?;

        let _ = self.storage.publish_nostr_ref(
            &self.npub,
            &format!("{}/latest", self.history_path),
            &response.content_hash,
        ).await;

        info!("📜 Histórico de execuções salvo ({} items)", executions.len());
        Ok(response.content_hash)
    }

    pub async fn load_execution_history(&self) -> Result<Vec<SkillExecution>, String> {
        let nref = match self.storage.resolve_nostr_ref(&self.npub, &format!("{}/latest", self.history_path)).await {
            Ok(n) => n,
            Err(_) => return Ok(Vec::new()),
        };

        let response = self.storage.retrieve(RetrieveRequest {
            content_hash: nref.current_root,
            path: None,
        }).await.map_err(|e| format!("Erro ao recuperar histórico: {}", e))?;

        let history: ExecutionHistory = bincode::deserialize(&response.data)
            .map_err(|e| format!("Erro ao desserializar histórico: {}", e))?;

        Ok(history.executions)
    }

    pub async fn add_execution_record(
        &self,
        skill_name: &str,
        status: ExecutionStatus,
        output: Option<Vec<u8>>,
        error: Option<String>,
    ) -> Result<ContentHash, String> {
        let mut history = self.load_execution_history().await?;
        history.push(SkillExecution {
            skill_name: skill_name.to_string(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: Some(chrono::Utc::now().timestamp() as u64),
            status,
            output,
            error,
        });

        if history.len() > 100 {
            history.drain(..history.len() - 100);
        }

        self.save_execution_history(&history).await
    }
}
