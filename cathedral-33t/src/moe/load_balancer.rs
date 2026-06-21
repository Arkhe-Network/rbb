use crate::moe::router::RoutingIndex;

pub struct LoadBalancer {
    pub capacity_factor: f32,
    pub expert_loads: Vec<usize>,
}

impl LoadBalancer {
    pub fn new(capacity_factor: f32, num_experts: usize) -> Self {
        Self {
            capacity_factor,
            expert_loads: vec![0; num_experts],
        }
    }

    pub fn apply(&mut self, routing: &[Vec<RoutingIndex>]) -> Vec<(usize, usize, f32)> {
        let mut result = Vec::new();
        let capacity = (self.capacity_factor
            * (routing.len() as f32 / self.expert_loads.len() as f32))
            as usize;

        for load in self.expert_loads.iter_mut() {
            *load = 0;
        }

        for (token_idx, indices) in routing.iter().enumerate() {
            for routing_idx in indices {
                let expert_id = routing_idx.expert_id;
                if self.expert_loads[expert_id] < capacity {
                    self.expert_loads[expert_id] += 1;
                    result.push((token_idx, expert_id, routing_idx.weight));
                    break;
                }
            }
        }

        result
    }
}
