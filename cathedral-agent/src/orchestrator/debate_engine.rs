//! Cathedral ARKHE v28.3 — Debate Engine
//! Structured debate between agents with argument evaluation and synthesis.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-DEBATE-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// A debate session between Cathedral agents.
pub struct DebateSession {
    pub id: String,
    pub topic: String,
    pub participants: Vec<super::AgentId>,
    pub rounds: Vec<DebateRound>,
    pub status: DebateStatus,
    pub max_rounds: u32,
    pub current_round: u32,
    pub judge: Option<super::AgentId>, // Oracle or Guardian
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateStatus {
    Forming,      // Collecting participants
    Active,       // Debate in progress
    Evaluating,   // Judge reviewing arguments
    Concluded,    // Final verdict reached
    Deadlocked,   // No consensus possible
    Cancelled,    // Emergency stop or timeout
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRound {
    pub round_number: u32,
    pub arguments: Vec<Argument>,
    pub rebuttals: Vec<Rebuttal>,
    pub scores: HashMap<super::AgentId, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub agent_id: super::AgentId,
    pub position: String,       // "for", "against", "neutral"
    pub claim: String,
    pub evidence: Vec<String>,
    pub reasoning_chain: Vec<String>,
    pub confidence: f32,
    pub citations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rebuttal {
    pub agent_id: super::AgentId,
    pub target_argument_idx: usize,
    pub counter_claim: String,
    pub identified_flaws: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateVerdict {
    pub winner: Option<super::AgentId>,
    pub winning_position: String,
    pub confidence: f32,
    pub reasoning: String,
    pub dissenting_opinions: Vec<String>,
    pub policy_implications: Vec<String>,
    pub recommended_action: String,
}

/// Debate engine — manages structured argumentation.
pub struct DebateEngine {
    sessions: HashMap<String, DebateSession>,
    evaluation_criteria: Vec<EvaluationCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCriterion {
    pub name: String,
    pub weight: f32,
    pub description: String,
}

impl DebateEngine {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            evaluation_criteria: vec![
                EvaluationCriterion {
                    name: "logical_soundness".to_string(),
                    weight: 0.30,
                    description: "Validity of reasoning chain".to_string(),
                },
                EvaluationCriterion {
                    name: "evidence_quality".to_string(),
                    weight: 0.25,
                    description: "Strength and relevance of evidence".to_string(),
                },
                EvaluationCriterion {
                    name: "rebuttal_strength".to_string(),
                    weight: 0.20,
                    description: "Effectiveness in addressing counter-arguments".to_string(),
                },
                EvaluationCriterion {
                    name: "cathedral_alignment".to_string(),
                    weight: 0.15,
                    description: "Consistency with Cathedral policy".to_string(),
                },
                EvaluationCriterion {
                    name: "practical_feasibility".to_string(),
                    weight: 0.10,
                    description: "Can the proposal be implemented".to_string(),
                },
            ],
        }
    }

    /// Start a new debate session.
    pub fn start_debate(
        &mut self,
        topic: String,
        participants: Vec<super::AgentId>,
        judge: Option<super::AgentId>,
        max_rounds: u32,
    ) -> Result<String, DebateError> {
        if participants.len() < 2 {
            return Err(DebateError::InsufficientParticipants(participants.len()));
        }

        let debate_id = format!("debate_{}", blake3::hash(topic.as_bytes()).to_hex());

        let session = DebateSession {
            id: debate_id.clone(),
            topic,
            participants,
            rounds: Vec::new(),
            status: DebateStatus::Active,
            max_rounds,
            current_round: 0,
            judge,
        };

        self.sessions.insert(debate_id.clone(), session);
        Ok(debate_id)
    }

    /// Submit an argument in the current round.
    pub fn submit_argument(
        &mut self,
        debate_id: &str,
        agent_id: super::AgentId,
        argument: Argument,
    ) -> Result<(), DebateError> {
        let session = self.sessions.get_mut(debate_id)
            .ok_or(DebateError::SessionNotFound(debate_id.to_string()))?;

        if session.status != DebateStatus::Active {
            return Err(DebateError::DebateNotActive);
        }

        // Get or create current round
        let round_idx = session.current_round as usize;
        if session.rounds.len() <= round_idx {
            session.rounds.push(DebateRound {
                round_number: session.current_round,
                arguments: Vec::new(),
                rebuttals: Vec::new(),
                scores: HashMap::new(),
            });
        }

        let round = &mut session.rounds[round_idx];
        round.arguments.push(argument);

        // Check if all participants have argued
        if round.arguments.len() >= session.participants.len() {
            // Move to rebuttal phase or next round
            if session.current_round < session.max_rounds - 1 {
                session.current_round += 1;
            } else {
                session.status = DebateStatus::Evaluating;
            }
        }

        Ok(())
    }

    /// Submit a rebuttal to an argument.
    pub fn submit_rebuttal(
        &mut self,
        debate_id: &str,
        rebuttal: Rebuttal,
    ) -> Result<(), DebateError> {
        let session = self.sessions.get_mut(debate_id)
            .ok_or(DebateError::SessionNotFound(debate_id.to_string()))?;

        let round_idx = session.current_round.saturating_sub(1) as usize;
        if let Some(round) = session.rounds.get_mut(round_idx) {
            round.rebuttals.push(rebuttal);
        }

        Ok(())
    }

    /// Evaluate debate and produce verdict.
    pub fn evaluate_debate(&mut self, debate_id: &str) -> Result<DebateVerdict, DebateError> {
        let session = self.sessions.get_mut(debate_id)
            .ok_or(DebateError::SessionNotFound(debate_id.to_string()))?;

        if session.status != DebateStatus::Evaluating && session.status != DebateStatus::Active {
            return Err(DebateError::DebateNotReady);
        }

        session.status = DebateStatus::Concluded;

        // Score all arguments across all rounds
        let mut agent_scores: HashMap<super::AgentId, f32> = HashMap::new();

        for round in &session.rounds {
            for argument in &round.arguments {
                let score = self.score_argument(argument, &round.rebuttals);
                *agent_scores.entry(argument.agent_id.clone()).or_insert(0.0) += score;
            }
        }

        // Determine winner
        let (winner_id, max_score) = agent_scores.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or_default();

        let winning_position = session.rounds.iter()
            .flat_map(|r| &r.arguments)
            .find(|a| a.agent_id == winner_id)
            .map(|a| a.position.clone())
            .unwrap_or_default();

        let all_positions: Vec<String> = session.rounds.iter()
            .flat_map(|r| &r.arguments)
            .map(|a| a.position.clone())
            .collect();

        let dissenting: Vec<String> = all_positions.into_iter()
            .filter(|p| p != &winning_position)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let verdict = DebateVerdict {
            winner: Some(winner_id),
            winning_position,
            confidence: (max_score / (self.evaluation_criteria.len() as f32)).min(1.0),
            reasoning: "Evaluated across logical soundness, evidence quality, rebuttal strength, Cathedral alignment, and feasibility.".to_string(),
            dissenting_opinions: dissenting,
            policy_implications: vec!["Review Cathedral policy for consistency".to_string()],
            recommended_action: "Proceed with winning proposal under Guardian oversight".to_string(),
        };

        Ok(verdict)
    }

    /// Get debate status.
    pub fn get_status(&self, debate_id: &str) -> Option<&DebateSession> {
        self.sessions.get(debate_id)
    }

    // --- Private ---

    fn score_argument(&self, argument: &Argument, rebuttals: &[Rebuttal]) -> f32 {
        let mut score = 0.0f32;

        // Logical soundness
        if argument.reasoning_chain.len() >= 3 {
            score += self.evaluation_criteria[0].weight;
        } else if !argument.reasoning_chain.is_empty() {
            score += self.evaluation_criteria[0].weight * 0.5;
        }

        // Evidence quality
        if argument.evidence.len() >= 2 {
            score += self.evaluation_criteria[1].weight;
        } else if !argument.evidence.is_empty() {
            score += self.evaluation_criteria[1].weight * 0.5;
        }

        // Rebuttal strength (inverse — more rebuttals = lower score)
        let rebuttal_count = rebuttals.iter()
            .filter(|r| r.target_argument_idx < 1000) // simplified
            .count() as f32;
        let rebuttal_penalty = (rebuttal_count * 0.1).min(self.evaluation_criteria[2].weight);
        score += self.evaluation_criteria[2].weight - rebuttal_penalty;

        // Cathedral alignment (stub — would check against policy)
        score += self.evaluation_criteria[3].weight * argument.confidence;

        // Feasibility
        score += self.evaluation_criteria[4].weight * argument.confidence;

        score
    }
}

#[derive(Debug, Clone)]
pub enum DebateError {
    InsufficientParticipants(usize),
    SessionNotFound(String),
    DebateNotActive,
    DebateNotReady,
    AgentNotParticipant(super::AgentId),
    MaxRoundsReached,
}