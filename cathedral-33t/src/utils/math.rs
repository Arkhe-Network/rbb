use crate::tensor::Tensor;

pub fn clip_gradients(grads: &mut [Tensor], max_norm: f32) {
    let mut total_norm = 0.0f32;
    for grad in grads.iter() {
        total_norm += grad.mapv(|v| v * v).sum();
    }
    total_norm = total_norm.sqrt();
    if total_norm > max_norm {
        let scale = max_norm / total_norm;
        for grad in grads.iter_mut() {
            *grad = grad.scale(scale);
        }
    }
}

pub fn compute_rope_frequencies(
    dim: usize,
    theta: f32,
    max_seq_len: usize,
) -> (Vec<f32>, Vec<f32>) {
    let inv_freq: Vec<f32> = (0..dim / 2)
        .map(|i| 1.0 / theta.powf(2.0 * i as f32 / dim as f32))
        .collect();

    let mut cos_cache = Vec::with_capacity(max_seq_len * dim / 2);
    let mut sin_cache = Vec::with_capacity(max_seq_len * dim / 2);

    for pos in 0..max_seq_len {
        for freq in &inv_freq {
            let angle = pos as f32 * freq;
            cos_cache.push(angle.cos());
            sin_cache.push(angle.sin());
        }
    }

    (cos_cache, sin_cache)
}

pub fn apply_rope(
    x: &Tensor,
    cos_cache: &[f32],
    sin_cache: &[f32],
    pos: usize,
    head_dim: usize,
) -> Tensor {
    let mut result = x.clone();
    let half_dim = head_dim / 2;

    for token_idx in 0..x.nrows() {
        for i in 0..half_dim {
            let cos = cos_cache[(pos + token_idx) * half_dim + i];
            let sin = sin_cache[(pos + token_idx) * half_dim + i];

            let x0 = result.get(token_idx, i);
            let x1 = result.get(token_idx, i + half_dim);

            result.set(token_idx, i, x0 * cos - x1 * sin);
            result.set(token_idx, i + half_dim, x0 * sin + x1 * cos);
        }
    }

    result
}
