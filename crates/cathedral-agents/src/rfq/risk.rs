use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct RiskManager {
    pub position_limits: RwLock<HashMap<String, u64>>,
    pub exposure_limits: RwLock<HashMap<String, f64>>,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            position_limits: RwLock::new(HashMap::new()),
            exposure_limits: RwLock::new(HashMap::new()),
        }
    }

    pub async fn check_position(&self, asset: &str, amount: u64) -> bool {
        let limits = self.position_limits.read().await;
        if let Some(limit) = limits.get(asset) {
            return amount <= *limit;
        }
        true
    }
}
