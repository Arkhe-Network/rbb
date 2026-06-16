//! Cathedral ARKHE v28.3 — Hierarchical Command Structure
//! Chain of command with delegation, escalation, and override protocols.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-HIERARCHY-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Hierarchical levels in Cathedral multi-agent system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommandLevel {
    Strategic,    // Guardian + Oracle — policy decisions
    Tactical,     // Analyst + Coder — plan execution
    Operational,  // Executor — tool execution
    Observational, // Observer — monitoring only
}

/// Command node in the hierarchy tree.
pub struct CommandNode {
    pub agent_id: super::AgentId,
    pub level: CommandLevel,
    pub subordinates: Vec<super::AgentId>,
    pub superior: Option<super::AgentId>,
    pub delegated_authority: Vec<AuthorityDelegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityDelegation {
    pub from: super::AgentId,
    pub to: super::AgentId,
    pub authority_type: DelegatedAuthority,
    pub expires_at: u64, // Unix timestamp
    pub scope: String,   // e.g., "substrate_319.1", "all_tools"
    pub revocable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelegatedAuthority {
    ToolExecution,      // Can execute specific tools
    PolicyModification, // Can modify policy within scope
    EmergencyOverride,  // Can override emergency stops
    CoalitionFormation, // Can form coalitions
    ConsensusInitiation, // Can call for consensus
}

/// Command structure manager.
pub struct HierarchyManager {
    nodes: HashMap<super::AgentId, CommandNode>,
    root: super::AgentId,
    delegations: Vec<AuthorityDelegation>,
}

impl HierarchyManager {
    pub fn new(root_agent: super::AgentId) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(root_agent.clone(), CommandNode {
            agent_id: root_agent.clone(),
            level: CommandLevel::Strategic,
            subordinates: Vec::new(),
            superior: None,
            delegated_authority: Vec::new(),
        });

        Self {
            nodes,
            root: root_agent,
            delegations: Vec::new(),
        }
    }

    /// Add agent to hierarchy under a superior.
    pub fn add_agent(
        &mut self,
        agent_id: super::AgentId,
        level: CommandLevel,
        superior: super::AgentId,
    ) -> Result<(), HierarchyError> {
        // Validate superior exists
        if !self.nodes.contains_key(&superior) {
            return Err(HierarchyError::SuperiorNotFound(superior));
        }

        // Validate level is below superior
        let superior_level = self.nodes.get(&superior).unwrap().level;
        if level >= superior_level {
            return Err(HierarchyError::InvalidLevel {
                agent: agent_id,
                requested: level,
                superior_level,
            });
        }

        // Add node
        self.nodes.insert(agent_id.clone(), CommandNode {
            agent_id: agent_id.clone(),
            level,
            subordinates: Vec::new(),
            superior: Some(superior.clone()),
            delegated_authority: Vec::new(),
        });

        // Register as subordinate
        self.nodes.get_mut(&superior).unwrap().subordinates.push(agent_id);

        Ok(())
    }

    /// Delegate authority from one agent to another.
    pub fn delegate_authority(
        &mut self,
        from: super::AgentId,
        to: super::AgentId,
        authority: DelegatedAuthority,
        scope: String,
        duration_secs: u64,
    ) -> Result<AuthorityDelegation, HierarchyError> {
        // Validate both agents exist
        if !self.nodes.contains_key(&from) {
            return Err(HierarchyError::AgentNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(HierarchyError::AgentNotFound(to));
        }

        // Validate delegator has the authority
        let from_node = self.nodes.get(&from).unwrap();
        let has_authority = match authority {
            DelegatedAuthority::EmergencyOverride => from_node.level == CommandLevel::Strategic,
            DelegatedAuthority::PolicyModification => from_node.level == CommandLevel::Strategic,
            _ => from_node.level <= CommandLevel::Tactical,
        };

        if !has_authority {
            return Err(HierarchyError::InsufficientAuthority { agent: from, authority });
        }

        let delegation = AuthorityDelegation {
            from: from.clone(),
            to: to.clone(),
            authority_type: authority,
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + duration_secs,
            scope,
            revocable: true,
        };

        self.delegations.push(delegation.clone());

        // Add to node
        self.nodes.get_mut(&to).unwrap().delegated_authority.push(delegation.clone());

        Ok(delegation)
    }

    /// Revoke a delegation.
    pub fn revoke_delegation(&mut self, delegation: &AuthorityDelegation) -> Result<(), HierarchyError> {
        self.delegations.retain(|d| d != delegation);

        if let Some(node) = self.nodes.get_mut(&delegation.to) {
            node.delegated_authority.retain(|d| d != delegation);
        }

        Ok(())
    }

    /// Check if agent has authority for an action.
    pub fn has_authority(
        &self,
        agent_id: &super::AgentId,
        authority: DelegatedAuthority,
        scope: &str,
    ) -> bool {
        // Check direct authority
        if let Some(node) = self.nodes.get(agent_id) {
            let has_direct = match authority {
                DelegatedAuthority::EmergencyOverride => node.level == CommandLevel::Strategic,
                DelegatedAuthority::PolicyModification => node.level == CommandLevel::Strategic,
                DelegatedAuthority::ToolExecution => node.level <= CommandLevel::Operational,
                DelegatedAuthority::CoalitionFormation => node.level <= CommandLevel::Tactical,
                DelegatedAuthority::ConsensusInitiation => node.level <= CommandLevel::Tactical,
            };

            if has_direct {
                return true;
            }

            // Check delegated authority
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            node.delegated_authority.iter().any(|d| {
                d.authority_type == authority
                    && d.to == *agent_id
                    && d.expires_at > now
                    && (d.scope == scope || d.scope == "all")
            })
        } else {
            false
        }
    }

    /// Escalate a decision up the chain of command.
    pub fn escalate(&self, agent_id: &super::AgentId) -> Option<super::AgentId> {
        self.nodes.get(agent_id)
            .and_then(|node| node.superior.clone())
    }

    /// Get chain of command from agent to root.
    pub fn chain_of_command(&self, agent_id: &super::AgentId) -> Vec<super::AgentId> {
        let mut chain = Vec::new();
        let mut current = Some(agent_id.clone());

        while let Some(id) = current {
            chain.push(id.clone());
            current = self.nodes.get(&id).and_then(|n| n.superior.clone());
        }

        chain
    }

    /// Get all subordinates (recursive).
    pub fn get_subordinates(&self, agent_id: &super::AgentId) -> Vec<super::AgentId> {
        let mut result = Vec::new();
        if let Some(node) = self.nodes.get(agent_id) {
            for sub in &node.subordinates {
                result.push(sub.clone());
                result.extend(self.get_subordinates(sub));
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub enum HierarchyError {
    SuperiorNotFound(super::AgentId),
    AgentNotFound(super::AgentId),
    InvalidLevel { agent: super::AgentId, requested: CommandLevel, superior_level: CommandLevel },
    InsufficientAuthority { agent: super::AgentId, authority: DelegatedAuthority },
    DelegationExpired,
    CircularReference,
}