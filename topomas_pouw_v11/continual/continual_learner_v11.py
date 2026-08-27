import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import TensorDataset, DataLoader
from collections import defaultdict
from typing import Dict, Any, List, Tuple, Callable
import random

class ElasticWeightConsolidation:
    def __init__(self, model: nn.Module, importance: float = 1e3):
        self.model = model
        self.importance = importance
        self.fisher: Dict[str, torch.Tensor] = {}
        self.optimal_params: Dict[str, torch.Tensor] = {}
        self._has_fisher = False

    def compute_fisher(
        self,
        reference_loader: DataLoader,
        criterion: Callable[[torch.Tensor, torch.Tensor], torch.Tensor],
        device: str = "cpu",
    ) -> Dict[str, torch.Tensor]:
        self.model.eval()
        fisher_new: Dict[str, torch.Tensor] = {}

        for inputs, targets in reference_loader:
            inputs, targets = inputs.to(device), targets.to(device)
            self.model.zero_grad()
            outputs = self.model(inputs)
            loss = criterion(outputs, targets)
            loss.backward(retain_graph=False)

            batch_size_actual = inputs.size(0)
            for name, param in self.model.named_parameters():
                if param.grad is None:
                    continue
                # E[g_i²] ≈ B * E[(grad_mean)²]  (correção por amostras)
                grad_sq = param.grad.detach().clone() ** 2 * batch_size_actual
                if name not in fisher_new:
                    fisher_new[name] = grad_sq
                else:
                    fisher_new[name] += grad_sq

        # Normalization
        n_samples = max(1, len(reference_loader.dataset))
        for name in fisher_new:
            fisher_new[name] /= n_samples

        self.fisher = fisher_new
        self.optimal_params = {n: p.detach().clone() for n, p in self.model.named_parameters() if p.requires_grad}
        self._has_fisher = True
        return self.fisher

    def ewc_loss(self, device="cpu") -> torch.Tensor:
        if not self._has_fisher:
            return torch.tensor(0.0, device=device)

        loss = torch.tensor(0.0, device=device)
        for name, param in self.model.named_parameters():
            if name in self.fisher and name in self.optimal_params:
                fisher_term = self.fisher[name].to(device)
                opt_param = self.optimal_params[name].to(device)
                loss += (fisher_term * (param - opt_param) ** 2).sum()

        return loss * (self.importance / 2.0)

class ExperienceReplay:
    def __init__(self, capacity: int = 1000):
        self.capacity = capacity
        self.buffer: List[Tuple[torch.Tensor, torch.Tensor]] = []
        self.position = 0

    def push(self, state: torch.Tensor, target: torch.Tensor):
        if len(self.buffer) < self.capacity:
            self.buffer.append((state, target))
        else:
            self.buffer[self.position] = (state, target)
        self.position = (self.position + 1) % self.capacity

    def sample(self, batch_size: int) -> Tuple[torch.Tensor, torch.Tensor]:
        batch = random.sample(self.buffer, min(batch_size, len(self.buffer)))
        states, targets = zip(*batch)
        return torch.stack(states), torch.stack(targets)

    def __len__(self):
        return len(self.buffer)

class ContinualLearningAgent:
    def __init__(self, model, importance=1e3, replay_capacity=5000):
        self.model = model
        self.ewc = ElasticWeightConsolidation(model, importance)
        self.replay = ExperienceReplay(replay_capacity)
