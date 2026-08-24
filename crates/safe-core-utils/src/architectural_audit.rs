//! Registro append-only de decisoes arquiteturais com encadeamento criptografico,
//! fork resolution, e persistencia em disco.
//!
//! Design decisions:
//! - Hash chains para forward integrity
//! - Indice bidirecional para travessia O(1)
//! - Fork detection e resolution por timestamp + comprimento
//! - Persistencia append-only com fsync
//! - Determinismo via postcard

use crate::architectural_decision::{ArchitecturalDecision, DecisionHash};
use crate::persistence::AppendOnlyLog;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Estado de quiescencia do sistema (I76.1 / I79.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuiescenceState {
    Active,
    ForkDetected,
    PartitionDetected,
    Quiescent,
}

pub struct ArchitecturalAudit {
    decisions: HashMap<Uuid, ArchitecturalDecision>,
    hash_to_id: HashMap<DecisionHash, Uuid>,
    id_to_hash: HashMap<Uuid, DecisionHash>,
    // Maps a DecisionHash to its successor's UUID
    next_index: HashMap<DecisionHash, Vec<Uuid>>,
    head: Option<DecisionHash>,
    tail: Option<DecisionHash>,
    pub quiescence: QuiescenceState,
    pub quiescence_since: Option<u64>,
    log: Option<AppendOnlyLog>,
}

impl std::fmt::Debug for ArchitecturalAudit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchitecturalAudit")
            .field("decisions", &self.decisions)
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("quiescence", &self.quiescence)
            .finish()
    }
}

impl ArchitecturalAudit {
    pub fn new() -> Self {
        Self {
            decisions: HashMap::new(),
            hash_to_id: HashMap::new(),
            id_to_hash: HashMap::new(),
            next_index: HashMap::new(),
            head: None,
            tail: None,
            quiescence: QuiescenceState::Active,
            quiescence_since: None,
            log: None,
        }
    }

    pub fn with_persistence<P: AsRef<Path>>(path: P) -> Result<Self, AuditError> {
        let mut audit = Self::new();
        let log = AppendOnlyLog::open(path).map_err(|e| AuditError::Persistence(e.to_string()))?;

        for entry in log
            .recover()
            .map_err(|e| AuditError::Persistence(e.to_string()))?
        {
            let decision: ArchitecturalDecision = postcard::from_bytes(&entry)
                .map_err(|e| AuditError::Serialization(e.to_string()))?;
            audit.record(decision)?;
        }

        audit.log = Some(log);
        Ok(audit)
    }

    pub fn record(
        &mut self,
        mut decision: ArchitecturalDecision,
    ) -> Result<DecisionHash, AuditError> {
        if self.quiescence == QuiescenceState::Quiescent {
            return Err(AuditError::Quiescent);
        }

        let is_external = decision.previous_decision_hash.is_some();
        if !is_external {
            decision.previous_decision_hash = self.head.clone();
        }

        // Zero out the hash before computing the new one to ensure determinism across serialization
        // otherwise when recovered from persistence the struct will include its own hash
        decision.decision_hash = DecisionHash([0u8; 32]);

        let serialized = postcard::to_allocvec(&decision)
            .map_err(|e| AuditError::Serialization(e.to_string()))?;
        let hash = DecisionHash(*blake3::hash(&serialized).as_bytes());
        decision.decision_hash = hash.clone();
        let id = decision.id;

        if let Some(prev_hash) = &decision.previous_decision_hash {
            self.next_index
                .entry(prev_hash.clone())
                .or_default()
                .push(id);
        }

        self.hash_to_id.insert(hash.clone(), id);
        self.id_to_hash.insert(id, hash.clone());
        self.decisions.insert(id, decision.clone());

        if self.tail.is_none() {
            self.tail = Some(hash.clone());
        }

        // If it extends the current head, or there is no head, advance the head
        if decision.previous_decision_hash == self.head || self.head.is_none() {
            self.head = Some(hash.clone());
        }

        if let Some(ref mut log) = self.log {
            let entry = postcard::to_allocvec(self.decisions.get(&id).unwrap())
                .map_err(|e| AuditError::Serialization(e.to_string()))?;
            log.append(&entry)
                .map_err(|e| AuditError::Persistence(e.to_string()))?;
        }

        self.check_and_handle_forks()?;

        Ok(hash)
    }

    pub fn trail_backwards(&self) -> Vec<&ArchitecturalDecision> {
        let mut trail = Vec::new();
        let mut current_hash = self.head.clone();
        while let Some(hash) = current_hash {
            if let Some(id) = self.hash_to_id.get(&hash) {
                if let Some(d) = self.decisions.get(id) {
                    trail.push(d);
                    current_hash = d.previous_decision_hash.clone();
                    continue;
                }
            }
            break;
        }
        trail
    }

    pub fn trail_forwards(&self) -> Vec<&ArchitecturalDecision> {
        let mut trail = self.trail_backwards();
        trail.reverse();
        trail
    }

