use crate::skill::types::{Skill, SkillType};
use crate::hashtree::storage::provider::StorageProvider;
use crate::hashtree::types::{ContentHash, RetrieveRequest, StoreRequest, VisibilityMode};
use crate::hashtree::nostr::resolver::NostrReferenceResolver;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub struct SkillRegistry {
    storage: Arc<dyn StorageProvider>,
    npub: String,
    cache: HashMap<String, Skill>,
    resolver: NostrReferenceResolver,
    _relays: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryFilters {
    pub prefix: Option<String>,
    pub tag: Option<String>,
}

impl SkillRegistry {
    pub fn new(
        storage: Arc<dyn StorageProvider>,
        npub: String,
        relays: Vec<String>,
    ) -> Self {
        Self {
            storage,
            npub,
            cache: HashMap::new(),
            resolver: NostrReferenceResolver::new(relays.clone()),
            _relays: relays,
        }
    }

    pub async fn publish_skill(&mut self, skill: &Skill) -> Result<ContentHash, String> {
        let data = bincode::serialize(skill)
            .map_err(|e| format!("Erro ao serializar skill: {}", e))?;

        let store_req = StoreRequest {
            data,
            visibility: VisibilityMode::Public,
            path: Some(format!("swarm/skill/{}", skill.name)),
            metadata: Some(serde_json::json!({
                "type": "skill",
                "skill_type": format!("{:?}", skill.skill_type),
                "version": skill.version,
                "tags": skill.tags,
                "author": skill.author,
                "timestamp": chrono::Utc::now().timestamp(),
            })),
        };

        let response = self.storage.store(store_req)
            .await
            .map_err(|e| format!("Erro ao salvar no HashTree: {}", e))?;

        let content_hash = response.content_hash;

        let event = self.storage.publish_nostr_ref(
            &self.npub,
            &format!("swarm/skill/{}", skill.name),
            &content_hash,
        ).await
        .map_err(|e| format!("Erro ao publicar Nostr: {}", e))?;

        self.cache.insert(skill.name.clone(), skill.clone());

        info!("📡 Skill '{}' publicada no registro (hash: {}, event: {})",
            skill.name, content_hash.to_nhash(), event.id);
        Ok(content_hash)
    }

    pub async fn fetch_skill(&mut self, name: &str) -> Option<Skill> {
        if let Some(cached) = self.cache.get(name) {
            return Some(cached.clone());
        }

        let path = format!("swarm/skill/{}", name);

        match self.resolver.resolve(&self.npub, &path).await {
            Ok(nref) => {
                let response = self.storage.retrieve(RetrieveRequest {
                    content_hash: nref.current_root,
                    path: None,
                }).await.ok()?;

                let skill: Skill = bincode::deserialize(&response.data).ok()?;
                self.cache.insert(name.to_string(), skill.clone());
                Some(skill)
            }
            Err(e) => {
                warn!("Erro ao buscar skill '{}' via Nostr: {}", name, e);
                None
            }
        }
    }

    pub async fn list_skills(&self, prefix: Option<&str>) -> Vec<String> {
        let mut skills = vec![
            "grill-me".to_string(),
            "to-prd".to_string(),
            "diagnose".to_string(),
            "tdd".to_string(),
            "improve-architecture".to_string(),
            "triage".to_string(),
        ];

        if let Some(pre) = prefix {
            skills.retain(|s| s.starts_with(pre));
        }

        skills
    }

    pub async fn import_skill(
        &mut self,
        name: &str,
        manager: &mut crate::skill::manager::SkillManager,
    ) -> Result<Skill, String> {
        let skill = self.fetch_skill(name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada no registro", name))?;

        manager.save_skill(&skill).await?;
        info!("✅ Skill '{}' importada do registro", name);
        Ok(skill)
    }

    pub async fn discover_skills(
        &mut self,
        manager: &mut crate::skill::manager::SkillManager,
        filters: Option<DiscoveryFilters>,
    ) -> Result<Vec<Skill>, String> {
        info!("🔍 Iniciando auto-descoberta de skills...");

        let mut discovered = Vec::new();
        let all_skills = self.list_skills(None).await;

        let filter = filters.unwrap_or_default();

        for name in all_skills {
            if let Some(prefix) = &filter.prefix {
                if !name.starts_with(prefix) {
                    continue;
                }
            }

            if let Some(tag) = &filter.tag {
                if !self.cache.get(&name).map(|s| s.tags.contains(tag)).unwrap_or(false) {
                    continue;
                }
            }

            match self.import_skill(&name, manager).await {
                Ok(skill) => {
                    discovered.push(skill);
                    info!("✅ Skill descoberta e importada: {}", name);
                }
                Err(e) => {
                    warn!("⚠️ Falha ao importar '{}': {}", name, e);
                }
            }
        }

        info!("🔍 Auto-descoberta concluída: {} skills importadas", discovered.len());
        Ok(discovered)
    }
}
