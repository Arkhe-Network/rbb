use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CathedralConfig {
    pub model: ModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub num_layers: usize,
    pub vocab_size: usize,
    pub max_seq_len: usize,
    pub num_experts: usize,
    pub top_k: usize,
    pub intermediate_size: usize,
    pub mhc_expansion_rate: usize,
    pub capacity_factor: f32,
    pub load_balancing_loss_coef: f32,
    pub moe: MoEConfig,
    pub attention: AttentionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoEConfig {
    pub num_experts: usize,
    pub top_k: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub capacity_factor: f32,
    pub load_balancing_loss_coef: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttentionConfig {
    pub num_heads: usize,
    pub head_dim: usize,
    pub csa_compression: usize,
    pub hca_compression: usize,
    pub sliding_window_size: usize,
    pub mla_latent_dim: usize,
}
