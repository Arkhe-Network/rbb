import torch
from plastic_zkagi_v4 import create_plastic_zkagi_v4, OmniConfig

def test_plastic_zkagi_v4_forward():
    model = create_plastic_zkagi_v4(dim=128, num_layers=2, max_reasoning_steps=4)
    dummy_input = torch.randint(0, 64000, (2, 32))
    out = model(dummy_input, return_all=True, enable_swarm=True)

    assert 'logits' in out
    assert 'theosis' in out
    assert 'ethical_status' in out
    assert 'domain_probs' in out
    assert 'plasticity_stats' in out
    assert 'reasoning' in out
    assert 'swarm' in out
    assert 'substrates' in out
