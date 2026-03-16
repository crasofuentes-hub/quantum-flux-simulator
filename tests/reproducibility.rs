use flux_sim::core::analysis::analyze_file_with_seed;
use std::path::Path;

#[test]
fn same_seed_produces_same_solver_summary() {
    let a = analyze_file_with_seed(
        Path::new("examples/my_crypto.py"),
        0.01,
        0.2,
        77.0,
        None,
        42,
    )
    .expect("first analysis should succeed");

    let b = analyze_file_with_seed(
        Path::new("examples/my_crypto.py"),
        0.01,
        0.2,
        77.0,
        None,
        42,
    )
    .expect("second analysis should succeed");

    assert_eq!(a.run_metadata.seed, b.run_metadata.seed);
    assert_eq!(
        a.run_metadata.input_fingerprint,
        b.run_metadata.input_fingerprint
    );
    assert!((a.solver_summary.mean_stress - b.solver_summary.mean_stress).abs() < 1e-12);
    assert!((a.solver_summary.stress_variance - b.solver_summary.stress_variance).abs() < 1e-12);
    assert!((a.stability_score - b.stability_score).abs() < 1e-12);
    assert!((a.singularity_risk - b.singularity_risk).abs() < 1e-12);
}
