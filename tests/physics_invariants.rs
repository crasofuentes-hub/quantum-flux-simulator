use flux_sim::core::physics::lindblad_density_step;
use flux_sim::core::state::{
    entropy_von_neumann_2x2, has_only_finite_entries, hermiticity_residual,
    initial_density_from_information_density, min_eigenvalue_2x2, trace_distance_from_one,
    ComplexMatrix2,
};
use num_complex::Complex64;

fn diagonal_hamiltonian(energy: f64) -> ComplexMatrix2 {
    ComplexMatrix2 {
        a00: Complex64::new(energy, 0.0),
        a01: Complex64::new(0.0, 0.0),
        a10: Complex64::new(0.0, 0.0),
        a11: Complex64::new(-energy, 0.0),
    }
}

fn dephasing_lindblad(gamma: f64) -> ComplexMatrix2 {
    let s = gamma.sqrt();
    ComplexMatrix2 {
        a00: Complex64::new(s, 0.0),
        a01: Complex64::new(0.0, 0.0),
        a10: Complex64::new(0.0, 0.0),
        a11: Complex64::new(-s, 0.0),
    }
}

#[test]
fn lindblad_step_preserves_basic_invariants_under_effective_evolution() {
    let rho0 = initial_density_from_information_density(0.61);
    let h = diagonal_hamiltonian(1.3);
    let l = dephasing_lindblad(0.08);

    let rho1 = lindblad_density_step(&rho0, &h, &[l], 0.05);

    assert!(has_only_finite_entries(&rho1));
    assert!(trace_distance_from_one(&rho1) < 1e-9);
    assert!(hermiticity_residual(&rho1) < 1e-9);
    assert!(entropy_von_neumann_2x2(&rho1).is_finite());
    assert!(entropy_von_neumann_2x2(&rho1) >= 0.0);
    assert!(min_eigenvalue_2x2(&rho1) >= -1e-9);
}

#[test]
fn repeated_effective_steps_remain_finite_and_near_physical_region() {
    let h = diagonal_hamiltonian(0.9);
    let l = dephasing_lindblad(0.12);
    let mut rho = initial_density_from_information_density(0.44);

    for _ in 0..64 {
        rho = lindblad_density_step(&rho, &h, &[l], 0.02);
    }

    assert!(has_only_finite_entries(&rho));
    assert!(trace_distance_from_one(&rho) < 1e-9);
    assert!(hermiticity_residual(&rho) < 1e-9);
    assert!(entropy_von_neumann_2x2(&rho) >= 0.0);
    assert!(min_eigenvalue_2x2(&rho) >= -1e-9);
}
