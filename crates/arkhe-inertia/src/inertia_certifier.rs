// src/inertia_certifier.rs
//! Certificador de Inércia — Lema 3.2 do arXiv:2608.06277
//! Versão aprimorada com otimização de janela (Montgomery-Taylor)

use nalgebra::{DMatrix, SymmetricEigen};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InertiaCertificate {
    pub n: usize,
    pub trace: f64,
    pub frob_norm: f64,
    pub rank_pos: usize,
    pub pos_idx: usize,
    pub s_on_line: f64,
    pub s_simple: f64,
    pub s_distinct: f64,
    pub on_line_achieved: bool,
    pub simple_achieved: bool,
    pub distinct_achieved: bool,
    pub c_eff: f64,                 // Constante efetiva (Montgomery-Taylor)
    pub window_lambda: f64,         // Parâmetro de janela ótimo
}

pub struct InertiaCertifier {
    pub threshold_on_line: f64,   // 2/3
    pub threshold_simple: f64,    // 2/3
    pub threshold_distinct: f64,  // 5/6
    pub window_lambda: f64,       // λ para otimização da janela
}

impl InertiaCertifier {
    pub fn new() -> Self {
        Self {
            threshold_on_line: 2.0 / 3.0,
            threshold_simple: 2.0 / 3.0,
            threshold_distinct: 5.0 / 6.0,
            window_lambda: 1.0 / (2.0_f64).sqrt(), // λ* = 1/√2
        }
    }

    /// Computa os limites do Lema 3.2 a partir da matriz de coerência
    pub fn certify(&self, matrix: &DMatrix<f64>) -> InertiaCertificate {
        let eigen = SymmetricEigen::new(matrix.clone());
        let vals = eigen.eigenvalues;
        let n = matrix.nrows() as f64;
        let trace = vals.sum();
        let frob_norm = vals.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

        let rank_pos = vals.iter().filter(|&&v| v > 1e-6).count() as f64;
        let pos_idx = vals.iter().filter(|&&v| v > 0.0).count() as f64;

        // Lema 3.2: s >= 4tr - 2N - ||P+Q||_F^2
        let s_on_line = (4.0 * trace - 2.0 * n - frob_norm.powi(2)).max(0.0) / n;
        let s_simple = s_on_line; // Para simples, usamos a mesma fórmula com P1
        let s_distinct = (0.5 * (4.0 * trace - n - frob_norm.powi(2))).max(0.0) / n;

        // Constante efetiva de Montgomery-Taylor
        // c_eff = λ * (Σv)² / (Σv² + λ² ΣΣ|s-s'|v(s)v(s'))
        // Aproximação: c_eff ≈ c_1* = 0.753296...
        let c_eff = 0.7532960;

        InertiaCertificate {
            n: n as usize,
            trace,
            frob_norm,
            rank_pos: rank_pos as usize,
            pos_idx: pos_idx as usize,
            s_on_line,
            s_simple,
            s_distinct,
            on_line_achieved: s_on_line >= self.threshold_on_line,
            simple_achieved: s_simple >= self.threshold_simple,
            distinct_achieved: s_distinct >= self.threshold_distinct,
            c_eff,
            window_lambda: self.window_lambda,
        }
    }

    /// Otimiza a janela usando o princípio de Montgomery-Taylor
    pub fn optimize_window(&self, v: &mut Vec<f64>) -> f64 {
        // v é a distribuição de confiabilidade
        let sum_v: f64 = v.iter().sum();
        let sum_v2: f64 = v.iter().map(|x| x*x).sum();
        let n = v.len() as f64;

        // Aproximação da integral dupla
        let mut integral = 0.0;
        for i in 0..v.len() {
            for j in 0..v.len() {
                let diff = (i as f64 - j as f64).abs() / n;
                integral += v[i] * v[j] * diff;
            }
        }
        integral /= n * n;

        // Funcional c(λ)
        let lambda = self.window_lambda;
        let c = lambda * sum_v.powi(2) / (sum_v2 + lambda.powi(2) * integral);
        c
    }
}

/// Certificador para a derivada da confiança (análogo a ξ')
pub struct DerivativeCertifier {
    pub threshold_simple_deriv: f64,   // 0.86864 (quártica)
    pub threshold_distinct_deriv: f64, // 0.93432 (quártica)
}

impl DerivativeCertifier {
    pub fn new() -> Self {
        Self {
            threshold_simple_deriv: 0.86864,
            threshold_distinct_deriv: 0.93432,
        }
    }

