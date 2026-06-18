use crate::skill::types::{Skill, SkillType, SkillExecution, ExecutionStatus};
use std::collections::HashMap;

pub struct SkillManager {
    skills: HashMap<String, Skill>,
    executions: Vec<SkillExecution>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            executions: Vec::new(),
        }
    }

    pub fn load_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn save_skill(&mut self, skill: &Skill) -> Result<(), String> {
        self.skills.insert(skill.name.clone(), skill.clone());
        Ok(())
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
}
