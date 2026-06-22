// src/swarm.rs — Orquestração de enxame
use libp2p::*;
use libp2p::gossipsub::{Gossipsub, GossipsubEvent};

pub struct SwarmOrchestrator {
    peer_id: PeerId,
    swarm: Swarm<Gossipsub>,
    agents: HashSet<AgentId>,
}

impl SwarmOrchestrator {
    pub fn new() -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let transport = build_transport(keypair.clone());
        let behaviour = Gossipsub::new(
            gossipsub::ConfigBuilder::default().build().unwrap()
        );
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id).build();
        Self { peer_id, swarm, agents: HashSet::new() }
    }

    pub async fn broadcast(&mut self, message: &[u8]) -> Result<()> {
        let topic = Topic::new("cathedral-swarm");
        self.swarm.behaviour_mut().publish(topic, message)?;
        Ok(())
    }

    pub fn add_agent(&mut self, agent_id: AgentId) {
        self.agents.insert(agent_id);
    }

    pub fn consensus_vote(&self, proposal: &[u8], quorum: usize) -> VoteResult {
        // Implementação de PBFT simplificada
        let votes = self.agents.len();
        let approved = votes >= quorum;
        VoteResult { approved, votes, quorum }
    }
}
