import numpy as np
from vector_theosis_1091_1 import OrchestratorRSI

orch = OrchestratorRSI()

for i in range(10):
    vec = np.array([float(i), float(i*2)])
    orch.ingest_hidden_state(vec, token_text=f"t{i}")

# Force SINDY action
reading = orch.vt._readings[-1]
action_res = orch._execute_action("ACTIVATE_SINDY_DISCOVERY", reading)
print(action_res)
