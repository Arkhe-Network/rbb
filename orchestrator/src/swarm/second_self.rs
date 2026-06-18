use std::sync::Arc;
use crate::hashtree::storage::provider::StorageProvider;
use crate::swarm::orchestrator::SwarmOrchestrator;
use crate::swarm::types::{SwarmResult, SwarmSpec};
use crate::skill::manager::SkillManager;
use crate::skill::types::{SkillType, Skill};
use crate::skill::executor::SkillExecutor;
use crate::skill::scheduler::SkillScheduler;
use crate::skill::persistence::SkillPersistence;
use crate::skill::registry::{SkillRegistry, DiscoveryFilters};
use crate::dark_relay::relay::DarkRelay;
use crate::swarm::background::BackgroundSwarm;
use tracing::{info, warn};

pub enum LLMProvider {
    Mock,
}

pub struct SecondSelfOrchestrator {
    pub storage: Arc<dyn StorageProvider>,
    pub orchestrator: SwarmOrchestrator,
    pub npub: String,
}

impl SecondSelfOrchestrator {
    pub async fn new(
        storage: Arc<dyn StorageProvider>,
        npub: String,
        llm: LLMProvider,
        identity_path: &str,
        concurrency: usize,
        verbose: bool,
    ) -> Result<Self, String> {
        Ok(Self {
            storage,
            npub,
            orchestrator: SwarmOrchestrator,
        })
    }

    pub async fn run_self_spec(&self, spec: SwarmSpec) -> Result<SwarmResult, String> {
        self.orchestrator.run_spec(spec).await
    }

    pub async fn load_all_skills(
        &mut self,
        skill_mgr: &mut SkillManager,
        skills_dir: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let mut loaded = Vec::new();

        let builtin = crate::skill::builtin::register_all(skill_mgr).await?;
        loaded.extend(builtin);

        if let Some(dir) = skills_dir {
            let imported = Box::pin(skill_mgr.import_from_dir(dir)).await?;
            loaded.extend(imported);
        }

        let context = skill_mgr.generate_context();
        tokio::fs::write("CONTEXT.md", context)
            .await
            .map_err(|e| format!("Erro ao escrever CONTEXT.md: {}", e))?;
        info!("📄 CONTEXT.md gerado com {} skills", loaded.len());

        Ok(loaded)
    }

    pub async fn execute_skill(
        &mut self,
        skill_mgr: &mut SkillManager,
        skill_name: &str,
    ) -> Result<crate::swarm::types::SwarmResult, String> {
        let orchestrator = self.orchestrator.clone();
        let mut executor = SkillExecutor::new(orchestrator, skill_mgr);
        executor.execute_skill(skill_name).await
    }

    pub async fn apply_model_skills(
        &mut self,
        input_text: &str,
        skill_mgr: &mut SkillManager,
    ) -> Result<Vec<String>, String> {
        let triggered = skill_mgr.find_by_trigger(input_text);
        let mut applied = Vec::new();

        for skill in triggered {
            if skill.skill_type == SkillType::ModelInvoked {
                info!("⚡ Aplicando skill model-invoked: {}", skill.name);
                applied.push(skill.name.clone());
            }
        }

        Ok(applied)
    }

    pub async fn create_scheduler(
        &self,
        skill_manager: Arc<tokio::sync::Mutex<SkillManager>>,
        dark_relay: Option<DarkRelay>,
    ) -> Result<SkillScheduler, String> {
        let persistence = SkillPersistence::new(
            self.storage.clone(),
            self.npub.clone(),
        );

        let scheduler = SkillScheduler::new(
            Arc::new(self.orchestrator.clone()),
            skill_manager,
            persistence,
            dark_relay,
        );
        Ok(scheduler)
    }

    pub async fn discover_skills_from_registry(
        &mut self,
        registry: &mut SkillRegistry,
        skill_manager: &mut SkillManager,
        prefix: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let skills = registry.list_skills(prefix).await;
        let mut imported = Vec::new();

        for name in skills {
            match registry.import_skill(&name, skill_manager).await {
                Ok(skill) => {
                    imported.push(skill.name.clone());
                    info!("🔍 Skill descoberta e importada: {}", skill.name);
                }
                Err(e) => {
                    warn!("Erro ao importar skill '{}': {}", name, e);
                }
            }
        }

        Ok(imported)
    }

    pub async fn discover_and_import_skills(
        &mut self,
        registry: &mut SkillRegistry,
        skill_manager: &mut SkillManager,
        filters: Option<DiscoveryFilters>,
    ) -> Result<Vec<String>, String> {
        info!("🔍 SecondSelf: iniciando auto-descoberta de skills...");

        let skills = registry.discover_skills(skill_manager, filters).await?;
        let names: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();

        if !names.is_empty() {
            info!("✅ {} skills descobertas e importadas: {:?}", names.len(), names);
            let context = skill_manager.generate_context();
            tokio::fs::write("CONTEXT.md", context)
                .await
                .map_err(|e| format!("Erro ao escrever CONTEXT.md: {}", e))?;
        }

        Ok(names)
    }

    pub async fn periodic_discovery(
        &mut self,
        registry: &mut SkillRegistry,
        skill_manager: &mut SkillManager,
    ) -> Result<usize, String> {
        let discovered = self.discover_and_import_skills(
            registry,
            skill_manager,
            Some(DiscoveryFilters {
                prefix: Some("cathedral".to_string()),
                tag: None,
            }),
        ).await?;

        Ok(discovered.len())
    }
}
