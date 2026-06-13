pub struct SwiReasoningConfig {
    pub entropy_ref_x1000: u32,
    pub max_switches: u32,
    pub block_size: usize,
    pub latent_depth: usize,
    pub explicit_commit_threshold: u32,
    pub tee_enforced: bool,
    pub pqc_anchor: bool,
}

pub enum ReasoningMode {
    Explicit,
    Latent,
}
