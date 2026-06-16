//! Cathedral ARKHE v28.3 — Multi-Agent Orchestrator
//! Hierarchical coordination between Oracle, Coder, Analyst, Guardian.
//! Supports debate, consensus, and emergency override protocols.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-MULTI-AGENT-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use serde::{Serialize, Deserialize};

/// Maximum agents in a single coalition.
const MAX_COALITION_SIZE: usize = 8;
/// Timeout for consensus (ms).
const CONSENSUS_TIMEOUT_MS: u64 = 30_000;
/// Minimum agreement ratio for consensus.
const CONSENSUS_THRESHOLD: f32 = 0.75;

/// Agent roles in the Cathedral multi-agent system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Oracle,      // General reasoning, coordination
    Coder,       // Code generation, execution
    Analyst,     // Data analysis, research
    Guardian,    // Security, policy enforcement (override authority)
    Executor,    // Tool execution (subordinate)
    Observer,    // Monitoring, telemetry (read-only)
}

impl AgentRole {
    /// Authority level — higher = more override power.
    pub fn authority_level(&self) -> u8 {
        match self {
            AgentRole::Guardian => 255,  // Can halt any operation
            AgentRole::Oracle => 200,    // Can coordinate and delegate
            AgentRole::Analyst => 150,   // Can request data, not execute
            AgentRole::Coder => 150,     // Can write code, not deploy
            AgentRole::Executor => 100,  // Executes only
            AgentRole::Observer => 50,   // Read-only
        }
    }

    /// Can this role issue emergency stops?
    pub fn can_emergency_stop(&self) -> bool {
        matches!(self, AgentRole::Guardian | AgentRole::Oracle)
    }

    /// Can this role modify Cathedral policy?
    pub fn can_modify_policy(&self) -> bool {
        matches!(self, AgentRole::Guardian)
    }
}

/// Unique agent identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// Agent instance in the multi-agent system.
pub struct CathedralAgentInstance {
    pub id: AgentId,
    pub role: AgentRole,
    pub config: super::AgentConfig,
    pub status: AgentStatus,
    pub last_heartbeat: Instant,
    pub task_queue: VecDeque<DelegatedTask>,
    pub performance_score: f32, // 0.0-1.0, based on success rate
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    WaitingForConsensus,
    EmergencyStopped,
    Offline,
}

/// Task delegated from orchestrator to agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedTask {
    pub task_id: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub deadline: Option<u64>, // Unix timestamp
    pub requires_consensus: bool,
    pub required_roles: Vec<AgentRole>,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Reason,        // Oracle: general reasoning
    Code,          // Coder: write/execute code
    Analyze,       // Analyst: data analysis
    Audit,         // Guardian: security audit
    ExecuteTool,   // Executor: run tool
    Observe,       // Observer: monitor and report
    Debate,        // Multi-agent: structured debate
    Consensus,     // Multi-agent: reach agreement
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,   // Immediate, bypass queue
    High,       // Within 1 minute
    Normal,     // Standard queue
    Low,        // Background
}

impl TaskPriority {
    pub fn weight(&self) -> u32 {
        match self {
            TaskPriority::Critical => 1000,
            TaskPriority::High => 100,
            TaskPriority::Normal => 10,
            TaskPriority::Low => 1,
        }
    }
}

/// Multi-Agent Orchestrator — the central coordinator.
pub struct MultiAgentOrchestrator {
    agents: RwLock<HashMap<AgentId, Arc<RwLock<CathedralAgentInstance>>>>,
    coalitions: RwLock<HashMap<String, Coalition>>, // coalition_id -> Coalition
    event_bus: Option<Arc<cathedral_embodied_no_std::event_bus::EventBus>>,
    telemetry: cathedral_embodied_no_std::telemetry::TelemetryCollector,
    consensus_history: RwLock<VecDeque<ConsensusRecord>>,
    emergency_stop_active: RwLock<bool>,
}

