#[cfg(feature = "deployment-sim")]
pub mod geometry;
#[cfg(feature = "deployment-sim")]
pub mod simulation;
#[cfg(feature = "deployment-sim")]
pub mod cuda;
#[cfg(feature = "deployment-sim")]
pub mod governance;
#[cfg(feature = "deployment-sim")]
pub mod integration;
pub mod testing;

pub mod rl {
    pub mod async_rl_orchestrator;
    pub mod debate_consensus_reward;
    pub mod ledger_relayer;
    pub mod reward_model;
    pub mod contract_bindings;
}

pub mod mcp;
pub mod attestation;
pub mod identity_attestation;
pub mod voice;

pub mod hashtree;
pub mod okf;
pub mod swarm;
pub mod skill;
pub mod dark_relay;
pub mod cli;
