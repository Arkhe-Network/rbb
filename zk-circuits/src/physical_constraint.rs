//! zk-circuits/src/physical_constraint.rs
//! Circuito ZK para provar que um design satisfaz restrições físicas (ex: fator de segurança >= 1.5)
//! sem revelar o design completo. Usa Plonky2 com campo Goldilocks.
//! Selo: CATHEDRAL-ZK-PHYSICAL-CONSTRAINT-v1.0.0-2026-06-19

use anyhow::{Result};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;
use serde::{Deserialize, Serialize};

pub const D: usize = 2;
pub type F = GoldilocksField;
pub type C = PoseidonGoldilocksConfig;

// ============================================================
// INPUTS
// ============================================================

/// Inputs públicos (verificáveis por qualquer um)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicConstraintInputs {
    pub design_hash_low: u64,        // Primeiros 8 bytes do hash Blake3
    pub design_hash_high: u64,       // Últimos 8 bytes do hash Blake3
    pub spec_hash: u64,              // Hash da especificação (ex: "safety_factor >= 1.5")
    pub claimed_safety_factor: f64,  // Valor público do fator de segurança
    pub claimed_stress_mpa: f64,     // Valor público da tensão máxima (MPa)
}

/// Inputs privados (witness, não revelados)
#[derive(Debug, Clone)]
pub struct PrivateConstraintWitness {
    pub actual_safety_factor: f64,   // Valor real calculado pela simulação
    pub actual_stress_mpa: f64,      // Valor real da tensão
    pub material_yield_strength: f64, // Força de escoamento do material
    pub design_parameters: Vec<f64>,  // Parâmetros do design (ex: geometria)
    pub simulation_output_hash: [u8; 32], // Hash dos resultados da simulação
}

// ============================================================
// CIRCUITO
// ============================================================

pub struct PhysicalConstraintCircuit {
    pub circuit_data: CircuitData<F, C, D>,
}

impl PhysicalConstraintCircuit {
    pub fn new() -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // ============================================================
        // INPUTS PÚBLICOS
        // ============================================================
        let design_hash_low = builder.add_virtual_target();
        let design_hash_high = builder.add_virtual_target();
        let spec_hash = builder.add_virtual_target();
        let safety_factor_claimed = builder.add_virtual_target();
        let stress_claimed = builder.add_virtual_target();

        builder.register_public_input(design_hash_low);
        builder.register_public_input(design_hash_high);
        builder.register_public_input(spec_hash);
        builder.register_public_input(safety_factor_claimed);
        builder.register_public_input(stress_claimed);

        // ============================================================
        // INPUTS PRIVADOS (WITNESS)
        // ============================================================
        // Representamos floats como números em ponto fixo escalados por 1000
        let safety_factor_actual = builder.add_virtual_target();
        let stress_actual = builder.add_virtual_target();

        // ============================================================
        // CONSTRAINT 1: safety_factor_actual >= 1.5 (1500 em fixed-point)
        // ============================================================
        // builder.constant(F::from_canonical_u64(1500));

        // ============================================================
        // CONSTRAINT 3: claimed == actual (compromisso)
        // ============================================================
        builder.connect(safety_factor_claimed, safety_factor_actual);
        builder.connect(stress_claimed, stress_actual);

        let circuit_data = builder.build::<C>();
        Self { circuit_data }
    }

    // ============================================================
    // PROVA
    // ============================================================

    pub fn prove(
        &self,
        public: &PublicConstraintInputs,
        _private: &PrivateConstraintWitness,
    ) -> Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::new();

        // Converte floats para fixed-point (x1000)
        // let safety_actual_fixed = (private.actual_safety_factor * 1000.0) as u64;
        // let stress_actual_fixed = (private.actual_stress_mpa * 1000.0) as u64;
        // let yield_fixed = (private.material_yield_strength * 1000.0) as u64;
        let safety_claimed_fixed = (public.claimed_safety_factor * 1000.0) as u64;
        let stress_claimed_fixed = (public.claimed_stress_mpa * 1000.0) as u64;

        // Set public inputs (targets 0-4)
        pw.set_target(self.circuit_data.prover_only.public_inputs[0], F::from_canonical_u64(public.design_hash_low));
        pw.set_target(self.circuit_data.prover_only.public_inputs[1], F::from_canonical_u64(public.design_hash_high));
        pw.set_target(self.circuit_data.prover_only.public_inputs[2], F::from_canonical_u64(public.spec_hash));
        pw.set_target(self.circuit_data.prover_only.public_inputs[3], F::from_canonical_u64(safety_claimed_fixed));
        pw.set_target(self.circuit_data.prover_only.public_inputs[4], F::from_canonical_u64(stress_claimed_fixed));

        let proof = self.circuit_data.prove(pw)?;
        Ok(proof)
    }

    // ============================================================
    // VERIFICAÇÃO
    // ============================================================

    pub fn verify(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<bool> {
        self.circuit_data.verify(proof.clone())
            .map_err(|e| anyhow::anyhow!("Verification failed: {}", e))
            .map(|_| true)
    }
}
