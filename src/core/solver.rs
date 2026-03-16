use crate::core::physics::EffectivePhysicalModel;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloSummary {
    pub samples: usize,
    pub mean_stress: f64,
    pub stress_variance: f64,
    pub p05_stress: f64,
    pub p50_stress: f64,
    pub p95_stress: f64,
    pub collapse_probability: f64,
    pub computational_singularity_risk: f64,
    pub solver_stability_score: f64,
}

pub fn run_effective_solver(
    physical_model: &EffectivePhysicalModel,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
) -> MonteCarloSummary {
    let samples = 128usize;
    let thermal_factor = 1.0 + target_temp_kelvin / 300.0;

    let base_stress = physical_model.decoherence_rate * 8.0
        + physical_model.von_neumann_entropy * 0.9
        + physical_model.wheeler_dewitt_penalty * 0.12
        + (physical_model.effective_runtime_dilation - 1.0) * 3.5;

    let mut values = Vec::with_capacity(samples);

    for i in 0..samples {
        let q = quantum_perturbation(i, quantum_noise);
        let t = thermal_perturbation(i, thermal_factor);
        let r = relativistic_perturbation(i, relativistic_beta);

        let combined = base_stress * (1.0 + q + t + r);
        values.push(combined.max(0.0));
    }

    renormalize_tail(&mut values);

    let mean_stress = mean(&values);
    let stress_variance = variance(&values, mean_stress);
    let p05_stress = percentile(&values, 0.05);
    let p50_stress = percentile(&values, 0.50);
    let p95_stress = percentile(&values, 0.95);

    let collapse_threshold = (base_stress * 1.18).max(0.45);
    let singularity_threshold = (base_stress * 1.35 + 0.25).max(0.80);

    let collapse_probability = probability_above(&values, collapse_threshold);
    let computational_singularity_risk = probability_above(&values, singularity_threshold);

    let solver_stability_score = (100.0
        - mean_stress * 10.0
        - collapse_probability * 35.0
        - computational_singularity_risk * 45.0
        - stress_variance * 4.0)
        .clamp(0.0, 100.0);

    MonteCarloSummary {
        samples,
        mean_stress,
        stress_variance,
        p05_stress,
        p50_stress,
        p95_stress,
        collapse_probability,
        computational_singularity_risk,
        solver_stability_score,
    }
}

fn quantum_perturbation(index: usize, quantum_noise: f64) -> f64 {
    let x = pseudo_unit(index as u64 + 11);
    ((x - 0.5) * 2.0) * (quantum_noise * 1.8)
}

fn thermal_perturbation(index: usize, thermal_factor: f64) -> f64 {
    let x = pseudo_unit(index as u64 + 97);
    ((x - 0.5) * 2.0) * ((thermal_factor - 1.0) * 0.18)
}

fn relativistic_perturbation(index: usize, relativistic_beta: f64) -> f64 {
    let x = pseudo_unit(index as u64 + 211);
    ((x - 0.5) * 2.0) * (relativistic_beta * 0.22)
}

fn pseudo_unit(seed: u64) -> f64 {
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;

    let top = (x >> 11) as f64;
    let denom = ((1u64 << 53) - 1) as f64;
    (top / denom).clamp(0.0, 1.0)
}

fn renormalize_tail(values: &mut [f64]) {
    if values.is_empty() {
        return;
    }

    values.sort_by(|a, b| a.total_cmp(b));

    let p95 = percentile_sorted(values, 0.95);
    let p99 = percentile_sorted(values, 0.99);
    let upper_cap = p95 + (p99 - p95) * 0.50;

    for value in values.iter_mut() {
        if *value > upper_cap {
            *value = upper_cap + (*value - upper_cap) * 0.15;
        }
    }

    values.sort_by(|a, b| a.total_cmp(b));
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn variance(values: &[f64], mean_value: f64) -> f64 {
    if values.len() < 2 {
        0.0
    } else {
        values
            .iter()
            .map(|v| {
                let d = *v - mean_value;
                d * d
            })
            .sum::<f64>()
            / values.len() as f64
    }
}

fn percentile(values: &[f64], q: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    percentile_sorted(&sorted, q)
}

fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    let q_clamped = q.clamp(0.0, 1.0);
    let idx = ((sorted.len() - 1) as f64 * q_clamped).round() as usize;
    sorted[idx]
}

fn probability_above(values: &[f64], threshold: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let count = values.iter().filter(|v| **v > threshold).count();
    count as f64 / values.len() as f64
}
