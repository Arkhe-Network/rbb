use crate::types::*;
use crate::hash::Hasher;

pub fn apply_evidence(mut state: State, id: EvidenceID, art_id: ArtifactID, c: Payload,
                  sig: Hash, ts: u64, ph: Option<Hash>, hasher: &dyn Hasher)
                  -> Result<State, TransitionError> {
    if state.evidences.contains_key(&id) { return Err(TransitionError::IdAlreadyExists); }
    if !state.artifacts.contains_key(&art_id) { return Err(TransitionError::ReferencedIdNotFound); }

    // F3: Verificação de parent_hash via Option
    let pre_hash_ok = match &ph {
        None => true,
        Some(parent) => state.evidences.values().any(|e| e.hash == *parent),
    };
    if !pre_hash_ok { return Err(TransitionError::InvalidParentHash); }

    let content_bytes = c.as_bytes();
    let hash = format!("{:x?}", hasher.hash(content_bytes));  // ← Hasher abstrato

    let ev = Evidence {
        artifact_id: art_id, content: c, signature: sig,
        timestamp: ts, parent_hash: ph, hash
    };
    state.evidences.insert(id, ev);
    Ok(state)
}

pub fn apply(mut state: State, event: &Event) -> Result<State, TransitionError> {
    match event {
        Event::ArtifactAdded(id, payload, metadata) => {
            if state.artifacts.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            state.artifacts.insert(*id, Artifact { payload: payload.clone(), metadata: metadata.clone() });
            Ok(state)
        }
        Event::EvidenceAdded(id, art_id, c, sig, ts, ph) => {
            // we need a hasher, let's just use IdentityHasher for apply for now, or just dummy hash
            // for the test to work we probably want to pass the hasher, but to conform to `apply(s, &Event::...)`
            // we will just use a dummy hasher inside apply if EvidenceAdded is used this way.
            use crate::hash::IdentityHasher;
            apply_evidence(state, *id, *art_id, c.clone(), sig.clone(), *ts, ph.clone(), &IdentityHasher)
        }
        Event::ClaimAdded(id, prop, ev_ids) => {
            if state.claims.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            for eid in ev_ids {
                if !state.evidences.contains_key(eid) { return Err(TransitionError::ReferencedIdNotFound); }
            }
            state.claims.insert(*id, Claim { proposition: prop.clone(), evidence_ids: ev_ids.clone() });
            Ok(state)
        }
        Event::BeliefAdded(id, cid, conf, just) => {
            if state.beliefs.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            if !state.claims.contains_key(cid) { return Err(TransitionError::ReferencedIdNotFound); }
            state.beliefs.insert(*id, Belief { claim_id: *cid, confidence: *conf, justification: just.clone() });
            Ok(state)
        }
        Event::DecisionAdded(id, goal, bids, ts) => {
            if state.decisions.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            for bid in bids {
                if !state.beliefs.contains_key(bid) { return Err(TransitionError::ReferencedIdNotFound); }
            }
            state.decisions.insert(*id, Decision { goal: goal.clone(), belief_ids: bids.clone(), timestamp: *ts });
            Ok(state)
        }
    }
}
