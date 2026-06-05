/-
Substrato 1069 — Neuronal Communication Channels
Lema: Plasticidade Hebbiana com Theosis
-/

structure Neuron where
  id : Nat
  theosis : Float  -- métrica canônica
  V_m : Float      -- potencial de membrana

structure Synapse where
  pre : Neuron
  post : Neuron
  w : Float         -- peso sináptico
  deriving Repr

def apply_plasticity (syn : Synapse) (rate : Float := 0.5334) : Synapse :=
  let delta := syn.pre.theosis - syn.post.theosis
  { syn with w := min 5.0 (max 0.0 (syn.w + rate * delta * 0.1)) }

theorem plasticity_increases_when_pre_theosis_higher (syn : Synapse)
    (h : syn.pre.theosis > syn.post.theosis) : (apply_plasticity syn).w > syn.w := by
  simp [apply_plasticity, h]
  linarith
