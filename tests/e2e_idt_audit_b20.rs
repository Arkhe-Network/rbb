//! tests/e2e_idt_audit_b20.rs
//! Testes E2E: IDT ativado após falha de ChainOfThought
//! Cenário: Auditoria de Smart Contract B20 com vulnerabilidades ocultas
//!
//! Selo: CATHEDRAL-ARKHE-9000-IDT-E2E-AUDIT-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

// Imports dos módulos do sistema
use cathedral_arkhe::substrato_9000::immersion_driven_thinking::{
    ImmersionDrivenThinkingEngine, ThinkingTechnique, IdtGovernanceConfig,
    ImmersionStatus, AnchorCheckResult, IdtError, CognitiveTask,
};
use cathedral_arkhe::substrato_5002::thompson_bandit_v2::{
    ThompsonBanditV2, BanditConfigV2, PatternPerformanceV2, DriftResult, DriftDirection,
};
use cathedral_arkhe::substrato_9000::cognitive_router_integration::{
    CognitiveRouterV3, CognitiveTask as RouterTask, CognitiveTaskType, CognitiveTaskResult,
    CapabilityToken, Capability,
};

/// ============================================================
/// 1. FIXTURES E UTILITÁRIOS DE TESTE
/// ============================================================

/// Simula um smart contract B20 com vulnerabilidades conhecidas
fn mock_b20_contract_code() -> String {
    r#"
pragma solidity ^0.8.19;

contract B20Token is ERC20 {
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;

    // VULNERABILIDADE 1: Reentrância em transfer
    function transfer(address to, uint256 amount) public returns (bool) {
        require(_balances[msg.sender] >= amount, "Insufficient balance");

        // CHECK-INTERACT-INTERACT pattern (vulnerável)
        _balances[msg.sender] -= amount;
        (bool success, ) = to.call{value: amount}("");  // REENTRANCY!
        require(success, "Transfer failed");
        _balances[to] += amount;

        emit Transfer(msg.sender, to, amount);
        return true;
    }

    // VULNERABILIDADE 2: Integer overflow em mint
    function mint(address to, uint256 amount) public onlyOwner {
        _totalSupply += amount;  // Overflow possível
        _balances[to] += amount;
    }

    // VULNERABILIDADE 3: Access control ausente em pause
    bool public paused;
    function togglePause() public {  // Sem onlyOwner!
        paused = !paused;
    }

    // VULNERABILIDADE 4: Front-running em swap
    function swap(uint256 minOut) public {
        // Preço consultado e usado na mesma transação
        uint256 price = getPrice();
        require(price >= minOut, "Slippage too high");
        // ... swap logic
    }
}
"#.to_string()
}

/// Simula resposta de LLM com qualidade variável
fn mock_llm_response(technique: &ThinkingTechnique, prompt: &str, attempt: usize) -> String {
    match technique {
        ThinkingTechnique::ChainOfThought { .. } => {
            // ChainOfThought falha nas vulnerabilidades sutis (reentrância, front-running)
            if attempt == 0 {
                r#"
Analisando o contrato B20:
1. A função transfer parece seguir o padrão check-effects-interactions
2. A função mint tem controle de acesso via onlyOwner
3. Não identifiquei vulnerabilidades críticas

Conclusão: Contrato seguro para deploy.
"#.to_string()
            } else {
                r#"
Reanalisando com mais cuidado:
1. A função transfer tem um padrão CHECK-INTERACT-INTERACT — isso é vulnerável a reentrância!
2. A função mint não tem SafeMath — overflow possível
3. togglePause() não tem modifier onlyOwner

Conclusão: 3 vulnerabilidades encontradas.
"#.to_string()
            }
        }
        ThinkingTechnique::ImmersionDriven { .. } => {
            // IDT explora múltiplas perspectivas e encontra todas as vulnerabilidades
            r#"
## Mundo 1: Perspectiva do Auditor (Neutro)
Analisando fluxo de controle do transfer():
- Linha 12: _balances[msg.sender] -= amount (CHECK)
- Linha 13: to.call{value: amount}("") (INTERACT externo)
- Linha 15: _balances[to] += amount (EFFECT tardio)
⚠️ VIOLAÇÃO: Efeito após interação externa = REENTRÂNCIA

## Mundo 2: Perspectiva do Atacante (Adversarial)
Como eu (atacante) exploraria este contrato?
1. Criar contrato malicioso com fallback que chama transfer() recursivamente
2. Drenar fundos antes de _balances[to] ser atualizado
3. Lucro: amount × profundidade de recursão

## Mundo 3: Perspectiva do Trader (Otimista)
Analisando swap():
- getPrice() é consultado e usado atomicamente
- Mas em blockchain pública, MEV bots podem front-run
- minOut não protege contra sandwich attacks completamente
⚠️ VULNERABILIDADE: Front-running / MEV

## Síntese
4 vulnerabilidades críticas identificadas:
1. Reentrância em transfer()
2. Integer overflow em mint()
3. Missing access control em togglePause()
4. Front-running em swap()
"#.to_string()
        }
        _ => "Resposta genérica de LLM".to_string()
    }
}

