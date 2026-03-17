use flux_sim::core::physics::lindblad_density_step;
use flux_sim::core::state::{
    entropy_von_neumann_2x2, initial_density_from_information_density, ComplexMatrix2,
};
use num_complex::Complex64;

fn hermitian_distance(m: &ComplexMatrix2) -> f64 {
    let d = m.sub(&m.dagger());
    [d.a00.norm(), d.a01.norm(), d.a10.norm(), d.a11.norm()]
        .into_iter()
        .fold(0.0, f64::max)
}

fn trace_distance_to_one(m: &ComplexMatrix2) -> f64 {
    (m.trace().re - 1.0).abs() + m.trace().im.abs()
}

fn eigenvalues_2x2_hermitian(m: &ComplexMatrix2) -> (f64, f64) {
    let tr = (m.a00 + m.a11).re;
    let det = (m.a00 * m.a11 - m.a01 * m.a10).re;
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();
    (((tr + disc) * 0.5), ((tr - disc) * 0.5))
}

fn sample_hamiltonian() -> ComplexMatrix2 {
    ComplexMatrix2 {
        a00: Complex64::new(1.25, 0.0),
        a01: Complex64::new(0.18, 0.0),
        a10: Complex64::new(0.18, 0.0),
        a11: Complex64::new(-1.25, 0.0),
    }
}

fn sample_lindblad_ops() -> [ComplexMatrix2; 2] {
    [
        ComplexMatrix2 {
            a00: Complex64::new(0.3_f64.sqrt(), 0.0),
            a01: Complex64::new(0.0, 0.0),
            a10: Complex64::new(0.0, 0.0),
            a11: Complex64::new(-0.3_f64.sqrt(), 0.0),
        },
        ComplexMatrix2 {
            a00: Complex64::new(0.0, 0.0),
            a01: Complex64::new(0.15_f64.sqrt(), 0.0),
            a10: Complex64::new(0.0, 0.0),
            a11: Complex64::new(0.0, 0.0),
        },
    ]
}

#[test]
fn repeated_effective_evolution_preserves_near_unit_trace_and_hermiticity() {
    let h = sample_hamiltonian();
    let ops = sample_lindblad_ops();
    let mut rho = initial_density_from_information_density(0.61);

    for _ in 0..128 {
        rho = lindblad_density_step(&rho, &h, &ops, 0.01);
    }

    assert!(
        trace_distance_to_one(&rho) < 1e-9,
        "trace should remain near 1 after stabilization"
    );
    assert!(
        hermitian_distance(&rho) < 1e-9,
        "state should remain Hermitian after stabilization"
    );
}

#[test]
fn repeated_effective_evolution_keeps_eigenvalues_in_reasonable_region() {
    let h = sample_hamiltonian();
    let ops = sample_lindblad_ops();
    let mut rho = initial_density_from_information_density(0.37);

    for _ in 0..256 {
        rho = lindblad_density_step(&rho, &h, &ops, 0.01);
    }

    let (l1, l2) = eigenvalues_2x2_hermitian(&rho);

    assert!(
        l1.is_finite() && l2.is_finite(),
        "eigenvalues must be finite"
    );
    assert!(
        l1 <= 1.0 + 1e-6,
        "largest eigenvalue should remain near physical region"
    );
    assert!(
        l2 >= -1e-6,
        "smallest eigenvalue should not drift far below zero"
    );
}

#[test]
fn entropy_remains_finite_and_non_negative_under_repeated_steps() {
    let h = sample_hamiltonian();
    let ops = sample_lindblad_ops();
    let mut rho = initial_density_from_information_density(0.52);

    for _ in 0..200 {
        rho = lindblad_density_step(&rho, &h, &ops, 0.01);
    }

    let entropy = entropy_von_neumann_2x2(&rho);
    assert!(entropy.is_finite(), "entropy must remain finite");
    assert!(
        entropy >= -1e-12,
        "entropy should remain non-negative up to numerical tolerance"
    );
}

#[test]
fn zero_step_leaves_state_unchanged_after_stabilization() {
    let h = sample_hamiltonian();
    let ops = sample_lindblad_ops();
    let rho0 = initial_density_from_information_density(0.73);
    let rho1 = lindblad_density_step(&rho0, &h, &ops, 0.0);

    assert!(trace_distance_to_one(&rho1) < 1e-12);
    assert!(hermitian_distance(&rho1) < 1e-12);

    let delta = rho1.sub(&rho0);
    let max_delta = [
        delta.a00.norm(),
        delta.a01.norm(),
        delta.a10.norm(),
        delta.a11.norm(),
    ]
    .into_iter()
    .fold(0.0, f64::max);

    assert!(
        max_delta < 1e-12,
        "dt=0 should leave the stabilized state unchanged"
    );
}
