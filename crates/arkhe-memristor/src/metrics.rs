//! Métricas de consumo, endurance e retenção.

/// Métricas de potência.
#[derive(Debug, Clone, Copy)]
pub struct PowerMetrics {
    pub set_power_mw: f64,
    pub reset_power_mw: f64,
    pub average_power_mw: f64,
    pub peak_power_mw: f64,
}

/// Métricas de endurance.
#[derive(Debug, Clone, Copy)]
pub struct EnduranceMetrics {
    pub total_cycles: u32,
    pub remaining_cycles: u32,
    pub estimated_lifetime_cycles: u32,
    pub retention_secs: f64,
}

/// Trait para fornecer métricas.
pub trait Metrics {
    fn power_metrics(&self) -> PowerMetrics;
    fn endurance_metrics(&self) -> EnduranceMetrics;
}
