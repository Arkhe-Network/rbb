import torch
import torch.nn as nn
from vector_theosis_1091_1 import OrchestratorRSI

class DummyModel(nn.Module):
    def __init__(self):
        super().__init__()
        self.layer = nn.Linear(8, 8)
    def forward(self, x):
        return self.layer(x)

m = DummyModel()
orch = OrchestratorRSI()
hook_handle = orch.attach_to_transformer_layer(m.layer)

print("Hook registration test:")
x = torch.randn(1, 10, 8) # batch=1, seq_len=10, dim=8
print("output shape:", m(x).shape)
print("Ingested states:", len(orch.vt.engine.state_history))

hook_handle.remove()
