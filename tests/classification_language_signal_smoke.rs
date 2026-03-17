use flux_sim::core::analysis::analyze_file_with_seed;
use std::fs;
use std::path::Path;

#[test]
fn rust_crypto_signal_is_classified_as_crypto() {
    let path = Path::new("target/rust_crypto_case.rs");
    fs::write(
        path,
        r#"use sha2::{Digest, Sha256};

pub fn verify_signature(message: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(message);
    let digest = hasher.finalize();
    digest.to_vec()
}
"#,
    )
    .expect("rust crypto fixture should be writable");

    let analysis = analyze_file_with_seed(path, 0.01, 0.2, 77.0, None, 42)
        .expect("rust analysis should succeed");

    assert_eq!(
        format!("{:?}", analysis.algorithm_class).to_ascii_lowercase(),
        "crypto"
    );
}

#[test]
fn typescript_ml_signal_is_classified_as_ml() {
    let path = Path::new("target/typescript_ml_case.ts");
    fs::write(
        path,
        r#"export function trainStep(tensor: number[], weights: number[]) {
  const logits = tensor.map((x, i) => x * weights[i]);
  const loss = logits.reduce((acc, value) => acc + value, 0);
  const gradient = logits.map(x => x - 1.0);
  return { logits, loss, gradient };
}
"#,
    )
    .expect("typescript ml fixture should be writable");

    let analysis = analyze_file_with_seed(path, 0.01, 0.2, 77.0, None, 42)
        .expect("typescript analysis should succeed");

    assert_eq!(
        format!("{:?}", analysis.algorithm_class).to_ascii_lowercase(),
        "ml"
    );
}
