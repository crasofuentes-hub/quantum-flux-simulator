#[path = "../src/core/analysis.rs"]
mod analysis;

use analysis::{analyze_file, AlgorithmClass, BlockKind};
use std::path::Path;

#[test]
fn crypto_case_is_detected() {
    let analysis = analyze_file(Path::new("examples/my_crypto.py"), 0.01, 0.2, 77.0, None)
        .expect("analysis should succeed");

    assert_eq!(analysis.algorithm_class, AlgorithmClass::Crypto);
    assert!(!analysis.crypto_hits.is_empty());
    assert!(!analysis.hotspots.is_empty());
    assert!(!analysis.intermediate_model.critical_blocks.is_empty());
    assert!(analysis.stability_score >= 0.0);
    assert!(analysis.singularity_risk >= 0.0);
    assert!(analysis.singularity_risk <= 1.0);

    let has_crypto_block = analysis
        .intermediate_model
        .critical_blocks
        .iter()
        .any(|b| b.kind == BlockKind::CryptoPrimitive);

    assert!(has_crypto_block);
}

#[test]
fn numerical_case_is_detected() {
    let analysis = analyze_file(Path::new("examples/navier_stub.py"), 0.01, 0.1, 300.0, None)
        .expect("analysis should succeed");

    assert_eq!(analysis.algorithm_class, AlgorithmClass::Numerical);
    assert!(!analysis.numerical_hits.is_empty());
    assert!(!analysis.intermediate_model.information_channels.is_empty());
    assert!(analysis.intermediate_model.structural_complexity > 0.0);
}

#[test]
fn ml_case_is_detected() {
    let analysis = analyze_file(Path::new("examples/ml_stub.py"), 0.02, 0.3, 300.0, None)
        .expect("analysis should succeed");

    assert_eq!(analysis.algorithm_class, AlgorithmClass::Ml);
    assert!(!analysis.ml_hits.is_empty());

    let has_ml_block = analysis
        .intermediate_model
        .critical_blocks
        .iter()
        .any(|b| b.kind == BlockKind::MlKernel);

    assert!(has_ml_block);
}
