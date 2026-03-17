use flux_sim::core::analysis::analyze_file_with_seed;
use flux_sim::core::state::ComplexMatrix2;
use num_complex::Complex64;
use std::path::Path;

fn hermitian_distance(m: &ComplexMatrix2) -> f64 {
    let d = m.sub(&m.dagger());
    [d.a00.norm(), d.a01.norm(), d.a10.norm(), d.a11.norm()]
        .into_iter()
        .fold(0.0, f64::max)
}

#[test]
fn effective_model_remains_in_reasonable_region_across_small_parameter_sweep() {
    let quantum_noises = [0.001, 0.01, 0.05];
    let relativistic_betas = [0.0, 0.2, 0.6];
    let target_temps = [10.0, 77.0, 300.0];

    for quantum_noise in quantum_noises {
        for relativistic_beta in relativistic_betas {
            for target_temp in target_temps {
                let analysis = analyze_file_with_seed(
                    Path::new("examples/my_crypto.py"),
                    quantum_noise,
                    relativistic_beta,
                    target_temp,
                    None,
                    42,
                )
                .expect("analysis should succeed across sweep");

                assert!(
                    analysis.physical_model.decoherence_rate.is_finite(),
                    "decoherence_rate should be finite"
                );
                assert!(
                    analysis
                        .physical_model
                        .effective_runtime_dilation
                        .is_finite(),
                    "runtime_dilation should be finite"
                );
                assert!(
                    analysis.physical_model.von_neumann_entropy.is_finite(),
                    "von_neumann_entropy should be finite"
                );
                assert!(
                    analysis
                        .physical_model
                        .global_constraint_penalty
                        .is_finite(),
                    "global_constraint_penalty should be finite"
                );
                assert!(
                    analysis.stability_score.is_finite(),
                    "stability_score should be finite"
                );
                assert!(
                    analysis.singularity_risk.is_finite(),
                    "singularity_risk should be finite"
                );

                assert!(
                    analysis.physical_model.von_neumann_entropy >= -1e-12,
                    "entropy should remain non-negative up to tolerance"
                );
                assert!(
                    analysis.singularity_risk >= -1e-12,
                    "risk should remain non-negative up to tolerance"
                );
                assert!(
                    analysis.singularity_risk <= 1.0 + 1e-12,
                    "risk should remain bounded near [0,1]"
                );

                for block in &analysis.physical_model.blocks {
                    let rho = ComplexMatrix2 {
                        a00: Complex64::new(
                            block.density_state.rho00_re,
                            block.density_state.rho00_im,
                        ),
                        a01: Complex64::new(
                            block.density_state.rho01_re,
                            block.density_state.rho01_im,
                        ),
                        a10: Complex64::new(
                            block.density_state.rho10_re,
                            block.density_state.rho10_im,
                        ),
                        a11: Complex64::new(
                            block.density_state.rho11_re,
                            block.density_state.rho11_im,
                        ),
                    };

                    let trace = rho.trace();
                    assert!(
                        trace.re.is_finite() && trace.im.is_finite(),
                        "trace should be finite"
                    );
                    assert!(
                        (trace.re - 1.0).abs() < 1e-9,
                        "trace real part should remain near 1"
                    );
                    assert!(
                        trace.im.abs() < 1e-9,
                        "trace imaginary part should remain near 0"
                    );
                    assert!(
                        hermitian_distance(&rho) < 1e-9,
                        "density should remain Hermitian after stabilization"
                    );
                }
            }
        }
    }
}