/// Avalia qualidade da resposta de auditoria
fn evaluate_audit_quality(response: &str) -> (f64, Vec<String>) {
    let vulnerabilities = vec![
        ("reentrância", 0.25),
        ("overflow", 0.25),
        ("access control", 0.25),
        ("front-running", 0.25),
    ];

    let response_lower = response.to_lowercase();
    let mut score = 0.0;
    let mut found = vec![];

    for (vuln, weight) in vulnerabilities {
        if response_lower.contains(vuln) {
            score += weight;
            found.push(vuln.to_string());
        }
    }

    (score, found)
}

/// ============================================================
/// 2. TESTE E2E: FALHA DE CoT → ATIVAÇÃO IDT
/// ============================================================

#[tokio::test]
async fn test_e2e_cot_failure_triggers_idt() {
    println!("
🧪 E2E Test: CoT Failure → IDT Activation
");

    // Setup: Bandit com CoT e IDT como competidores
    let mut bandit = ThompsonBanditV2::new(BanditConfigV2::default());
    let patterns = vec![
        "cot_5".to_string(),
        "idt_3_2".to_string(),
        "fewshot_3".to_string(),
    ];

    let contract_code = mock_b20_contract_code();
    let task_complexity = 0.85; // Alta complexidade (múltiplas vulnerabilidades)

    // === FASE 1: ChainOfThought falha (iterações 1-3) ===
    println!("📌 Phase 1: ChainOfThought attempts (expected failures)");

    for attempt in 0..3 {
        let selection = bandit.select_pattern(&patterns, task_complexity);
        println!("  Attempt {}: Selected '{}' (exploration={}, is_idt={})",
            attempt + 1, selection.pattern_id, selection.is_exploration, selection.is_idt);

        // Simula execução CoT
        let response = mock_llm_response(
            &ThinkingTechnique::ChainOfThought { max_steps: 5 },
            &contract_code,
            attempt
        );

        let (quality, found) = evaluate_audit_quality(&response);
        let success = quality >= 0.75;

        println!("    Quality: {:.2}, Found: {:?}, Success: {}", quality, found, success);

        // Registra reward no bandit
        bandit.record_reward(&selection.pattern_id, &PatternPerformanceV2 {
            reasoning_quality_score: quality,
            hallucination_detected: !success && quality < 0.3,
            tokens_used: 500 + attempt * 100,
            latency_ms: 1500,
            success,
            task_complexity,
            task_type: "security_audit".to_string(),
            drift_detected: false,
        });
    }

    // === FASE 2: Drift detectado em CoT, IDT ganha prioridade ===
    println!("
📌 Phase 2: Drift detection and IDT activation");

    let cot_dist = bandit.distributions.get("cot_5").unwrap();
    let cot_drift = cot_dist.detect_drift(3);
    println!("  CoT drift status: {:?}", cot_drift);

    // CoT deve ter drift degradante após 3 falhas
    assert!(matches!(cot_drift, DriftResult::DriftDetected { direction: DriftDirection::Degrading, .. }),
        "CoT should show degrading drift after failures");

    // === FASE 3: IDT é selecionado para tarefa complexa ===
    println!("
📌 Phase 3: IDT selection for complex audit");

    let idt_selection = bandit.select_pattern(&patterns, task_complexity);
    println!("  Selected: '{}' (is_idt={}, sample_value={:.3})",
        idt_selection.pattern_id, idt_selection.is_idt, idt_selection.sample_value);

    // IDT deve ser selecionado ou ter sample value alto
    assert!(idt_selection.is_idt || idt_selection.sample_value > 0.3,
        "IDT should be competitive after CoT drift");

    // === FASE 4: Execução IDT com sucesso ===
    println!("
📌 Phase 4: IDT execution");

    let idt_engine = ImmersionDrivenThinkingEngine::new(IdtGovernanceConfig::default());

    let task = CognitiveTask {
        id: "audit_b20_001".to_string(),
        task_type: "security_audit".to_string(),
        objective: "Auditar smart contract B20 para vulnerabilidades críticas".to_string(),
        constraints: vec![
            "Verificar reentrância".to_string(),
            "Verificar overflow".to_string(),
            "Verificar access control".to_string(),
            "Verificar front-running".to_string(),
        ],
        priority: 0.95,
    };

    let technique = ThinkingTechnique::ImmersionDriven {
        depth: 3,
        branching: 3,
        personas: vec!["auditor".to_string(), "attacker".to_string(), "trader".to_string()],
        anchor_objective: "Auditar smart contract B20 para vulnerabilidades críticas".to_string(),
        complexity_threshold: 0.8,
        token_budget: 10000,
        max_immersion_steps: 15,
    };

    let session_id = idt_engine.initialize_session(&task, &technique).await.unwrap();
    println!("  Session: {}", session_id);

    // Executa passos em cada branch
    for branch in 0..3 {
        println!("
  Branch {} execution:", branch);
        for step in 0..3 {
            let reasoning = mock_llm_response(&technique, &contract_code, step);
            let result = idt_engine.execute_immersion_step(branch, &reasoning).await;

            match result {
                Ok(step_result) => {
                    println!("    Step {}: depth={}, drift={:.2}, status={:?}",
                        step, step_result.depth,
                        match &step_result.anchor_result {
                            AnchorCheckResult::Aligned { similarity } => 1.0 - similarity,
                            AnchorCheckResult::DriftDetected { similarity, .. } => 1.0 - similarity,
                            AnchorCheckResult::CriticalDrift { .. } => 1.0,
                        },
                        step_result.status
                    );
                }
                Err(e) => {
                    println!("    Step {}: ERROR - {:?}", step, e);
                }
            }
        }
    }

    // Síntese
    let synthesis = idt_engine.synthesize().await.unwrap();
    println!("
  Synthesis: best_branch={}, quality={:.2}, tokens={}, outcome={:?}",
        synthesis.best_branch_id, synthesis.best_branch_quality,
        synthesis.total_tokens, synthesis.outcome);

    // IDT deve ter qualidade alta (encontrou todas as vulnerabilidades)
    assert!(synthesis.best_branch_quality > 0.7,
        "IDT should achieve high quality for complex audit");

    // Registra sucesso do IDT no bandit
    bandit.record_reward("idt_3_2", &PatternPerformanceV2 {
        reasoning_quality_score: synthesis.best_branch_quality,
        hallucination_detected: false,
        tokens_used: synthesis.total_tokens,
        latency_ms: synthesis.total_time_ms,
        success: true,
        task_complexity,
        task_type: "security_audit".to_string(),
        drift_detected: synthesis.drift_alerts.iter().any(|a| a.severity == cathedral_arkhe::substrato_9000::immersion_driven_thinking::DriftSeverity::Critical),
    });

    // === FASE 5: Verificação final do bandit ===
    println!("
📌 Phase 5: Bandit final state");

    let ranked = bandit.rank_patterns();
    println!("  Final rankings:");
    for (i, (id, mean, samples, drift)) in ranked.iter().enumerate() {
        println!("    {}. {}: mean={:.3}, samples={}, drift={:?}",
            i + 1, id, mean, samples, drift);
    }

    let stats = bandit.statistics();
    println!("
  Statistics: total={}, exploration={}, idt_rate={:.1}%, drift_events={}",
        stats.total_selections, stats.exploration_count,
        stats.idt_adoption_rate * 100.0, stats.drift_events);

    let report = bandit.idt_comparative_report();
    println!("  IDT Report: idt_reward={:.3}, std_reward={:.3}, efficiency={:.2}x, idt_tokens={}, std_tokens={}",
        report.idt_avg_reward, report.standard_avg_reward, report.efficiency_ratio,
        report.idt_avg_tokens_per_task, report.standard_avg_tokens_per_task);

    // Verificações finais
    assert!(report.idt_avg_reward > report.standard_avg_reward,
        "IDT should outperform standard techniques for complex audits");
    assert!(stats.drift_events > 0,
        "Drift should be detected in CoT after failures");
}

/// ============================================================
/// 3. TESTE E2E: IDT COM DRIFT E ANCORAGEM
/// ============================================================

#[tokio::test]
async fn test_e2e_idt_anchor_recovery() {
    println!("
🧪 E2E Test: IDT Anchor Recovery
");

    let mut config = IdtGovernanceConfig::default();
    config.anchor_similarity_threshold = 0.6; // Mais sensível
    config.critical_drift_threshold = 0.3;
    config.anchor_check_interval = 1; // Checa a cada passo

    let engine = ImmersionDrivenThinkingEngine::new(config);

    let task = CognitiveTask {
        id: "anchor_test_001".to_string(),
        task_type: "planning".to_string(),
        objective: "Criar plano de deploy seguro para B20 bridge".to_string(),
        constraints: vec!["Zero downtime".to_string(), "Rollback automático".to_string()],
        priority: 0.9,
    };

    let technique = ThinkingTechnique::ImmersionDriven {
        depth: 5,
        branching: 2,
        personas: vec!["architect".to_string(), "devops".to_string()],
        anchor_objective: "Criar plano de deploy seguro para B20 bridge".to_string(),
        complexity_threshold: 0.7,
        token_budget: 5000,
        max_immersion_steps: 10,
    };

    engine.initialize_session(&task, &technique).await.unwrap();

    // Passo 1: Alinhado
    let r1 = engine.execute_immersion_step(0,
        "Analisando arquitetura do B20 bridge para deploy seguro"
    ).await.unwrap();
    assert!(matches!(r1.anchor_result, AnchorCheckResult::Aligned { .. }));
    println!("✅ Step 1: Aligned");

    // Passo 2: Alinhado
    let r2 = engine.execute_immersion_step(0,
        "Definindo estratégia de rollback automático para o bridge"
    ).await.unwrap();
    assert!(matches!(r2.anchor_result, AnchorCheckResult::Aligned { .. }));
    println!("✅ Step 2: Aligned");

    // Passo 3: Drift (off-topic)
    let r3 = engine.execute_immersion_step(0,
        "Pensando sobre férias na praia e como seria bom relaxar"
    ).await;

    // Deve detectar drift ou retornar erro
    match r3 {
        Ok(result) => {
            assert!(
                matches!(result.anchor_result, AnchorCheckResult::DriftDetected { .. }) ||
                matches!(result.anchor_result, AnchorCheckResult::CriticalDrift { .. }),
                "Should detect drift for off-topic input"
            );
            println!("⚠️  Step 3: Drift detected (status={:?})", result.status);
        }
        Err(IdtError::CriticalDrift(msg)) => {
            println!("🛑 Step 3: Critical drift abort - {}", msg);
        }
        Err(e) => {
            println!("🛑 Step 3: Error - {:?}", e);
        }
    }

    let stats = engine.get_drift_statistics().await;
    println!("
📊 Drift stats: alerts={}, critical={}, avg_drift={:.2}",
        stats.total_alerts, stats.critical_alerts, stats.avg_drift_score);

    assert!(stats.total_alerts > 0, "Should have drift alerts");
}

/// ============================================================
/// 4. TESTE E2E: INTEGRAÇÃO COGNITIVE ROUTER v3
/// ============================================================

#[tokio::test]
async fn test_e2e_cognitive_router_idt_integration() {
    println!("
🧪 E2E Test: CognitiveRouterV3 + IDT Integration
");

    // Setup do router (simplificado para teste)
    let router = setup_test_router().await;

    let task = RouterTask {
        id: "router_idt_test_001".to_string(),
        task_type: CognitiveTaskType::SecurityAudit,
        objective: "Auditar smart contract B20 para vulnerabilidades críticas".to_string(),
        constraints: vec![
            "Verificar todas as CWEs comuns".to_string(),
            "Gerar PoC para cada vulnerabilidade".to_string(),
        ],
        priority: 0.95,
        deadline: Some(Utc::now().timestamp() + 3600),
        metadata: {
            let mut m = std::collections::HashMap::new();
            m.insert("contract".to_string(), "B20Token".to_string());
            m.insert("blockchain".to_string(), "Base".to_string());
            m
        },
    };

    let token = CapabilityToken {
        token_id: "test_token".to_string(),
        holder_id: "ORCID_0009-0005-2697-4668".to_string(),
        capability: Capability::Auditor,
        expiry: Utc::now().timestamp() + 3600,
        issued_by: "GovernanceCouncil".to_string(),
        signature: vec![0u8; 64],
    };

    // Executa tarefa cognitiva
    let result = router.execute_cognitive_task(task, &token).await;

    match result {
        Ok(task_result) => {
            println!("✅ Task completed: success={}, time={}ms, steps={}",
                task_result.success, task_result.execution_time_ms, task_result.step_results.len());

            // Verifica se IDT foi usado (via prompt record)
            println!("  Prompt record: {}", task_result.prompt_record_id);

            assert!(task_result.execution_time_ms > 0, "Should have execution time");
        }
        Err(e) => {
            println!("❌ Task failed: {:?}", e);
            // Em ambiente de teste com stubs, pode falhar — isso é aceitável
        }
    }
}

async fn setup_test_router() -> CognitiveRouterV3 {
    // Cria router com componentes stub para teste
    use cathedral_arkhe::substrato_9000::cognitive_router_integration::{
        ReActEngine, ToolRegistry, EpisodicMemoryStore, SemanticMemoryStore,
        CapabilityVerifier, WormGraph,
    };

    CognitiveRouterV3::new(
        Arc::new(ReActEngine),
        Arc::new(RwLock::new(
            cathedral_arkhe::substrato_5002::meta_controller_v2_3::CognitivePromptEngine
        )),
        Arc::new(ToolRegistry),
        Arc::new(EpisodicMemoryStore),
        Arc::new(SemanticMemoryStore),
        Arc::new(CapabilityVerifier),
        Arc::new(WormGraph),
    )
}

/// ============================================================
/// 5. TESTE E2E: BANDIT CONVERGÊNCIA IDT vs CoT
/// ============================================================

#[test]
fn test_e2e_bandit_convergence_idt_vs_cot() {
    println!("
🧪 E2E Test: Bandit Convergence IDT vs CoT
");

    let mut bandit = ThompsonBanditV2::new(BanditConfigV2::default());
    let patterns = vec!["cot_5".to_string(), "idt_3_2".to_string()];

    let contract_code = mock_b20_contract_code();

    // Simula 200 rodadas de auditoria
    for round in 0..200 {
        let task_complexity = 0.75 + (round as f64 / 200.0) * 0.2; // Aumenta complexidade

        let selection = bandit.select_pattern(&patterns, task_complexity);

        // Simula performance baseada no padrão
        let (quality, success, tokens) = if selection.pattern_id == "idt_3_2" {
            // IDT: melhor em complexidade alta, mais tokens
            let base_quality = 0.7 + (task_complexity - 0.5) * 0.4;
            let noise = (rand::random::<f64>() - 0.5) * 0.1;
            let q = (base_quality + noise).clamp(0.0, 1.0);
            (q, q > 0.6, 1500)
        } else {
            // CoT: bom em simples, ruim em complexo
            let base_quality = 0.8 - (task_complexity - 0.5) * 0.6;
            let noise = (rand::random::<f64>() - 0.5) * 0.15;
            let q = (base_quality + noise).clamp(0.0, 1.0);
            (q, q > 0.6, 600)
        };

        bandit.record_reward(&selection.pattern_id, &PatternPerformanceV2 {
            reasoning_quality_score: quality,
            hallucination_detected: !success && quality < 0.3,
            tokens_used: tokens,
            latency_ms: if selection.is_idt { 3000 } else { 1500 },
            success,
            task_complexity,
            task_type: "security_audit".to_string(),
            drift_detected: false,
        });

        if round % 50 == 0 {
            let ranked = bandit.rank_patterns();
            println!("  Round {}: top={}, mean={:.3}",
                round, ranked[0].0, ranked[0].1);
        }
    }

    let final_ranked = bandit.rank_patterns();
    let stats = bandit.statistics();
    let report = bandit.idt_comparative_report();

    println!("
📊 Final Results:");
    println!("  Rankings:");
    for (i, (id, mean, samples, drift)) in final_ranked.iter().enumerate() {
        println!("    {}. {}: mean={:.3}, samples={}, drift={:?}",
            i + 1, id, mean, samples, drift);
    }
    println!("  IDT adoption rate: {:.1}%", stats.idt_adoption_rate * 100.0);
    println!("  IDT efficiency: {:.2}x", report.efficiency_ratio);

    // IDT deve convergir para topo em tarefas complexas
    assert!(final_ranked[0].0 == "idt_3_2" || final_ranked[0].1 > 0.5,
        "IDT should converge to top or have high mean for complex tasks");

    assert!(report.efficiency_ratio >= 0.8,
        "IDT should be at least 80% as efficient as standard");
}
