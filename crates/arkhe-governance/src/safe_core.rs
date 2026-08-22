use chrono::{Datelike, TimeZone, Timelike};

/// Erro de hook do Safe Core.
#[derive(Debug, thiserror::Error, Clone)]
pub enum HookError {
    #[error("Hook bloqueou a ação: {0}")]
    Blocked(String),

    #[error("Erro interno do hook: {0}")]
    Internal(String),
}

pub trait SafeCoreHook: Send + Sync {
    fn pre_submit(&self, action: &crate::GovernanceAction) -> Result<(), HookError>;
    fn pre_execute(&self, action: &crate::GovernanceAction) -> Result<(), HookError>;
    fn post_execute(&self, action: &crate::GovernanceAction, success: bool);
}

pub struct BusinessHoursHook {
    pub start_hour: u32,
    pub end_hour: u32,
    pub allowed_days: Vec<u32>,
}

impl BusinessHoursHook {
    pub fn weekday_9_to_18() -> Self {
        Self {
            start_hour: 9,
            end_hour: 18,
            allowed_days: vec![0, 1, 2, 3, 4], // Mon-Fri
        }
    }

    /// Verifica se um timestamp específico é permitido.
    /// ✅ P10: Função pura para testabilidade.
    pub fn is_allowed_at(&self, now: chrono::DateTime<chrono::Local>) -> bool {
        let hour = now.hour() as u32;
        let weekday = now.weekday().num_days_from_monday();

        if hour < self.start_hour || hour >= self.end_hour {
            return false;
        }
        if !self.allowed_days.is_empty() && !self.allowed_days.contains(&weekday) {
            return false;
        }
        true
    }
}

impl SafeCoreHook for BusinessHoursHook {
    fn pre_submit(&self, action: &crate::GovernanceAction) -> Result<(), HookError> {
        if let crate::ActionClass::Operational = action.class {
            return Ok(());
        }

        let now = chrono::Local::now();
        if !self.is_allowed_at(now) {
            return Err(HookError::Blocked(format!(
                "Fora do horário de expediente ({}h, precisa {}h-{}h)",
                now.hour(),
                self.start_hour,
                self.end_hour
            )));
        }
        Ok(())
    }

    fn pre_execute(&self, _action: &crate::GovernanceAction) -> Result<(), HookError> {
        Ok(())
    }
    fn post_execute(&self, _action: &crate::GovernanceAction, _success: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_hours_allows_weekday_10h() {
        let hook = BusinessHoursHook::weekday_9_to_18();
        let tuesday_10h = chrono::Local
            .with_ymd_and_hms(2023, 10, 10, 10, 0, 0)
            .unwrap();
        assert!(hook.is_allowed_at(tuesday_10h));
    }

    #[test]
    fn test_business_hours_blocks_sunday_20h() {
        let hook = BusinessHoursHook::weekday_9_to_18();
        let sunday_20h = chrono::Local
            .with_ymd_and_hms(2023, 10, 8, 20, 0, 0)
            .unwrap();
        assert!(!hook.is_allowed_at(sunday_20h));
    }

    #[test]
    fn test_business_hours_blocks_tuesday_5h() {
        let hook = BusinessHoursHook::weekday_9_to_18();
        let tuesday_5h = chrono::Local
            .with_ymd_and_hms(2023, 10, 10, 5, 0, 0)
            .unwrap();
        assert!(!hook.is_allowed_at(tuesday_5h));
    }

    #[test]
    fn test_business_hours_operational_bypasses() {
        let hook = BusinessHoursHook::weekday_9_to_18();

        let action = crate::GovernanceAction::new(
            crate::ActionClass::Operational,
            "rotate keys".into(),
            "did:arkhe:test".into(),
            std::time::Duration::ZERO,
            [0u8; 32],
        );

        // Operational bypassa a verificação de horário
        assert!(hook.pre_submit(&action).is_ok());
    }
}