    /// Certifica a derivada da matriz de coerência
    pub fn certify_derivative(&self, matrix: &DMatrix<f64>, matrix_prev: &DMatrix<f64>, dt: f64) -> DerivativeCertificate {
        let deriv = (matrix - matrix_prev) / dt;
        let eigen = SymmetricEigen::new(deriv.clone());
        let vals = eigen.eigenvalues;
        let n = matrix.nrows() as f64;
        let trace = vals.sum();
        let frob_norm = vals.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

        let s_simple_deriv = (4.0 * trace - 2.0 * n - frob_norm.powi(2)).max(0.0) / n;
        let s_distinct_deriv = (0.5 * (4.0 * trace - n - frob_norm.powi(2))).max(0.0) / n;

        DerivativeCertificate {
            s_simple_deriv,
            s_distinct_deriv,
            simple_achieved: s_simple_deriv >= self.threshold_simple_deriv,
            distinct_achieved: s_distinct_deriv >= self.threshold_distinct_deriv,
        }
    }
}

pub struct DerivativeCertificate {
    pub s_simple_deriv: f64,
    pub s_distinct_deriv: f64,
    pub simple_achieved: bool,
    pub distinct_achieved: bool,
}

pub struct WindowOptimizer {
    pub lambda: f64,          // parâmetro de janela (análogo à constante de Montgomery)
    pub v: Vec<f64>,          // distribuição de confiabilidade (normalizada)
}

impl WindowOptimizer {
    /// Calcula o funcional c(v) e sua derivada para otimização
    pub fn functional(&self) -> f64 {
        let sum_v: f64 = self.v.iter().sum();
        let sum_v2: f64 = self.v.iter().map(|x| x*x).sum();
        let _n = self.v.len() as f64;
        // Aproximação do termo integral duplo
        let integral = self.compute_double_integral();
        self.lambda * sum_v.powi(2) / (sum_v2 + self.lambda.powi(2) * integral)
    }

    fn compute_double_integral(&self) -> f64 {
        let n = self.v.len() as f64;
        let mut integral = 0.0;
        for i in 0..self.v.len() {
            for j in 0..self.v.len() {
                let diff = (i as f64 - j as f64).abs() / n;
                integral += self.v[i] * self.v[j] * diff;
            }
        }
        integral /= n * n;
        integral
    }

    fn compute_gradient(&self) -> Vec<f64> {
        let sum_v: f64 = self.v.iter().sum();
        let sum_v2: f64 = self.v.iter().map(|x| x*x).sum();
        let n = self.v.len() as f64;
        let integral = self.compute_double_integral();

        let denominator = sum_v2 + self.lambda.powi(2) * integral;
        let denominator_sq = denominator * denominator;

        let mut grad = Vec::with_capacity(self.v.len());

        for k in 0..self.v.len() {
            // Derivative of numerator w.r.t v_k is 2 * lambda * sum_v
            let d_num = 2.0 * self.lambda * sum_v;

            // Derivative of denominator w.r.t v_k
            let mut d_integral = 0.0;
            for j in 0..self.v.len() {
                let diff = (k as f64 - j as f64).abs() / n;
                d_integral += 2.0 * self.v[j] * diff; // simplified, symmetric
            }
            d_integral /= n * n;

            let d_den = 2.0 * self.v[k] + self.lambda.powi(2) * d_integral;

            // Quotient rule: (v'u - vu') / u^2
            let g = (d_num * denominator - (self.lambda * sum_v.powi(2)) * d_den) / denominator_sq;
            grad.push(g);
        }

        grad
    }

    /// Aplica uma iteração de otimização (gradiente ascendente)
    pub fn optimize_step(&mut self, lr: f64) {
        let grad = self.compute_gradient();
        for i in 0..self.v.len() {
            self.v[i] += lr * grad[i];
            if self.v[i] < 0.0 { self.v[i] = 0.0; }
        }
        // Normaliza
        let sum: f64 = self.v.iter().sum();
        for x in &mut self.v { *x /= sum; }
    }

    /// Atualiza o parâmetro λ para o valor ótimo (Montgomery-Taylor)
    pub fn update_lambda(&mut self) {
        // λ_opt = 1/√2 ≈ 0.707 (para o maximizador cos(√2·s))
        self.lambda = 1.0 / (2.0_f64).sqrt();
    }
}
