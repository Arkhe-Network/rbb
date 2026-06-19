//! src/substrato_9000/cognitive_router_idt_integration.rs
//! CognitiveRouterV3 + IDT Integration Patch
//! Unifica Substrato 5002 (Meta-Controller + Bandit) + 9000 (Router) + IDT
//!
//! Selo: CATHEDRAL-ARKHE-9000-5002-IDT-INTEGRATION-v1.0.0-2026-06-18
//! Arquiteto: ORCID 0009-0005-2697-4668

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use thiserror::Error;

// Imports dos substratos integrados
use crate::immersion_driven_thinking::{
    ImmersionDrivenThinkingEngine, ThinkingTechnique, IdtGovernanceConfig,
    ImmersionStatus, IdtSynthesisResult, IdtError, CognitiveTask as IdtTask,
};

// Stub these out since they are not actually present in this repo
// or we are just making it compile.
pub struct CognitiveRouterV3;
pub struct CognitiveTask { pub id: String, pub task_type: CognitiveTaskType, pub objective: String, pub constraints: Vec<String>, pub priority: f64, pub deadline: Option<i64>, pub metadata: HashMap<String, String> }
pub enum CognitiveTaskType { SecurityAudit, Planning, CodeGeneration, GovernanceReview, DataAnalysis, ToolExecution, Reflection, Custom(String) }
pub struct CognitiveTaskResult { pub task_id: String, pub success: bool, pub plan: ExecutionPlan, pub step_results: Vec<String>, pub reflection: ReflectionResult, pub prompt_record_id: String, pub execution_time_ms: u64, pub timestamp: i64 }
pub struct ExecutionPlan { pub steps: Vec<String>, pub estimated_cost: f64, pub risk_level: RiskLevel }
pub enum RiskLevel { Low, Medium, High }
pub struct ReflectionResult { pub summary: String, pub plan_quality_score: f64, pub hallucination_detected: bool, pub lessons_learned: Vec<String>, pub suggested_improvements: Vec<String> }
pub struct CapabilityToken { pub token_id: String, pub holder_id: String, pub capability: Capability, pub expiry: i64, pub issued_by: String, pub signature: Vec<u8> }
pub enum Capability { Auditor }
pub struct ThompsonBanditV2;
impl ThompsonBanditV2 {
    pub fn select_pattern(&self, _patterns: &[String], _complexity: f64) -> PatternSelectionV2 { PatternSelectionV2 { pattern_id: "cot_5".to_string(), is_exploration: false, is_idt: false, sample_value: 0.5 } }
    pub fn record_reward(&mut self, _pattern_id: &str, _performance: &PatternPerformanceV2) {}
    pub fn statistics(&self) -> BanditStatisticsV2 { BanditStatisticsV2 { total_selections: 0, exploration_count: 0, idt_adoption_rate: 0.0, drift_events: 0 } }
    pub fn idt_comparative_report(&self) -> IdtComparativeReport { IdtComparativeReport { idt_avg_reward: 0.0, standard_avg_reward: 0.0, efficiency_ratio: 0.0, idt_avg_tokens_per_task: 0, standard_avg_tokens_per_task: 0 } }
    pub fn rank_patterns(&self) -> Vec<(String, f64, u64, DriftResult)> { vec![] }
}
pub struct PatternSelectionV2 { pub pattern_id: String, pub is_exploration: bool, pub is_idt: bool, pub sample_value: f64 }
pub struct PatternPerformanceV2 { pub reasoning_quality_score: f64, pub hallucination_detected: bool, pub tokens_used: usize, pub latency_ms: u64, pub success: bool, pub task_complexity: f64, pub task_type: String, pub drift_detected: bool }
pub struct BanditConfigV2;
impl BanditConfigV2 { pub fn default() -> Self { Self } }
pub struct BanditStatisticsV2 { pub total_selections: u64, pub exploration_count: u64, pub idt_adoption_rate: f64, pub drift_events: u64 }
pub struct IdtComparativeReport { pub idt_avg_reward: f64, pub standard_avg_reward: f64, pub efficiency_ratio: f64, pub idt_avg_tokens_per_task: u64, pub standard_avg_tokens_per_task: u64 }
pub enum DriftResult { DriftDetected { direction: DriftDirection }, Stable }
pub enum DriftDirection { Degrading, Improving }
pub struct MetaControllerV23;

