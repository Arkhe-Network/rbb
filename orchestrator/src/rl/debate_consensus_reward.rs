
use crate::rl::reward_model::RewardModel;

pub struct DebateConsensusRewardModel {}

impl DebateConsensusRewardModel {
    pub fn new() -> Self {
        Self {}
    }
}

impl RewardModel for DebateConsensusRewardModel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
