/-!
  Cathedral Arkhe — Abstract Agent Core
-/

namespace CathedralArkhe.AgentCore

universe u v

/-- Action type. No structure imposed. -/
abbrev Action (A : Type u) := A

/-- Observation type. No structure imposed. -/
abbrev Observation (O : Type v) := O

/-- Latent state type. "Continuous" is an interpretive label;
    the type itself is abstract. -/
abbrev State (S : Type u) := S

/-- A distribution over a type α. -/
class Distribution (α : Type u) (Real : Type u) where
  support : α → Real

/-- A belief state: a distribution over latent states. -/
abbrev Belief (S : Type u) (Real : Type u) [Distribution S Real] := Distribution S Real

/-- A policy: maps each state to a distribution over actions. -/
structure Policy (S A : Type u) (Real : Type u) [Distribution S Real] where
  eval : S → A → Real

/-- A world model: maps (state, action) to a distribution over next states. -/
structure WorldModel (S A : Type u) (Real : Type u) [Distribution S Real] where
  trans : S → A → S → Real

/-- A single experience step. -/
structure Experience (S A : Type u) where
  stateBefore : S
  action : A
  stateAfter : S

structure AgentData (S A : Type u) (Real : Type u) [Distribution S Real] where
  belief : Belief S Real
  policy : Policy S A Real
  worldModel : WorldModel S A Real

def AgentUpdate (S A : Type u) (Real : Type u) [Distribution S Real] :=
  AgentData S A Real → Experience S A → AgentData S A Real

structure Agent (S A : Type u) (Real : Type u) [Distribution S Real] where
  data : AgentData S A Real
  update : AgentUpdate S A Real

end CathedralArkhe.AgentCore
