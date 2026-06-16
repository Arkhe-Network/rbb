pub trait RewardModel {
    fn as_any(&self) -> &dyn std::any::Any;
}
