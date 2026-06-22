// src/federated_learning.rs
use candle_core::{Device, Tensor};
use candle_nn::{AdamW, Optimizer, VarBuilder, Linear};

pub struct FederatedLearner {
    model: Linear,
    optimizer: AdamW,
    gradients: Vec<Tensor>,
}

impl FederatedLearner {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        let device = Device::Cpu;
        let vs = VarBuilder::new(device);
        let model = Linear::new(input_dim, output_dim, vs.clone()).unwrap();
        let optimizer = AdamW::new(vs.all_vars(), 0.001).unwrap();
        Self { model, optimizer, gradients: Vec::new() }
    }

    pub fn train_local(&mut self, data: Tensor, target: Tensor) -> f32 {
        let prediction = self.model.forward(&data).unwrap();
        let loss = prediction.mse_loss(&target).unwrap();
        self.optimizer.backward_step(&loss).unwrap();
        loss.to_scalar::<f32>().unwrap()
    }

    pub fn get_gradients(&self) -> Vec<Tensor> {
        self.gradients.clone()
    }

    pub fn aggregate_gradients(&mut self, received: Vec<Tensor>) {
        // Aggregazione media
        for (i, grad) in received.iter().enumerate() {
            if let Some(existing) = self.gradients.get_mut(i) {
                *existing = (existing.clone() + grad) / 2.0;
            } else {
                self.gradients.push(grad.clone());
            }
        }
    }
}
