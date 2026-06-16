use std::sync::Arc;
use crate::rl::reward_model::RewardModel;
use crate::rl::debate_consensus_reward::DebateConsensusRewardModel;

pub struct AsyncRLOrchestrator {
    pub reward_model: Arc<dyn RewardModel>,
}

impl AsyncRLOrchestrator {
    pub fn new_with_debate(debate_reward_model: Arc<DebateConsensusRewardModel>) -> Self {
        Self {
            reward_model: debate_reward_model,
        }
    }
}