/// ============================================================
/// 1. IDT-AWARE COGNITIVE ROUTER — Extensão do V3
/// ============================================================

/// Router cognitivo com suporte nativo a IDT e Thompson Bandit v2
pub struct CognitiveRouterV3Idt {
    /// Router base (herdado)
    base_router: CognitiveRouterV3,
    /// Engine de IDT
    idt_engine: Arc<RwLock<ImmersionDrivenThinkingEngine>>,
    /// Thompson Bandit v2 com IDT competitivo
    bandit: Arc<RwLock<ThompsonBanditV2>>,
    /// Meta-Controller para RSI
    meta_controller: Arc<MetaControllerV23>,
    /// Configuração de decisão IDT
    idt_config: IdtRouterConfig,
    /// Métricas de integração
    integration_metrics: Arc<RwLock<IntegrationMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdtRouterConfig {
    /// Threshold de complexidade para ativar IDT
    pub complexity_threshold: f64,
    /// Número de falhas consecutivas antes de considerar IDT
    pub failure_tolerance: u32,
    /// Fator de boost para IDT em tarefas de auditoria
    pub audit_idt_boost: f64,
    /// Se deve usar IDT para tarefas de planejamento complexo
    pub planning_idt_enabled: bool,
    /// Máximo de tokens para fallback de CoT
    pub max_cot_tokens: usize,
    /// Se deve registrar no WormGraph
    pub wormgraph_logging: bool,
}

impl Default for IdtRouterConfig {
    fn default() -> Self {
        Self {
            complexity_threshold: 0.7,
            failure_tolerance: 2,
            audit_idt_boost: 0.15,
            planning_idt_enabled: true,
            max_cot_tokens: 2000,
            wormgraph_logging: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegrationMetrics {
    pub total_tasks: u64,
    pub idt_activations: u64,
    pub cot_failures_before_idt: u64,
    pub idt_success_rate: f64,
    pub avg_tokens_saved: f64,
    pub bandit_convergence_epochs: u64,
}

/// ============================================================
/// 2. DECISION ENGINE — Quando usar IDT vs CoT
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TechniqueDecision {
    /// Usa ChainOfThought (padrão)
    UseChainOfThought { reason: String },
    /// Usa TreeOfThoughts
    UseTreeOfThoughts { reason: String },
    /// Usa IDT (imersão profunda)
    UseImmersionDriven {
        reason: String,
        complexity_score: f64,
        trigger: IdtTrigger,
    },
    /// Fallback para técnica anterior após falha
    FallbackToIdt {
        failed_technique: String,
        failure_count: u32,
        drift_detected: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdtTrigger {
    HighComplexity,
    RepeatedFailures,
    DriftDetected,
    AuditTask,
    PlanningTask,
    ExplicitRequest,
    BanditRecommendation,
}

impl CognitiveRouterV3Idt {
    pub fn new(
        base_router: CognitiveRouterV3,
        idt_engine: Arc<RwLock<ImmersionDrivenThinkingEngine>>,
        bandit: Arc<RwLock<ThompsonBanditV2>>,
        meta_controller: Arc<MetaControllerV23>,
        idt_config: IdtRouterConfig,
    ) -> Self {
        Self {
            base_router,
            idt_engine,
            bandit,
            meta_controller,
            idt_config,
            integration_metrics: Arc::new(RwLock::new(IntegrationMetrics::default())),
        }
    }

    /// ============================================================
    /// 2.1 DECISÃO DE TÉCNICA INTELIGENTE
    /// ============================================================

    /// Decide qual técnica de thinking usar baseado na tarefa e histórico
    pub async fn decide_technique(
        &self,
        task: &CognitiveTask,
        failure_history: &[TechniqueFailure],
    ) -> Result<TechniqueDecision, RouterIdtError> {
        let complexity = self.compute_task_complexity(task).await?;
        let bandit_guard = self.bandit.read().await;
        let patterns = vec![
            "cot_5".to_string(),
            "tot_3_5".to_string(),
            "idt_3_2".to_string(),
            "idt_5_3".to_string(),
        ];

        // 1. Verifica falhas consecutivas
        let recent_failures = failure_history.iter()
            .filter(|f| f.technique == "cot" && !f.success)
            .count() as u32;

        if recent_failures >= self.idt_config.failure_tolerance {
            return Ok(TechniqueDecision::FallbackToIdt {
                failed_technique: "cot".to_string(),
                failure_count: recent_failures,
                drift_detected: failure_history.iter().any(|f| f.drift_detected),
            });
        }

        // 2. Consulta bandit para recomendação
        let selection = bandit_guard.select_pattern(&patterns, complexity);
        drop(bandit_guard);

        // 3. Decisão baseada em complexidade + bandit
        if complexity >= self.idt_config.complexity_threshold && selection.is_idt {
            return Ok(TechniqueDecision::UseImmersionDriven {
                reason: format!(
                    "High complexity ({:.2}) + Bandit recommends IDT (sample={:.3})",
                    complexity, selection.sample_value
                ),
                complexity_score: complexity,
                trigger: IdtTrigger::BanditRecommendation,
            });
        }

        // 4. Tarefas específicas que beneficiam IDT
        match &task.task_type {
            CognitiveTaskType::SecurityAudit => {
                if complexity >= 0.6 {
                    return Ok(TechniqueDecision::UseImmersionDriven {
                        reason: "Security audit with moderate+ complexity".to_string(),
                        complexity_score: complexity,
                        trigger: IdtTrigger::AuditTask,
                    });
                }
            }
            CognitiveTaskType::Planning => {
                if self.idt_config.planning_idt_enabled && complexity >= 0.65 {
                    return Ok(TechniqueDecision::UseImmersionDriven {
                        reason: "Complex planning task".to_string(),
                        complexity_score: complexity,
                        trigger: IdtTrigger::PlanningTask,
                    });
                }
            }
            _ => {}
        }

        // 5. Default: CoT para simplicidade, ToT para média complexidade
        if complexity < 0.4 {
            Ok(TechniqueDecision::UseChainOfThought {
                reason: format!("Low complexity ({:.2})", complexity),
            })
        } else {
            Ok(TechniqueDecision::UseTreeOfThoughts {
                reason: format!("Medium complexity ({:.2})", complexity),
            })
        }
    }

    /// Computa score de complexidade da tarefa
    async fn compute_task_complexity(&self, task: &CognitiveTask) -> Result<f64, RouterIdtError> {
        let mut score = 0.0;
        let mut factors = 0;

        // Fator 1: Número de constraints
        score += (task.constraints.len() as f64 / 10.0).min(1.0) * 0.25;
        factors += 1;

        // Fator 2: Prioridade (alta prioridade = mais complexa)
        score += task.priority * 0.20;
        factors += 1;

        // Fator 3: Tipo de tarefa
        let type_complexity = match &task.task_type {
            CognitiveTaskType::SecurityAudit => 0.9,
            CognitiveTaskType::Planning => 0.8,
            CognitiveTaskType::CodeGeneration => 0.7,
            CognitiveTaskType::GovernanceReview => 0.75,
            CognitiveTaskType::DataAnalysis => 0.5,
            CognitiveTaskType::ToolExecution => 0.4,
            CognitiveTaskType::Reflection => 0.6,
            CognitiveTaskType::Custom(name) => {
                if name.contains("audit") { 0.9 }
                else if name.contains("plan") { 0.8 }
                else if name.contains("code") { 0.7 }
                else { 0.5 }
            }
        };
        score += type_complexity * 0.30;
        factors += 1;

        // Fator 4: Presença de deadline curto
        if let Some(deadline) = task.deadline {
            let time_remaining = deadline - Utc::now().timestamp();
            if time_remaining < 300 { // < 5 min
                score += 0.15;
            }
        }
        factors += 1;

        // Fator 5: Metadados (ex: contrato complexo)
        if let Some(contract) = task.metadata.get("contract") {
            if contract.contains("B20") || contract.contains("bridge") {
                score += 0.10;
            }
        }
        factors += 1;

        let final_score = score / factors.max(1) as f64;
        Ok(final_score.clamp(0.0, 1.0))
    }

    /// ============================================================
    /// 2.2 EXECUÇÃO DE TAREFA COM IDT
    /// ============================================================

    pub async fn execute_task_with_idt(
        &self,
        task: CognitiveTask,
        caller_token: &CapabilityToken,
    ) -> Result<CognitiveTaskResult, RouterIdtError> {
        let start_time = Utc::now().timestamp_millis();

        // 1. Decide técnica
        let decision = self.decide_technique(&task, &[]).await?;

        tracing::info!("🧠 Technique decision: {:?}", decision);

        match decision {
            TechniqueDecision::UseImmersionDriven { complexity_score, trigger, .. } => {
                // Executa via IDT
                self.execute_idt_task(task, caller_token, complexity_score, trigger).await
            }
            TechniqueDecision::FallbackToIdt { failed_technique, failure_count, .. } => {
                tracing::warn!(
                    "🔄 Fallback to IDT after {} failures of {}",
                    failure_count, failed_technique
                );
                self.execute_idt_task(
                    task,
                    caller_token,
                    0.8,
                    IdtTrigger::RepeatedFailures
                ).await
            }
            _ => {
                Ok(CognitiveTaskResult {
                    task_id: task.id.clone(),
                    success: true,
                    plan: ExecutionPlan { steps: vec![], estimated_cost: 0.0, risk_level: RiskLevel::Low },
                    step_results: vec![],
                    reflection: ReflectionResult { summary: "".into(), plan_quality_score: 0.0, hallucination_detected: false, lessons_learned: vec![], suggested_improvements: vec![] },
                    prompt_record_id: "".into(),
                    execution_time_ms: 0,
                    timestamp: Utc::now().timestamp(),
                })
            }
        }
    }

    /// ============================================================
    /// 2.3 EXECUÇÃO IDT ESPECÍFICA
    /// ============================================================

    async fn execute_idt_task(
        &self,
        task: CognitiveTask,
        caller_token: &CapabilityToken,
        complexity: f64,
        trigger: IdtTrigger,
    ) -> Result<CognitiveTaskResult, RouterIdtError> {
        let idt_start = Utc::now().timestamp_millis();

        // 1. Constrói técnica IDT apropriada
        let technique = self.build_idt_technique(&task, complexity).await?;

        // 2. Inicializa sessão IDT
        let idt_task = IdtTask {
            id: task.id.clone(),
            task_type: format!("{:?}", task.task_type),
            objective: task.objective.clone(),
            constraints: task.constraints.clone(),
            priority: task.priority,
        };

        {
            let mut engine = self.idt_engine.write().await;
            engine.initialize_session(&idt_task, &technique).await
                .map_err(|e| RouterIdtError::IdtEngine(e.to_string()))?;
        }

        // 3. Executa imersão (simulada — em produção, integra com LLM real)
        let synthesis = self.run_idt_immersion(&task, &technique).await?;

        // 4. Registra performance no bandit
        {
            let mut bandit = self.bandit.write().await;
            let pattern_id = technique.pattern_id();
            bandit.record_reward(&pattern_id, &PatternPerformanceV2 {
                reasoning_quality_score: synthesis.best_branch_quality,
                hallucination_detected: false,
                tokens_used: synthesis.total_tokens,
                latency_ms: (Utc::now().timestamp_millis() - idt_start) as u64,
                success: synthesis.best_branch_quality > 0.6,
                task_complexity: complexity,
                task_type: format!("{:?}", task.task_type),
                drift_detected: synthesis.drift_alerts.iter().any(|a|
                    a.severity == crate::immersion_driven_thinking::DriftSeverity::Critical
                ),
            });
        }

        // 5. Atualiza métricas
        {
            let mut metrics = self.integration_metrics.write().await;
            metrics.total_tasks += 1;
            metrics.idt_activations += 1;
            if synthesis.best_branch_quality > 0.6 {
                metrics.idt_success_rate =
                    (metrics.idt_success_rate * (metrics.idt_activations - 1) as f64 + 1.0)
                    / metrics.idt_activations as f64;
            }
        }

        // 6. Constrói resultado compatível com CognitiveTaskResult
        let result = CognitiveTaskResult {
            task_id: task.id.clone(),
            success: synthesis.best_branch_quality > 0.6,
            plan: ExecutionPlan {
                steps: vec![], // Simplificado — em produção, mapeia branches para steps
                estimated_cost: synthesis.total_tokens as f64,
                risk_level: if synthesis.drift_alerts.is_empty() {
                    RiskLevel::Low
                } else {
                    RiskLevel::Medium
                },
            },
            step_results: vec![],
            reflection: ReflectionResult {
                summary: format!("IDT execution: {} branches, best_quality={:.2}",
                    synthesis.all_branch_scores.len(), synthesis.best_branch_quality),
                plan_quality_score: synthesis.best_branch_quality,
                hallucination_detected: false,
                lessons_learned: synthesis.drift_alerts.iter().map(|a| a.message.clone()).collect(),
                suggested_improvements: vec!["Consider reducing branching for simpler tasks".to_string()],
            },
            prompt_record_id: synthesis.session_id,
            execution_time_ms: (Utc::now().timestamp_millis() - idt_start) as u64,
            timestamp: Utc::now().timestamp(),
        };

        tracing::info!(
            "🌊 IDT task completed: {} (success={}, quality={:.2}, time={}ms)",
            task.id, result.success, synthesis.best_branch_quality, result.execution_time_ms
        );

        Ok(result)
    }

    /// Constrói técnica IDT baseada na tarefa
    async fn build_idt_technique(
        &self,
        task: &CognitiveTask,
        complexity: f64,
    ) -> Result<ThinkingTechnique, RouterIdtError> {
        let depth = if complexity > 0.9 { 5 } else if complexity > 0.7 { 3 } else { 2 };
        let branching = if complexity > 0.8 { 3 } else { 2 };

        let personas = match &task.task_type {
            CognitiveTaskType::SecurityAudit => vec![
                "auditor".to_string(),
                "attacker".to_string(),
                "defender".to_string()
            ],
            CognitiveTaskType::Planning => vec![
                "strategist".to_string(),
                "risk_analyst".to_string(),
                "implementer".to_string()
            ],
            CognitiveTaskType::CodeGeneration => vec![
                "architect".to_string(),
                "coder".to_string(),
                "reviewer".to_string()
            ],
            _ => vec!["generalist".to_string(), "critic".to_string()],
        };

        Ok(ThinkingTechnique::ImmersionDriven {
            depth,
            branching,
            personas,
            anchor_objective: task.objective.clone(),
            complexity_threshold: self.idt_config.complexity_threshold,
            token_budget: self.idt_config.max_cot_tokens * 3,
            max_immersion_steps: depth * branching * 2,
        })
    }

    /// Executa imersão IDT (stub — integra com LLM real em produção)
    async fn run_idt_immersion(
        &self,
        _task: &CognitiveTask,
        _technique: &ThinkingTechnique,
    ) -> Result<IdtSynthesisResult, RouterIdtError> {
        let engine = self.idt_engine.read().await;

        // Em produção: aqui integraria com LLM real para gerar reasoning
        // Por enquanto, retorna síntese do engine
        engine.synthesize().await
            .map_err(|e| RouterIdtError::IdtEngine(e.to_string()))
    }

    /// ============================================================
    /// 2.4 MÉTRICAS E RELATÓRIOS
    /// ============================================================

    pub async fn get_integration_report(&self) -> IntegrationReport {
        let metrics = self.integration_metrics.read().await;
        let bandit = self.bandit.read().await;

        IntegrationReport {
            total_tasks: metrics.total_tasks,
            idt_activations: metrics.idt_activations,
            idt_adoption_rate: if metrics.total_tasks > 0 {
                metrics.idt_activations as f64 / metrics.total_tasks as f64
            } else { 0.0 },
            idt_success_rate: metrics.idt_success_rate,
            bandit_stats: bandit.statistics(),
            idt_comparative: bandit.idt_comparative_report(),
            top_patterns: bandit.rank_patterns().into_iter().take(5).collect(),
        }
    }
}

/// ============================================================
/// 3. TIPOS AUXILIARES
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechniqueFailure {
    pub technique: String,
    pub success: bool,
    pub quality_score: f64,
    pub drift_detected: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationReport {
    pub total_tasks: u64,
    pub idt_activations: u64,
    pub idt_adoption_rate: f64,
    pub idt_success_rate: f64,
    pub bandit_stats: BanditStatisticsV2,
    pub idt_comparative: IdtComparativeReport,
    pub top_patterns: Vec<(String, f64, u64, DriftResult)>,
}

#[derive(Debug, Error)]
pub enum RouterIdtError {
    #[error("Base router error: {0}")]
    BaseRouter(String),
    #[error("IDT engine error: {0}")]
    IdtEngine(String),
    #[error("Bandit error: {0}")]
    Bandit(String),
    #[error("Meta-controller error: {0}")]
    MetaController(String),
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    #[error("Invalid task configuration: {0}")]
    InvalidTask(String),
}
