pub mod expert;
pub mod load_balancer;
pub mod router;

pub use expert::Expert;
pub use load_balancer::LoadBalancer;
pub use router::{HierarchicalRouter, RoutingIndex};

use crate::config::MoEConfig;
use crate::tensor::Tensor;

pub struct MoELayer {
    pub experts: Vec<Expert>,
    pub router: HierarchicalRouter,
    pub load_balancer: LoadBalancer,
    pub num_experts: usize,
    pub top_k: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub load_balancing_coef: f32,
}

impl MoELayer {
    pub fn new(config: &MoEConfig) -> Self {
        let experts = (0..config.num_experts)
            .map(|_| Expert::new(config.hidden_size, config.intermediate_size))
            .collect();
        let router = HierarchicalRouter::new(config.num_experts, config.top_k, config.hidden_size);
        Self {
            experts,
            router,
            load_balancer: LoadBalancer::new(config.capacity_factor, config.num_experts),
            num_experts: config.num_experts,
            top_k: config.top_k,
            hidden_size: config.hidden_size,
            intermediate_size: config.intermediate_size,
            load_balancing_coef: config.load_balancing_loss_coef,
        }
    }

    pub fn forward(&mut self, x: &Tensor) -> Tensor {
        let batch_size = x.nrows();
        let routing_indices = self.router.route(x);
        let balanced_indices = self.load_balancer.apply(&routing_indices);
        let mut expert_outputs: Vec<(usize, usize, f32, Tensor)> = Vec::new();

        for (token_idx, expert_id, weight) in &balanced_indices {
            let token = x.slice_row(*token_idx);
            let output = self.experts[*expert_id].forward(&token);
            expert_outputs.push((*token_idx, *expert_id, *weight, output));
        }

        self.combine(expert_outputs, batch_size)
    }

    fn combine(&self, outputs: Vec<(usize, usize, f32, Tensor)>, batch_size: usize) -> Tensor {
        let mut result = Tensor::zeros((batch_size, self.hidden_size));
        for (token_idx, _expert_id, weight, output) in outputs {
            let weighted_out = output.scale(weight);
            let row = result.slice_row(token_idx) + &weighted_out;
            for j in 0..self.hidden_size {
                result.set(token_idx, j, row.get(0, j));
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moe_forward() {
        let config = MoEConfig {
            num_experts: 4,
            top_k: 2,
            hidden_size: 8,
            intermediate_size: 16,
            capacity_factor: 1.0,
            load_balancing_loss_coef: 0.1,
        };
        let mut moe = MoELayer::new(&config);
        let x = Tensor::randn((2, 8));
        let out = moe.forward(&x);

        assert_eq!(out.nrows(), 2);
        assert_eq!(out.ncols(), 8);
    }
}