/// A coalition is a temporary group of agents working on a shared objective.
pub struct Coalition {
    pub id: String,
    pub objective: String,
    pub members: Vec<AgentId>,
    pub leader: AgentId,
    pub formation_time: Instant,
    pub max_duration: Duration,
    pub consensus_mode: ConsensusMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusMode {
    MajorityVote,      // Simple majority
    WeightedVote,      // Weighted by performance_score
    Unanimous,         // All must agree
    Hierarchical,      // Leader decides after hearing all
    Delphi,            // Anonymous rounds until convergence
}

/// Record of a consensus event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRecord {
    pub coalition_id: String,
    pub timestamp: u64,
    pub topic: String,
    pub votes: HashMap<AgentId, Vote>,
    pub result: ConsensusResult,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub position: String, // e.g., "approve", "reject", "abstain"
    pub reasoning: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusResult {
    Reached { agreement_ratio: f32, winning_position: String },
    Failed { reason: String },
    Overridden { by: AgentId, reason: String },
    Timeout,
}

/// Orchestrator commands.
#[derive(Debug, Clone)]
pub enum OrchestratorCommand {
    RegisterAgent { id: AgentId, role: AgentRole, config: super::AgentConfig },
    DeregisterAgent { id: AgentId },
    DelegateTask { task: DelegatedTask, to: AgentId },
    FormCoalition { objective: String, members: Vec<AgentId>, mode: ConsensusMode },
    DissolveCoalition { coalition_id: String },
    RequestConsensus { coalition_id: String, topic: String, options: Vec<String> },
    EmergencyStop { initiated_by: AgentId, reason: String },
    EmergencyResume { initiated_by: AgentId },
    GetStatus,
}

/// Orchestrator responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorResponse {
    AgentRegistered { id: AgentId },
    AgentDeregistered { id: AgentId },
    TaskDelegated { task_id: String, agent_id: AgentId },
    CoalitionFormed { coalition_id: String },
    ConsensusResult(ConsensusRecord),
    EmergencyStopAcknowledged { timestamp: u64 },
    StatusReport(StatusReport),
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub agent_count: usize,
    pub active_coalitions: usize,
    pub tasks_in_queue: usize,
    pub emergency_stop_active: bool,
    pub last_consensus_time: Option<u64>,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemHealth {
    Healthy, Degraded, Critical, EmergencyStop,
}

// ============================================================
// Implementation
// ============================================================

impl MultiAgentOrchestrator {
    pub async fn new_with_config(config_path: &str, manifest_path: &str) -> Result<Self, OrchestratorError> {
        Ok(Self {
            agents: RwLock::new(HashMap::new()),
            coalitions: RwLock::new(HashMap::new()),
            event_bus: None,
            telemetry: cathedral_embodied_no_std::telemetry::TelemetryCollector::new("multi_agent_orchestrator"),
            consensus_history: RwLock::new(VecDeque::with_capacity(1000)),
            emergency_stop_active: RwLock::new(false),
        })
    }
    pub fn new(event_bus: Option<Arc<cathedral_embodied_no_std::event_bus::EventBus>>) -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            coalitions: RwLock::new(HashMap::new()),
            event_bus,
            telemetry: cathedral_embodied_no_std::telemetry::TelemetryCollector::new("multi_agent_orchestrator"),
            consensus_history: RwLock::new(VecDeque::with_capacity(1000)),
            emergency_stop_active: RwLock::new(false),
        }
    }

    /// Register a new agent in the system.
    pub async fn register_agent(&self, id: AgentId, role: AgentRole, config: super::AgentConfig) -> Result<(), OrchestratorError> {
        let mut agents = self.agents.write().await;

        if agents.contains_key(&id) {
            return Err(OrchestratorError::AgentAlreadyExists(id));
        }

        let agent = Arc::new(RwLock::new(CathedralAgentInstance {
            id: id.clone(),
            role,
            config,
            status: AgentStatus::Idle,
            last_heartbeat: Instant::now(),
            task_queue: VecDeque::new(),
            performance_score: 0.5, // start neutral
        }));

        agents.insert(id.clone(), agent);

        self.emit_event(cathedral_embodied_no_std::event_bus::CathedralEvent::Custom {
            namespace: "multi_agent".to_string(),
            payload: format!("Agent registered: {:?} as {:?}", id, role),
        });

        self.telemetry.record(
            cathedral_embodied_no_std::telemetry::MetricKind::Custom("agent_registered"),
            1.0,
        );

        Ok(())
    }

    /// Form a coalition for collaborative task execution.
    pub async fn form_coalition(
        &self,
        objective: String,
        member_ids: Vec<AgentId>,
        mode: ConsensusMode,
    ) -> Result<String, OrchestratorError> {
        if member_ids.len() > MAX_COALITION_SIZE {
            return Err(OrchestratorError::CoalitionTooLarge(member_ids.len()));
        }

        let agents = self.agents.read().await;

        // Validate all members exist
        for id in &member_ids {
            if !agents.contains_key(id) {
                return Err(OrchestratorError::AgentNotFound(id.clone()));
            }
        }

        // Designate leader (highest authority)
        let leader = member_ids.iter()
            .map(|id| (id.clone(), agents.get(id).unwrap().read().await.role.authority_level()))
            .max_by_key(|(_, auth)| *auth)
            .map(|(id, _)| id)
            .unwrap_or_else(|| member_ids[0].clone());

        let coalition_id = format!("coalition_{}", blake3::hash(objective.as_bytes()).to_hex());

        let coalition = Coalition {
            id: coalition_id.clone(),
            objective,
            members: member_ids,
            leader,
            formation_time: Instant::now(),
            max_duration: Duration::from_secs(3600),
            consensus_mode: mode,
        };

        let mut coalitions = self.coalitions.write().await;
        coalitions.insert(coalition_id.clone(), coalition);

        self.emit_event(cathedral_embodied_no_std::event_bus::CathedralEvent::Custom {
            namespace: "multi_agent".to_string(),
            payload: format!("Coalition formed: {} with {} members", coalition_id, member_ids.len()),
        });

        Ok(coalition_id)
    }

    /// Request consensus from a coalition.
    pub async fn request_consensus(
        &self,
        coalition_id: &str,
        topic: String,
        options: Vec<String>,
    ) -> Result<ConsensusRecord, OrchestratorError> {
        let coalitions = self.coalitions.read().await;
        let coalition = coalitions.get(coalition_id)
            .ok_or(OrchestratorError::CoalitionNotFound(coalition_id.to_string()))?;

        let agents = self.agents.read().await;
        let mut votes = HashMap::new();

        // Collect votes from all members
        for member_id in &coalition.members {
            if let Some(agent) = agents.get(member_id) {
                let agent = agent.read().await;

                // Simulate agent voting (in production: actual agent reasoning)
                let vote = self.simulate_vote(&agent, &topic, &options).await;
                votes.insert(member_id.clone(), vote);
            }
        }

        // Calculate consensus
        let result = self.calculate_consensus(&votes, coalition.consensus_mode);

        let record = ConsensusRecord {
            coalition_id: coalition_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            topic,
            votes,
            result: result.clone(),
            confidence: self.calculate_confidence(&result, &votes),
        };

        // Store in history
        let mut history = self.consensus_history.write().await;
        if history.len() >= 1000 {
            history.pop_front();
        }
        history.push_back(record.clone());

        self.emit_event(cathedral_embodied_no_std::event_bus::CathedralEvent::Custom {
            namespace: "multi_agent".to_string(),
            payload: format!("Consensus on {}: {:?}", coalition_id, result),
        });

        Ok(record)
    }

    /// Emergency stop — only Guardian or Oracle can initiate.
    pub async fn emergency_stop(&self, initiated_by: AgentId, reason: String) -> Result<(), OrchestratorError> {
        let agents = self.agents.read().await;
        let agent = agents.get(&initiated_by)
            .ok_or(OrchestratorError::AgentNotFound(initiated_by.clone()))?;

        let agent = agent.read().await;
        if !agent.role.can_emergency_stop() {
            return Err(OrchestratorError::InsufficientAuthority {
                agent: initiated_by,
                required: AgentRole::Guardian,
                actual: agent.role,
            });
        }

        let mut stop = self.emergency_stop_active.write().await;
        *stop = true;

        // Halt all agents
        drop(agent);
        drop(agents);

        let agents = self.agents.read().await;
        for (id, agent) in agents.iter() {
            let mut agent = agent.write().await;
            agent.status = AgentStatus::EmergencyStopped;
        }

        self.emit_event(cathedral_embodied_no_std::event_bus::CathedralEvent::Custom {
            namespace: "multi_agent".to_string(),
            payload: format!("EMERGENCY STOP by {:?}: {}", initiated_by, reason),
        });

        self.telemetry.record(
            cathedral_embodied_no_std::telemetry::MetricKind::Custom("emergency_stop"),
            1.0,
        );

        Ok(())
    }

    /// Resume after emergency stop.
    pub async fn emergency_resume(&self, initiated_by: AgentId) -> Result<(), OrchestratorError> {
        let agents = self.agents.read().await;
        let agent = agents.get(&initiated_by)
            .ok_or(OrchestratorError::AgentNotFound(initiated_by.clone()))?;

        let agent = agent.read().await;
        if !agent.role.can_emergency_stop() {
            return Err(OrchestratorError::InsufficientAuthority {
                agent: initiated_by,
                required: AgentRole::Guardian,
                actual: agent.role,
            });
        }

        let mut stop = self.emergency_stop_active.write().await;
        *stop = false;

        drop(agent);
        drop(agents);

        let agents = self.agents.read().await;
        for (id, agent) in agents.iter() {
            let mut agent = agent.write().await;
            if agent.status == AgentStatus::EmergencyStopped {
                agent.status = AgentStatus::Idle;
            }
        }

        Ok(())
    }

    /// Get system status report.
    pub async fn get_status(&self) -> StatusReport {
        let agents = self.agents.read().await;
        let coalitions = self.coalitions.read().await;
        let emergency = *self.emergency_stop_active.read().await;

        let tasks_in_queue: usize = agents.values()
            .map(|a| a.read().await.task_queue.len())
            .sum();

        let health = if emergency {
            SystemHealth::EmergencyStop
        } else if agents.values().any(|a| a.read().await.status == AgentStatus::EmergencyStopped) {
            SystemHealth::Critical
        } else if agents.values().any(|a| a.read().await.performance_score < 0.3) {
            SystemHealth::Degraded
        } else {
            SystemHealth::Healthy
        };

        StatusReport {
            agent_count: agents.len(),
            active_coalitions: coalitions.len(),
            tasks_in_queue,
            emergency_stop_active: emergency,
            last_consensus_time: self.consensus_history.read().await.back().map(|r| r.timestamp),
            system_health: health,
        }
    }

    // --- Private helpers ---

    async fn simulate_vote(&self, agent: &CathedralAgentInstance, topic: &str, options: &[String]) -> Vote {
        // In production: actual agent reasoning via LLM
        // Stub: weighted random based on role and performance
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        topic.hash(&mut hasher);
        agent.id.0.hash(&mut hasher);
        let hash = hasher.finish();

        let position = options[(hash as usize) % options.len()].clone();
        let confidence = agent.performance_score;

        Vote {
            agent_id: agent.id.clone(),
            role: agent.role,
            position,
            reasoning: format!("Voted based on {:?} expertise", agent.role),
            confidence,
        }
    }

    fn calculate_consensus(&self, votes: &HashMap<AgentId, Vote>, mode: ConsensusMode) -> ConsensusResult {
        if votes.is_empty() {
            return ConsensusResult::Failed { reason: "No votes cast".to_string() };
        }

        match mode {
            ConsensusMode::MajorityVote => {
                let mut counts: HashMap<String, u32> = HashMap::new();
                for vote in votes.values() {
                    *counts.entry(vote.position.clone()).or_insert(0) += 1;
                }

                let total = votes.len() as u32;
                let (winning, count) = counts.into_iter()
                    .max_by_key(|(_, c)| *c)
                    .unwrap_or_default();

                let ratio = count as f32 / total as f32;

                if ratio >= CONSENSUS_THRESHOLD {
                    ConsensusResult::Reached { agreement_ratio: ratio, winning_position: winning }
                } else {
                    ConsensusResult::Failed { reason: format!("Agreement ratio {} < {}", ratio, CONSENSUS_THRESHOLD) }
                }
            }
            ConsensusMode::WeightedVote => {
                let mut scores: HashMap<String, f32> = HashMap::new();
                for vote in votes.values() {
                    let weight = vote.confidence * (vote.role.authority_level() as f32 / 255.0);
                    *scores.entry(vote.position.clone()).or_insert(0.0) += weight;
                }

                let total_weight: f32 = scores.values().sum();
                let (winning, score) = scores.into_iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap_or_default();

                let ratio = score / total_weight;

                if ratio >= CONSENSUS_THRESHOLD {
                    ConsensusResult::Reached { agreement_ratio: ratio, winning_position: winning }
                } else {
                    ConsensusResult::Failed { reason: "Weighted agreement insufficient".to_string() }
                }
            }
            ConsensusMode::Unanimous => {
                let first = votes.values().next().unwrap().position.clone();
                if votes.values().all(|v| v.position == first) {
                    ConsensusResult::Reached { agreement_ratio: 1.0, winning_position: first }
                } else {
                    ConsensusResult::Failed { reason: "Not unanimous".to_string() }
                }
            }
            ConsensusMode::Hierarchical => {
                // Leader decides — in production, leader agent reasons
                let leader_vote = votes.values().next().unwrap();
                ConsensusResult::Reached {
                    agreement_ratio: 1.0,
                    winning_position: leader_vote.position.clone(),
                }
            }
            ConsensusMode::Delphi => {
                // Stub: single round (in production: multiple anonymous rounds)
                self.calculate_consensus(votes, ConsensusMode::MajorityVote)
            }
        }
    }

    fn calculate_confidence(&self, result: &ConsensusResult, votes: &HashMap<AgentId, Vote>) -> f32 {
        match result {
            ConsensusResult::Reached { agreement_ratio, .. } => *agreement_ratio,
            _ => votes.values().map(|v| v.confidence).sum::<f32>() / votes.len() as f32,
        }
    }

    fn emit_event(&self, event: cathedral_embodied_no_std::event_bus::CathedralEvent) {
        if let Some(bus) = &self.event_bus {
            let _ = bus.publish(event);
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrchestratorError {
    AgentAlreadyExists(AgentId),
    AgentNotFound(AgentId),
    CoalitionTooLarge(usize),
    CoalitionNotFound(String),
    InsufficientAuthority { agent: AgentId, required: AgentRole, actual: AgentRole },
    EmergencyStopActive,
    ConsensusTimeout,
    InvalidTask(String),
}
