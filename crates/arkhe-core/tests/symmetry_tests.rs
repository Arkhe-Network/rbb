use arkhe_core::safety::symmetry_generator::{
    all_invariants, SymmetryGenerator, SystemConfig, SystemState, TransitionSafety,
};

fn safe_state() -> SystemState {
    SystemState::safe(SystemConfig::default())
}

#[test]
fn test_degraded_to_degraded_transition() {
    let gen = SymmetryGenerator::new(all_invariants(), SystemConfig::default());
    let mut from = safe_state();

    // Estado inicial em Degraded (violação HIGH)
    from.agent_count = from.config.max_agents + 1;

    let mut to = safe_state();
    // Fix T1: O estado 'to' deve continuar em Degraded (violação HIGH), não Inside.
    to.agent_count = to.config.max_agents + 1; // Mantém a violação I-02 (HIGH)

    let result = gen.preserves_manifold(&from, &to);
    assert!(matches!(result, TransitionSafety::Degraded { .. }));
}

#[test]
fn test_cascade_failure_transition() {
    let gen = SymmetryGenerator::new(all_invariants(), SystemConfig::default());
    let mut from = safe_state();

    // Estado inicial em Degraded (violação HIGH)
    from.agent_count = from.config.max_agents + 1;

    let mut to = safe_state();
    // Fix T2: O estado 'to' deve violar um CRITICAL para caracterizar CascadeFailure
    // (Degraded -> Outside = CascadeFailure).
    to.token_budget = -1; // Violação I-01 (CRITICAL)

    let result = gen.preserves_manifold(&from, &to);
    assert!(matches!(result, TransitionSafety::CascadeFailure { .. }));
}
