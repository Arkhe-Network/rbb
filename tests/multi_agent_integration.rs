//! Cathedral ARKHE v28.3 — Multi-Agent Integration Tests
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-TESTS-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use cathedral_agent::orchestrator::{
    MultiAgentOrchestrator, AgentId, AgentRole, AgentConfig,
    ConsensusMode, TaskType, TaskPriority, DelegatedTask,
    DebateEngine, Argument,
    HierarchyManager, CommandLevel, DelegatedAuthority,
};

fn create_test_config(role: AgentRole) -> AgentConfig {
    AgentConfig {
        name: format!("test_{:?}", role),
        system_prompt: "Test agent".to_string(),
        model_id: "test-model".to_string(),
        temperature: 0.7,
        max_tokens: 1024,
        tools: vec![],
        planning_strategy: cathedral_agent::PlanningStrategy::ReAct,
        memory_config: cathedral_agent::MemoryConfig {
            short_term_capacity: 10,
            long_term_enabled: false,
            vector_db_url: None,
            embedding_model: "test".to_string(),
        },
        guardrail_config: cathedral_agent::GuardrailConfig {
            content_filter_enabled: true,
            max_tool_execution_time_secs: 30,
            forbidden_tools: vec![],
            required_memory_proof_for: vec![],
            output_moderation_threshold: 0.7,
        },
        cathedral_policy_hash: "test".to_string(),
    }
}

#[tokio::test]
async fn test_agent_registration() {
    let orchestrator = MultiAgentOrchestrator::new(None);

    let oracle_id = AgentId("oracle_1".to_string());
    orchestrator.register_agent(oracle_id.clone(), AgentRole::Oracle, create_test_config(AgentRole::Oracle)).await.unwrap();

    let status = orchestrator.get_status().await;
    assert_eq!(status.agent_count, 1);
    assert_eq!(status.system_health, SystemHealth::Healthy);
}

#[tokio::test]
async fn test_coalition_formation() {
    let orchestrator = MultiAgentOrchestrator::new(None);

    let oracle_id = AgentId("oracle_1".to_string());
    let coder_id = AgentId("coder_1".to_string());
    let analyst_id = AgentId("analyst_1".to_string());

    orchestrator.register_agent(oracle_id.clone(), AgentRole::Oracle, create_test_config(AgentRole::Oracle)).await.unwrap();
    orchestrator.register_agent(coder_id.clone(), AgentRole::Coder, create_test_config(AgentRole::Coder)).await.unwrap();
    orchestrator.register_agent(analyst_id.clone(), AgentRole::Analyst, create_test_config(AgentRole::Analyst)).await.unwrap();

    let coalition_id = orchestrator.form_coalition(
        "Implement new substrate".to_string(),
        vec![oracle_id, coder_id, analyst_id],
        ConsensusMode::MajorityVote,
    ).await.unwrap();

    assert!(coalition_id.starts_with("coalition_"));

    let status = orchestrator.get_status().await;
    assert_eq!(status.active_coalitions, 1);
}

#[tokio::test]
async fn test_consensus_majority_vote() {
    let orchestrator = MultiAgentOrchestrator::new(None);

    let oracle_id = AgentId("oracle_1".to_string());
    let coder_id = AgentId("coder_1".to_string());
    let analyst_id = AgentId("analyst_1".to_string());

    orchestrator.register_agent(oracle_id.clone(), AgentRole::Oracle, create_test_config(AgentRole::Oracle)).await.unwrap();
    orchestrator.register_agent(coder_id.clone(), AgentRole::Coder, create_test_config(AgentRole::Coder)).await.unwrap();
    orchestrator.register_agent(analyst_id.clone(), AgentRole::Analyst, create_test_config(AgentRole::Analyst)).await.unwrap();

    let coalition_id = orchestrator.form_coalition(
        "Choose implementation approach".to_string(),
        vec![oracle_id, coder_id, analyst_id],
        ConsensusMode::MajorityVote,
    ).await.unwrap();

    let record = orchestrator.request_consensus(
        &coalition_id,
        "Should we use Rust or Python?".to_string(),
        vec!["Rust".to_string(), "Python".to_string()],
    ).await.unwrap();

    assert!(matches!(record.result, ConsensusResult::Reached { .. }));
    assert!(record.confidence > 0.0);
}

