use crate::core::analysis::{AlgorithmClass, BlockKind, CriticalBlock};
use crate::core::state::{
    entropy_von_neumann_2x2, initial_density_from_information_density, ComplexMatrix2, Density2,
};
use num_complex::Complex64;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LindbladRates {
    pub dephasing_gamma: f64,
    pub amplitude_gamma: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveHamiltonianTrace {
    pub estimated_cost: f64,
    pub information_density: f64,
    pub kind_factor: f64,
    pub domain_factor: f64,
    pub resulting_energy: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveCouplingTrace {
    pub information_density: f64,
    pub base_coupling: f64,
    pub density_contribution: f64,
    pub resulting_coupling: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveLindbladTrace {
    pub quantum_noise: f64,
    pub thermal_factor: f64,
    pub relativistic_beta: f64,
    pub logical_qubits: u32,
    pub information_density: f64,
    pub dephasing_gamma: f64,
    pub amplitude_gamma: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectivePhysicalBlock {
    pub name: String,
    pub kind: BlockKind,
    pub effective_logical_qubits: u32,
    pub effective_hamiltonian_energy: f64,
    pub coupling_strength: f64,
    pub lindblad_rates: LindbladRates,
    pub effective_relativistic_factor: f64,
    pub computational_energy_cost: f64,
    pub information_density: f64,
    pub coherence_penalty: f64,
    pub density_state: Density2,
    pub density_entropy: f64,
    pub hamiltonian_trace: EffectiveHamiltonianTrace,
    pub coupling_trace: EffectiveCouplingTrace,
    pub lindblad_trace: EffectiveLindbladTrace,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectivePhysicalModel {
    pub blocks: Vec<EffectivePhysicalBlock>,
    pub decoherence_rate: f64,
    pub effective_runtime_dilation: f64,
    pub von_neumann_entropy: f64,
    pub global_constraint_penalty: f64,
    pub recommended_qubit_budget: u32,
}

pub fn build_effective_physical_model(
    blocks: &[CriticalBlock],
    algorithm_class: AlgorithmClass,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
) -> EffectivePhysicalModel {
    let effective_relativistic_factor = compute_effective_relativistic_factor(relativistic_beta);
    let thermal_factor = 1.0 + target_temp_kelvin / 300.0;
    let domain_factor = match algorithm_class {
        AlgorithmClass::Crypto => 1.20,
        AlgorithmClass::Numerical => 1.00,
        AlgorithmClass::Ml => 1.10,
        AlgorithmClass::General => 0.85,
    };

    let mut physical_blocks = Vec::new();

    for block in blocks {
        let kind_factor = match block.kind {
            BlockKind::Function => 0.90,
            BlockKind::Loop => 1.15,
            BlockKind::CryptoPrimitive => 1.35,
            BlockKind::NumericalKernel => 1.20,
            BlockKind::MlKernel => 1.25,
        };

        let effective_hamiltonian_energy =
            block.estimated_cost * block.information_density * kind_factor * domain_factor;

        let base_coupling = 0.15;
        let density_contribution = block.information_density * 0.35;
        let coupling_strength = base_coupling + density_contribution;

        let dephasing_gamma =
            quantum_noise * thermal_factor * (0.45 + block.information_density * 0.60);

        let amplitude_gamma = quantum_noise
            * (1.0 + relativistic_beta)
            * (0.35 + (block.estimated_logical_qubits as f64 / 64.0));

        let hamiltonian_trace = EffectiveHamiltonianTrace {
            estimated_cost: block.estimated_cost,
            information_density: block.information_density,
            kind_factor,
            domain_factor,
            resulting_energy: effective_hamiltonian_energy,
        };

        let coupling_trace = EffectiveCouplingTrace {
            information_density: block.information_density,
            base_coupling,
            density_contribution,
            resulting_coupling: coupling_strength,
        };

        let lindblad_trace = EffectiveLindbladTrace {
            quantum_noise,
            thermal_factor,
            relativistic_beta,
            logical_qubits: block.estimated_logical_qubits,
            information_density: block.information_density,
            dephasing_gamma,
            amplitude_gamma,
        };

        let h = effective_hamiltonian_matrix(effective_hamiltonian_energy, coupling_strength);
        let l_phi = lindblad_dephasing(dephasing_gamma);
        let l_amp = lindblad_amplitude_damping(amplitude_gamma);

        let rho0 = initial_density_from_information_density(block.information_density);
        let rho1 = lindblad_density_step(&rho0, &h, &[l_phi, l_amp], 0.05);

        let density_entropy = entropy_von_neumann_2x2(&rho1);

        let computational_energy_cost = effective_hamiltonian_energy
            * effective_relativistic_factor
            * (1.0 + thermal_factor * 0.10);

        let coherence_penalty =
            (dephasing_gamma + amplitude_gamma) * block.information_density * kind_factor;

        physical_blocks.push(EffectivePhysicalBlock {
            name: block.name.clone(),
            kind: block.kind,
            effective_logical_qubits: block.estimated_logical_qubits,
            effective_hamiltonian_energy,
            coupling_strength,
            lindblad_rates: LindbladRates {
                dephasing_gamma,
                amplitude_gamma,
            },
            effective_relativistic_factor,
            computational_energy_cost,
            information_density: block.information_density,
            coherence_penalty,
            density_state: rho1.to_density2(),
            density_entropy,
            hamiltonian_trace,
            coupling_trace,
            lindblad_trace,
        });
    }

    let decoherence_rate = if physical_blocks.is_empty() {
        quantum_noise
    } else {
        physical_blocks
            .iter()
            .map(|b| b.lindblad_rates.dephasing_gamma + b.lindblad_rates.amplitude_gamma)
            .sum::<f64>()
            / physical_blocks.len() as f64
    };

    let total_energy: f64 = physical_blocks
        .iter()
        .map(|b| b.computational_energy_cost)
        .sum();
    let total_penalty: f64 = physical_blocks.iter().map(|b| b.coherence_penalty).sum();

    let global_constraint_penalty =
        (0.015 * total_energy + 1.25 * total_penalty + 2.0 * decoherence_rate).max(0.0);

    let effective_runtime_dilation =
        effective_relativistic_factor * (1.0 + total_energy / 200.0 + target_temp_kelvin / 4000.0);

    let von_neumann_entropy = if physical_blocks.is_empty() {
        0.0
    } else {
        physical_blocks
            .iter()
            .map(|b| b.density_entropy)
            .sum::<f64>()
            / physical_blocks.len() as f64
    };

    let recommended_qubit_budget = compute_recommended_qubit_budget(
        &physical_blocks,
        decoherence_rate,
        global_constraint_penalty,
    );

    EffectivePhysicalModel {
        blocks: physical_blocks,
        decoherence_rate,
        effective_runtime_dilation,
        von_neumann_entropy,
        global_constraint_penalty,
        recommended_qubit_budget,
    }
}

pub fn compute_effective_relativistic_factor(beta: f64) -> f64 {
    let clamped = beta.clamp(0.0, 0.999_999);
    1.0 / (1.0 - clamped * clamped).sqrt()
}

fn effective_hamiltonian_matrix(energy: f64, coupling: f64) -> ComplexMatrix2 {
    ComplexMatrix2 {
        a00: Complex64::new(energy, 0.0),
        a01: Complex64::new(coupling, 0.0),
        a10: Complex64::new(coupling, 0.0),
        a11: Complex64::new(-energy, 0.0),
    }
}

fn lindblad_dephasing(gamma: f64) -> ComplexMatrix2 {
    let s = gamma.max(0.0).sqrt();
    ComplexMatrix2 {
        a00: Complex64::new(s, 0.0),
        a01: Complex64::new(0.0, 0.0),
        a10: Complex64::new(0.0, 0.0),
        a11: Complex64::new(-s, 0.0),
    }
}

fn lindblad_amplitude_damping(gamma: f64) -> ComplexMatrix2 {
    let s = gamma.max(0.0).sqrt();
    ComplexMatrix2 {
        a00: Complex64::new(0.0, 0.0),
        a01: Complex64::new(s, 0.0),
        a10: Complex64::new(0.0, 0.0),
        a11: Complex64::new(0.0, 0.0),
    }
}

pub fn lindblad_density_step(
    rho: &ComplexMatrix2,
    h: &ComplexMatrix2,
    lindblad_ops: &[ComplexMatrix2],
    dt: f64,
) -> ComplexMatrix2 {
    let i = Complex64::new(0.0, 1.0);
    let unitary_term = h.commutator(rho).mul_scalar(-i);

    let mut dissipative = ComplexMatrix2::zero();

    for l in lindblad_ops {
        let ld = l.dagger();
        let l_rho_ld = l.mul(rho).mul(&ld);
        let ld_l = ld.mul(l);
        let anti = ld_l
            .anticommutator(rho)
            .mul_scalar(Complex64::new(0.5, 0.0));
        dissipative = dissipative.add(&l_rho_ld.sub(&anti));
    }

    rho.add(
        &unitary_term
            .add(&dissipative)
            .mul_scalar(Complex64::new(dt, 0.0)),
    )
    .stabilize()
}

fn compute_recommended_qubit_budget(
    blocks: &[EffectivePhysicalBlock],
    decoherence_rate: f64,
    global_constraint_penalty: f64,
) -> u32 {
    let base_qubits: u32 = blocks.iter().map(|b| b.effective_logical_qubits).sum();
    let safety_margin = 1.0 + decoherence_rate * 0.8 + global_constraint_penalty * 0.03;
    ((base_qubits as f64) * safety_margin).ceil() as u32
}
