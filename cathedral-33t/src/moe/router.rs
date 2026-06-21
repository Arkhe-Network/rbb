use crate::tensor::Tensor;

#[derive(Clone, Debug, PartialEq)]
pub struct RoutingIndex {
    pub expert_id: usize,
    pub weight: f32,
}

pub struct HierarchicalRouter {
    pub num_groups: usize,
    pub experts_per_group: usize,
    pub top_k: usize,
    pub hidden_size: usize,
    pub group_weights: Tensor,
    pub expert_weights: Tensor,
}

impl HierarchicalRouter {
    pub fn new(num_experts: usize, top_k: usize, hidden_size: usize) -> Self {
        let num_groups = 64;
        let experts_per_group = num_experts / num_groups;
        Self {
            num_groups,
            experts_per_group,
            top_k,
            hidden_size,
            group_weights: Tensor::randn((num_groups, hidden_size)),
            expert_weights: Tensor::randn((num_experts, hidden_size)),
        }
    }

    pub fn route(&self, x: &Tensor) -> Vec<Vec<RoutingIndex>> {
        let batch_size = x.nrows();
        let mut routing = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let token = x.slice_row(i);
            routing.push(self.route_single(&token));
        }

        routing
    }

    fn route_single(&self, token: &Tensor) -> Vec<RoutingIndex> {
        let group_logits = token.matmul(&self.group_weights.transpose());
        let group_logits_vec = group_logits.slice_row(0).to_vec();

        let top_groups = self.top_k_indices(&group_logits_vec, 2);
        let mut expert_indices = Vec::with_capacity(self.top_k);

        for &(group_idx, _) in &top_groups {
            let start = group_idx * self.experts_per_group;
            let end = start + self.experts_per_group;
            let mut group_expert_weights_data =
                ndarray::Array2::zeros((self.experts_per_group, self.hidden_size));
            for i in 0..self.experts_per_group {
                for j in 0..self.hidden_size {
                    group_expert_weights_data[[i, j]] = self.expert_weights.get(start + i, j);
                }
            }
            let group_expert_weights = Tensor::from(group_expert_weights_data);

            let expert_logits = token.matmul(&group_expert_weights.transpose());
            let expert_logits_vec = expert_logits.slice_row(0).to_vec();
            let top_experts = self.top_k_indices(&expert_logits_vec, self.top_k / 2);

            for (idx, weight) in top_experts {
                expert_indices.push(RoutingIndex {
                    expert_id: group_idx * self.experts_per_group + idx,
                    weight,
                });
            }
        }

        if expert_indices.len() > self.top_k {
            expert_indices.truncate(self.top_k);
        }
        while expert_indices.len() < self.top_k {
            expert_indices.push(RoutingIndex {
                expert_id: 0,
                weight: 0.01,
            });
        }

        expert_indices
    }

    fn top_k_indices(&self, values: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut indexed: Vec<_> = values.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        indexed.into_iter().take(k).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_route() {
        let router = HierarchicalRouter::new(4096, 8, 8192);
        let token = Tensor::randn((2, 8192));
        let routing = router.route(&token);

        assert_eq!(routing.len(), 2);
        assert_eq!(routing[0].len(), 8);
        assert_eq!(routing[1].len(), 8);
    }
}