#[tokio::test]
async fn test_emergency_stop_authority() {
    let orchestrator = MultiAgentOrchestrator::new(None);

    let guardian_id = AgentId("guardian_1".to_string());
    let oracle_id = AgentId("oracle_1".to_string());
    let executor_id = AgentId("executor_1".to_string());

    orchestrator.register_agent(guardian_id.clone(), AgentRole::Guardian, create_test_config(AgentRole::Guardian)).await.unwrap();
    orchestrator.register_agent(oracle_id.clone(), AgentRole::Oracle, create_test_config(AgentRole::Oracle)).await.unwrap();
    orchestrator.register_agent(executor_id.clone(), AgentRole::Executor, create_test_config(AgentRole::Executor)).await.unwrap();

    // Guardian can emergency stop
    orchestrator.emergency_stop(guardian_id.clone(), "Security breach detected".to_string()).await.unwrap();

    let status = orchestrator.get_status().await;
    assert!(status.emergency_stop_active);
    assert_eq!(status.system_health, SystemHealth::EmergencyStop);

    // Executor cannot resume
    let result = orchestrator.emergency_resume(executor_id.clone()).await;
    assert!(result.is_err());

    // Guardian can resume
    orchestrator.emergency_resume(guardian_id.clone()).await.unwrap();
    let status = orchestrator.get_status().await;
    assert!(!status.emergency_stop_active);
}

#[test]
fn test_debate_engine() {
    let mut engine = DebateEngine::new();

    let oracle_id = AgentId("oracle_1".to_string());
    let analyst_id = AgentId("analyst_1".to_string());

    let debate_id = engine.start_debate(
        "Should Cathedral use SPHINCS+ or ML-DSA?".to_string(),
        vec![oracle_id.clone(), analyst_id.clone()],
        None,
        3,
    ).unwrap();

    engine.submit_argument(&debate_id, oracle_id.clone(), Argument {
        agent_id: oracle_id.clone(),
        position: "for".to_string(),
        claim: "SPHINCS+ is quantum-resistant".to_string(),
        evidence: vec!["NIST PQC standard".to_string()],
        reasoning_chain: vec!["Quantum computers threaten RSA".to_string(), "SPHINCS+ is hash-based".to_string()],
        confidence: 0.9,
        citations: vec!["NIST.FIPS.205".to_string()],
    }).unwrap();

    engine.submit_argument(&debate_id, analyst_id.clone(), Argument {
        agent_id: analyst_id.clone(),
        position: "against".to_string(),
        claim: "ML-DSA has smaller signatures".to_string(),
        evidence: vec!["Performance benchmarks".to_string()],
        reasoning_chain: vec!["Signature size matters for bandwidth".to_string()],
        confidence: 0.8,
        citations: vec!["NIST.FIPS.204".to_string()],
    }).unwrap();

    let verdict = engine.evaluate_debate(&debate_id).unwrap();
    assert!(verdict.winner.is_some());
    assert!(verdict.confidence > 0.0);
}

#[test]
fn test_hierarchy_delegation() {
    let mut hierarchy = HierarchyManager::new(AgentId("guardian_root".to_string()));

    let oracle_id = AgentId("oracle_1".to_string());
    let coder_id = AgentId("coder_1".to_string());

    hierarchy.add_agent(oracle_id.clone(), CommandLevel::Tactical, AgentId("guardian_root".to_string())).unwrap();
    hierarchy.add_agent(coder_id.clone(), CommandLevel::Operational, oracle_id.clone()).unwrap();

    // Oracle can delegate tool execution to coder
    let delegation = hierarchy.delegate_authority(
        oracle_id.clone(),
        coder_id.clone(),
        DelegatedAuthority::ToolExecution,
        "substrate_319.1".to_string(),
        3600,
    ).unwrap();

    assert!(hierarchy.has_authority(&coder_id, DelegatedAuthority::ToolExecution, "substrate_319.1"));
    assert!(!hierarchy.has_authority(&coder_id, DelegatedAuthority::PolicyModification, "substrate_319.1"));

    // Chain of command
    let chain = hierarchy.chain_of_command(&coder_id);
    assert_eq!(chain.len(), 3); // coder -> oracle -> guardian
}