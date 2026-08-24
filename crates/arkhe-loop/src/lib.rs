pub mod nonology;
pub mod phases;
pub mod state;
pub mod triadic;

pub use nonology::{NonologyTerm, Transcendental};
pub use state::{LoopConfig, LoopPhase, LoopResult, LoopSnapshot, LoopState, LoopStep};
pub use triadic::*;

pub struct AgentLoop {
    config: LoopConfig,
    steps: Vec<state::LoopStep>,
}

impl AgentLoop {
    pub fn new(config: LoopConfig) -> Self {
        Self {
            config,
            steps: Vec::new(),
        }
    }
    pub async fn run_cycle(&mut self, task: &str, _ctx: &serde_json::Value) -> LoopResult {
        self.state().check_limits()?;
        let mut eng = TriadicEngine::new(
            DefaultOntologicDriver,
            DefaultContextDriver::new(vec![]),
            DefaultPromptBuilder,
        );
        let res = eng.execute_step(task, |_p| Ok(format!("Triadic: {}", task)))?;
        for ts in eng.steps() {
            self.steps.push(state::LoopStep {
                phase: LoopPhase::Reasoning,
                iteration: self.steps.len() as u64,
                input: String::new(),
                output: ts.output.clone(),
                duration_ms: ts.duration_ms,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                success: true,
            });
        }
        Ok(res)
    }
    pub fn snapshot(&self) -> LoopSnapshot {
        self.state().snapshot()
    }
    fn state(&self) -> LoopState {
        LoopState::new(self.config.clone())
    }
}
