#[path = "../src/core/analysis.rs"]
mod analysis;

use analysis::{analyze_file, AlgorithmClass};
use std::path::Path;

#[test]
fn crypto_case_is_detected() {
    let analysis = analyze_file(Path::new("examples/my_crypto.py"), 0.01, 0.2, 77.0, None)
        .expect("analysis should succeed");

    assert_eq!(analysis.algorithm_class, AlgorithmClass::Crypto);
    assert!(!analysis.crypto_hits.is_empty());
    assert!(analysis.stability_score >= 0.0);
    assert!(analysis.singularity_risk >= 0.0);
    assert!(analysis.singularity_risk <= 1.0);
}

#[test]
fn numerical_case_is_detected() {
    let analysis = analyze_file(Path::new("examples/navier_stub.py"), 0.01, 0.1, 300.0, None)
        .expect("analysis should succeed");

    assert_eq!(analysis.algorithm_class, AlgorithmClass::Numerical);
    assert!(!analysis.numerical_hits.is_empty());
}

#[test]
fn ml_case_is_detected() {
    let analysis = analyze_file(Path::new("examples/ml_stub.py"), 0.02, 0.3, 300.0, None)
        .expect("analysis should succeed");

    assert_eq!(analysis.algorithm_class, AlgorithmClass::Ml);
    assert!(!analysis.ml_hits.is_empty());
}