    pub fn trail_chronological(&self) -> Vec<&ArchitecturalDecision> {
        self.trail_forwards()
    }

    pub fn trail_current(&self) -> Vec<&ArchitecturalDecision> {
        self.decisions
            .values()
            .filter(|d| {
                matches!(
                    d.status,
                    crate::architectural_decision::DecisionStatus::Accepted
                )
            })
            .collect()
    }

    pub fn detect_forks(&self) -> Vec<Vec<&ArchitecturalDecision>> {
        let mut heads: Vec<DecisionHash> = Vec::new();
        for hash in self.hash_to_id.keys() {
            if !self.next_index.contains_key(hash) {
                heads.push(hash.clone());
            }
        }
        if heads.len() <= 1 {
            return Vec::new();
        }

        let mut forks = Vec::new();
        for head in heads {
            let mut chain = Vec::new();
            let mut current = Some(head);
            while let Some(hash) = current {
                if let Some(id) = self.hash_to_id.get(&hash) {
                    if let Some(decision) = self.decisions.get(id) {
                        chain.push(decision);
                        current = decision.previous_decision_hash.clone();
                        continue;
                    }
                }
                break;
            }
            chain.reverse();
            forks.push(chain);
        }
        forks
    }

    fn check_and_handle_forks(&mut self) -> Result<(), AuditError> {
        let forks = self.detect_forks();
        if forks.len() > 1 {
            self.quiescence = QuiescenceState::ForkDetected;
            self.quiescence_since = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
        Ok(())
    }

    pub fn resolve_fork(&mut self) -> Result<Vec<ArchitecturalDecision>, AuditError> {
        let forks = self.detect_forks();
        if forks.is_empty() {
            self.quiescence = QuiescenceState::Active;
            self.quiescence_since = None;
            return Ok(self.trail_backwards().into_iter().cloned().collect());
        }
        if forks.len() == 1 {
            let authoritative = forks
                .into_iter()
                .next()
                .unwrap()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            self.quiescence = QuiescenceState::Active;
            self.quiescence_since = None;
            return Ok(authoritative);
        }

        if let Some(since) = self.quiescence_since {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now - since > 30 {
                return Err(AuditError::ForkTimeout);
            }
        }

        let authoritative = forks
            .iter()
            .max_by_key(|chain| {
                let head_ts = chain.last().map(|d| d.timestamp).unwrap_or(0);
                let len = chain.len() as u64;
                (head_ts, len)
            })
            .ok_or(AuditError::ForkResolutionFailed)?;

        let head_hash = authoritative.last().map(|d| d.decision_hash.clone());
        let result = authoritative.iter().map(|&d| d.clone()).collect();

        self.quiescence = QuiescenceState::Active;
        self.quiescence_since = None;
        self.head = head_hash;

        Ok(result)
    }

    pub fn verify_chain(&self) -> Result<(), AuditError> {
        for (id, decision) in &self.decisions {
            if let Some(prev_hash) = &decision.previous_decision_hash {
                if !self.hash_to_id.contains_key(prev_hash) {
                    return Err(AuditError::BrokenChain {
                        decision_id: *id,
                        missing_hash: prev_hash.0,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn successor(&self, hash: DecisionHash) -> Option<&ArchitecturalDecision> {
        let next_ids = self.next_index.get(&hash)?;
        self.decisions.get(next_ids.first()?)
    }

    pub fn predecessor(&self, hash: DecisionHash) -> Option<&ArchitecturalDecision> {
        let id = self.hash_to_id.get(&hash)?;
        let decision = self.decisions.get(id)?;
        let prev_hash = decision.previous_decision_hash.clone()?;
        let prev_id = self.hash_to_id.get(&prev_hash)?;
        self.decisions.get(prev_id)
    }

    pub fn contains(&self, id: &Uuid) -> bool {
        self.decisions.contains_key(id)
    }
    pub fn len(&self) -> usize {
        self.decisions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }
}

impl Default for ArchitecturalAudit {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("falha na serializacao: {0}")]
    Serialization(String),
    #[error(
        "cadeia quebrada na decisao {decision_id}: hash anterior {missing_hash:?} nao encontrado"
    )]
    BrokenChain {
        decision_id: Uuid,
        missing_hash: [u8; 32],
    },
    #[error("ponteiro 'next' inconsistente: decisao {decision_id} aponta para {next_id} com previous_hash incompativel")]
    InconsistentNextPointer { decision_id: Uuid, next_id: Uuid },
    #[error("falha de persistencia: {0}")]
    Persistence(String),
    #[error("sistema em quiescencia — nenhuma decisao pode ser registrada ate resolucao")]
    Quiescent,
    #[error("fork nao resolvido em 30s — intervencao manual necessaria")]
    ForkTimeout,
    #[error("falha na resolucao de fork")]
    ForkResolutionFailed,
}
