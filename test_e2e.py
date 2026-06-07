import torch
import torch.nn as nn
from vector_theosis_1091_1 import OrchestratorRSI

class DummyModel(nn.Module):
    def __init__(self):
        super().__init__()
        self.layer = nn.Linear(2, 2)
    def forward(self, x):
        return self.layer(x)

def run():
    print("Running E2E tests...")
    model = DummyModel()
    orch = OrchestratorRSI()

    handle = orch.attach_to_transformer_layer(model.layer)
    for i in range(5):
        _ = model(torch.randn(1, 2))

    print(f"Ingested length: {len(orch.vt.engine.state_history)}")

    reading = orch.vt._readings[-1]
    res_sindy = orch._execute_action("ACTIVATE_SINDY_DISCOVERY", reading)
    assert res_sindy["type"] == "SINDY"

    res_ham = orch._execute_action("ACTIVATE_HAMILTONIAN_IMPLOSION", reading)
    assert res_ham["type"] == "HAMILTONIAN"
    print("E2E Tests passed.")
    handle.remove()

if __name__ == "__main__":
    run()
