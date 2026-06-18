use crate::skill::types::{Skill, SkillType, SkillExecution, ExecutionStatus};
use crate::hashtree::storage::provider::StorageProvider;
use crate::hashtree::types::{StoreRequest, RetrieveRequest, VisibilityMode, ContentHash};
use crate::okf::bundle::OkfBundle;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct SkillManager {
    storage: Arc<dyn StorageProvider>,
    pub skills: HashMap<String, Skill>,
    npub: String,
    executions: Vec<SkillExecution>,
}

impl SkillManager {
    pub fn new(storage: Arc<dyn StorageProvider>, npub: String) -> Self {
        Self {
            storage,
            skills: HashMap::new(),
            npub,
            executions: Vec::new(),
        }
    }

    pub async fn load_skill(&mut self, name: &str) -> Option<&Skill> {
        if self.skills.contains_key(name) {
            return self.skills.get(name);
        }

        let path = format!("swarm/skill/{}", name);
        let nref = self.storage.resolve_nostr_ref(&self.npub, &path).await.ok()?;
        let response = self.storage.retrieve(RetrieveRequest {
            content_hash: nref.current_root,
            path: None,
        }).await.ok()?;

        let skill: Skill = bincode::deserialize(&response.data).ok()?;
        self.skills.insert(name.to_string(), skill.clone());
        Some(self.skills.get(name).unwrap())
    }

    pub async fn save_skill(&mut self, skill: &Skill) -> Result<ContentHash, String> {
        let data = bincode::serialize(skill)
            .map_err(|e| format!("Erro ao serializar skill: {}", e))?;

        let request = StoreRequest {
            data,
            visibility: VisibilityMode::Public,
            path: Some(format!("swarm/skill/{}", skill.name)),
            metadata: Some(serde_json::json!({
                "type": "skill",
                "skill_type": format!("{:?}", skill.skill_type),
                "version": skill.version,
                "tags": skill.tags,
            })),
        };

        let response = self.storage.store(request)
            .await
            .map_err(|e| format!("Erro ao salvar: {}", e))?;

        self.storage.publish_nostr_ref(
            &self.npub,
            &format!("swarm/skill/{}", skill.name),
            &response.content_hash,
        ).await
        .map_err(|e| format!("Erro ao publicar Nostr: {}", e))?;

        info!("✅ Skill '{}' salva e publicada (hash: {})", skill.name, response.content_hash.to_nhash());
        self.skills.insert(skill.name.clone(), skill.clone());
        Ok(response.content_hash)
    }

    pub async fn import_from_file(&mut self, path: &str) -> Result<&Skill, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Erro ao ler {}: {}", path, e))?;
        let skill = Skill::from_markdown(&content, path)?;
        self.save_skill(&skill).await?;
        Ok(self.skills.get(&skill.name).unwrap())
    }

    pub async fn import_from_dir(&mut self, dir: &str) -> Result<Vec<String>, String> {
        let mut imported = Vec::new();
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Erro ao ler diretório {}: {}", dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() && path.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
                if let Ok(skill) = self.import_from_file(path.to_str().unwrap()).await {
                    imported.push(skill.name.clone());
                    info!("📥 Skill importada: {}", skill.name);
                }
            }
            if path.is_dir() {
                let sub = Box::pin(self.import_from_dir(path.to_str().unwrap())).await?;
                imported.extend(sub);
            }
        }
        Ok(imported)
    }

    pub fn list_by_type(&self, skill_type: SkillType) -> Vec<&Skill> {
        self.skills.values()
            .filter(|s| s.skill_type == skill_type)
            .collect()
    }

    pub fn find_by_trigger(&self, text: &str) -> Vec<&Skill> {
        let lower = text.to_lowercase();
        self.skills.values()
            .filter(|s| s.triggers.iter().any(|t| lower.contains(&t.to_lowercase())))
            .collect()
    }

    pub fn record_execution(&mut self, skill_name: &str, status: ExecutionStatus, output: Option<Vec<u8>>, error: Option<String>) {
        self.executions.push(SkillExecution {
            skill_name: skill_name.to_string(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: Some(chrono::Utc::now().timestamp() as u64),
            status,
            output,
            error,
        });
    }

    pub fn generate_context(&self) -> String {
        let mut context = String::new();
        context.push_str("# Domain Context — Skills\n\n");
        context.push_str("## Available Skills\n\n");

        for skill in self.skills.values() {
            context.push_str(&format!("### `{}` ({:?})\n", skill.name, skill.skill_type));
            context.push_str(&format!("> {}\n\n", skill.description));
            if !skill.triggers.is_empty() {
                context.push_str(&format!("**Triggers:** {}\n", skill.triggers.join(", ")));
            }
            context.push_str(&format!("**Steps:** {}\n\n", skill.steps.len()));
        }

        context.push_str("\n## Active Model-Invoked Skills\n\n");
        for skill in self.skills.values() {
            if skill.skill_type == SkillType::ModelInvoked && !skill.triggers.is_empty() {
                context.push_str(&format!("- `{}` (triggers: {})\n", skill.name, skill.triggers.join(", ")));
            }
        }

        context
    }

    pub async fn export_as_okf(&self, skill_name: &str) -> Result<OkfBundle, String> {
        let skill = self.skills.get(skill_name)
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?;

        let mut bundle = OkfBundle::new(
            &format!("skill-{}", skill.name),
            &format!("Skill: {}", skill.description),
        );

        bundle.add_document(
            "SKILL.md".to_string(),
            crate::okf::types::OkfMetadata {
                doc_type: "skill".to_string(),
                title: skill.name.clone(),
                description: skill.description.clone(),
                tags: skill.tags.clone(),
                author: skill.author.clone().unwrap_or_else(|| "cathedral-arkhe".to_string()),
                version: Some(skill.version.clone()),
            },
            skill.instructions.clone(),
        );

        let steps_content = skill.steps.iter()
            .map(|s| format!("{}. {}", s.order, s.description))
            .collect::<Vec<_>>()
            .join("\n");
        bundle.add_document(
            "steps.md".to_string(),
            crate::okf::types::OkfMetadata {
                doc_type: "steps".to_string(),
                title: format!("{} — Steps", skill.name),
                description: "Execução passo a passo".to_string(),
                tags: vec!["steps".to_string()],
                author: "cathedral-arkhe".to_string(),
                ..Default::default()
            },
            steps_content,
        );

        bundle.add_log_entry("export", &format!("Skill '{}' exportada para OKF", skill.name));
        Ok(bundle)
    }

    pub async fn list_skills(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }
}
