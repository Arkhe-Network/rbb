import dynamic_system_identification_engine_1089 as dsie
import hamiltonian_temporal_implosion_1053_4 as ht
from vector_theosis_1091_1 import OrchestratorRSI

def test_imports():
    try:
        sindy = dsie.SINDyEngine(dsie.CanonicalFunctionLibrary(n_states=8))
        ham = ht.HamiltonianTemporalImplosionV5()

        def mock_sindy_callback(reading):
            return {"type": "SINDY", "message": "real SINDy"}

        def mock_ham_callback(reading):
            return {"type": "HAMILTONIAN", "message": "real Hamiltonian"}

        orch = OrchestratorRSI(sindy_callback=mock_sindy_callback, hamiltonian_callback=mock_ham_callback)
        print("Imports successful!")
    except Exception as e:
        print(f"Error: {e}")

test_imports()
