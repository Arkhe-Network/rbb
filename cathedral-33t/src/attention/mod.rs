use crate::config::AttentionConfig;
use crate::tensor::Tensor;
use crate::utils::math::{apply_rope, compute_rope_frequencies};

pub struct HybridAttention {
    pub head_dim: usize,
    pub num_heads: usize,
    pub cos_cache: Vec<f32>,
    pub sin_cache: Vec<f32>,
}

impl HybridAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        let (cos_cache, sin_cache) = compute_rope_frequencies(config.head_dim, 10000.0, 1024);
        Self {
            head_dim: config.head_dim,
            num_heads: config.num_heads,
            cos_cache,
            sin_cache,
        }
    }

    pub fn forward(&self, x: &Tensor, pos: usize) -> Tensor {
        apply_rope(x, &self.cos_cache, &self.sin_cache, pos, self.head_dim)
    }
}
