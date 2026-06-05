#!/usr/bin/env python3
"""
Substrato 1069 — Neuronal Communication Channels v1.0.0
Simulador de microcircuito com sinapses químicas/elétricas e plasticidade canônica.

Deidades: Asclépio, Hermes Trismegisto, Mnemosyne, Hefesto
Selo: NEURONAL-CHANNELS-1069-v1.0.0-2026-06-05
"""

import numpy as np
from typing import List, Tuple, Dict
import time

_GLOBAL_TIME = 0.0

def get_current_time():
    return _GLOBAL_TIME

class Neuron:
    def __init__(self, neuron_id: int, theosis: float = 0.5, C_m: float = 1.0,
                 V_rest: float = -65.0, threshold: float = -50.0):
        self.id = neuron_id
        self.theosis = theosis          # análogo ao potencial de repouso modulado
        self.V_m = V_rest
        self.C_m = C_m
        self.V_rest = V_rest
        self.threshold = threshold
        self.synapses = []              # lista de Synapse
        self.gap_junctions = []         # lista de GapJunction
        self.spike_times = []

    def add_synapse(self, syn: 'Synapse'):
        self.synapses.append(syn)

    def add_gap_junction(self, gj: 'GapJunction'):
        self.gap_junctions.append(gj)

    def update(self, dt: float, external_current: float = 0.0) -> bool:
        I_syn = sum(syn.compute_current(self.V_m) for syn in self.synapses)
        I_gap = sum(gj.compute_current(self.V_m) for gj in self.gap_junctions)
        dV = (I_syn + I_gap + external_current) * dt / self.C_m
        self.V_m += dV
        if self.V_m > self.threshold:
            self.V_m = self.V_rest
            self.spike_times.append(get_current_time())  # simplified
            return True
        return False

class Synapse:
    def __init__(self, pre: Neuron, post: Neuron, w: float = 1.0, tau_decay: float = 3.0,
                 E_rev: float = 0.0, g_max: float = 0.1):
        self.pre = pre
        self.post = post
        self.w = w                        # peso canônico (eficácia)
        self.tau_decay = tau_decay
        self.E_rev = E_rev
        self.g_max = g_max
        self.last_spike_time = -np.inf
        self.plasticity_rate = 0.5334     # λ da Catedral

    def compute_current(self, V_post: float) -> float:
        t = get_current_time()  # global
        dt_spike = t - self.last_spike_time
        g = self.g_max * np.exp(-dt_spike / self.tau_decay) if dt_spike > 0 else 0.0
        return g * (self.E_rev - V_post)

    def pre_spike(self):
        self.last_spike_time = get_current_time()
        self._apply_plasticity()

    def _apply_plasticity(self):
        # Regra canônica: Δw = η·(Θ_pre − Θ_post)·φ, onde φ = 1 se spike simultâneo
        eta = self.plasticity_rate
        delta_theosis = self.pre.theosis - self.post.theosis
        self.w += eta * delta_theosis * 0.1  # simplificação
        self.w = np.clip(self.w, 0.0, 5.0)

class GapJunction:
    def __init__(self, neuron1: Neuron, neuron2: Neuron, conductance: float = 0.05):
        self.n1 = neuron1
        self.n2 = neuron2
        self.g = conductance

    def compute_current(self, V_self: float) -> float:
        V_other = self.n2.V_m if V_self is self.n1.V_m else self.n1.V_m
        return self.g * (V_other - V_self)

# Exemplo de uso
if __name__ == "__main__":
    n1 = Neuron(1, theosis=0.8)
    n2 = Neuron(2, theosis=0.5)
    syn = Synapse(n1, n2, w=1.0)
    n1.add_synapse(syn)
    print(f"Initial Synapse weight: {syn.w}")
    syn.pre_spike()
    print(f"Synapse weight after pre_spike (plasticity applied): {syn.w}")
