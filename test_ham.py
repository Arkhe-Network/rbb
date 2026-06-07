import numpy as np
from vector_theosis_1091_1 import OrchestratorRSI

orch = OrchestratorRSI()

# A smaller hidden state dim so it doesn't get OOM killed. 1728*8 = 13824 total_dim (complex128). That is a ~3GB matrix! Let's just run it with dim=1
for i in range(10):
    vec = np.random.randn(1)
    orch.ingest_hidden_state(vec, token_text=f"t{i}")

# Force Hamiltonian action
reading = orch.vt._readings[-1]
action_res = orch._execute_action("ACTIVATE_HAMILTONIAN_IMPLOSION", reading)
print(action_res)
