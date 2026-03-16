use crate::core::analysis::{AlgorithmClass, BlockKind, CriticalBlock};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LindbladOperators {
    pub dephasing: f64,
    pub amplitude_damping: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhysicalBlockModel {
    pub name: String,
    pub kind: BlockKind,
    pub effective_logical_qubits: u32,
    pub effective_hamiltonian: f64,
    pub lindblad: LindbladOperators,
    pub relativistic_factor: f64,
    pub computational_energy_cost: f64,
    pub information_density: f64,
    pub coherence_penalty: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectivePhysicalModel {
    pub blocks: Vec<PhysicalBlockModel>,
    pub decoherence_rate: f64,
    pub effective_runtime_dilation: f64,
    pub von_neumann_entropy: f64,
    pub wheeler_dewitt_penalty: f64,
    pub recommended_qubit_budget: u32,
}

pub fn build_effective_physical_model(
    blocks: &[CriticalBlock],
    algorithm_class: AlgorithmClass,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
) -> EffectivePhysicalModel {
    let relativistic_factor = compute_relativistic_factor(relativistic_beta);
    let thermal_factor = 1.0 + target_temp_kelvin / 300.0;
    let domain_factor = match algorithm_class {
        AlgorithmClass::Crypto => 1.20,
        AlgorithmClass::Numerical => 1.00,
        AlgorithmClass::Ml => 1.10,
        AlgorithmClass::General => 0.85,
    };

    let physical_blocks: Vec<PhysicalBlockModel> = blocks
        .iter()
        .map(|block| {
            let kind_factor = match block.kind {
                BlockKind::Function => 0.90,
                BlockKind::Loop => 1.15,
                BlockKind::CryptoPrimitive => 1.35,
                BlockKind::NumericalKernel => 1.20,
                BlockKind::MlKernel => 1.25,
            };

            let effective_hamiltonian =
                block.estimated_cost * block.information_density * kind_factor * domain_factor;

            let dephasing =
                quantum_noise * thermal_factor * (0.45 + block.information_density * 0.60);
            let amplitude_damping = quantum_noise
                * (1.0 + relativistic_beta)
                * (0.35 + (block.estimated_logical_qubits as f64 / 64.0));

            let computational_energy_cost =
                effective_hamiltonian * relativistic_factor * (1.0 + thermal_factor * 0.10);

            let coherence_penalty =
                (dephasing + amplitude_damping) * block.information_density * kind_factor;

            PhysicalBlockModel {
                name: block.name.clone(),
                kind: block.kind,
                effective_logical_qubits: block.estimated_logical_qubits,
                effective_hamiltonian,
                lindblad: LindbladOperators {
                    dephasing,
                    amplitude_damping,
                },
                relativistic_factor,
                computational_energy_cost,
                information_density: block.information_density,
                coherence_penalty,
            }
        })
        .collect();

    let decoherence_rate = if physical_blocks.is_empty() {
        quantum_noise
    } else {
        physical_blocks
            .iter()
            .map(|b| b.lindblad.dephasing + b.lindblad.amplitude_damping)
            .sum::<f64>()
            / physical_blocks.len() as f64
    };

    let total_energy: f64 = physical_blocks
        .iter()
        .map(|b| b.computational_energy_cost)
        .sum();

    let total_penalty: f64 = physical_blocks.iter().map(|b| b.coherence_penalty).sum();

    let wheeler_dewitt_penalty =
        (total_energy * 0.015 + total_penalty * 1.25 + decoherence_rate * 2.0).max(0.0);

    let effective_runtime_dilation =
        relativistic_factor * (1.0 + total_energy / 200.0 + target_temp_kelvin / 4000.0);

    let von_neumann_entropy =
        compute_effective_von_neumann_entropy(&physical_blocks, quantum_noise, thermal_factor);

    let recommended_qubit_budget = compute_recommended_qubit_budget(
        &physical_blocks,
        decoherence_rate,
        wheeler_dewitt_penalty,
    );

    EffectivePhysicalModel {
        blocks: physical_blocks,
        decoherence_rate,
        effective_runtime_dilation,
        von_neumann_entropy,
        wheeler_dewitt_penalty,
        recommended_qubit_budget,
    }
}

fn compute_relativistic_factor(beta: f64) -> f64 {
    let clamped = beta.clamp(0.0, 0.999_999);
    1.0 / (1.0 - clamped * clamped).sqrt()
}

fn compute_effective_von_neumann_entropy(
    blocks: &[PhysicalBlockModel],
    quantum_noise: f64,
    thermal_factor: f64,
) -> f64 {
    if blocks.is_empty() {
        return 0.0;
    }

    let raw_weights: Vec<f64> = blocks
        .iter()
        .map(|b| (b.information_density * b.effective_hamiltonian).max(1e-12))
        .collect();

    let total = raw_weights.iter().sum::<f64>().max(1e-12);

    let mut entropy = 0.0;
    for weight in raw_weights {
        let p = (weight / total).clamp(1e-12, 1.0);
        entropy -= p * p.ln();
    }

    entropy * (1.0 + quantum_noise * 2.0 + (thermal_factor - 1.0) * 0.15)
}

fn compute_recommended_qubit_budget(
    blocks: &[PhysicalBlockModel],
    decoherence_rate: f64,
    wheeler_dewitt_penalty: f64,
) -> u32 {
    let base_qubits: u32 = blocks.iter().map(|b| b.effective_logical_qubits).sum();

    let safety_margin = 1.0 + decoherence_rate * 0.8 + wheeler_dewitt_penalty * 0.03;
    ((base_qubits as f64) * safety_margin).ceil() as u32
}
