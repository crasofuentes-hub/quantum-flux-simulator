use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn benchmark_runs_on_seeded_defects_dataset() {
    let json_out = Path::new("target/seeded-defects-benchmark.json");
    let manifest_out = Path::new("target/seeded-defects-benchmark.manifest.json");
    let labels_path = Path::new("datasets/seeded_defects/labels.json");

    assert!(labels_path.exists(), "seeded defects labels should exist");

    if json_out.exists() {
        fs::remove_file(json_out).expect("old seeded defects benchmark json should be removable");
    }
    if manifest_out.exists() {
        fs::remove_file(manifest_out)
            .expect("old seeded defects benchmark manifest should be removable");
    }

    Command::cargo_bin("flux-sim")
        .expect("binary should exist")
        .args([
            "benchmark",
            "datasets/seeded_defects",
            "--quantum-noise",
            "0.01",
            "--relativistic",
            "0.2c",
            "--target-temp",
            "77K",
            "--json-out",
            "target/seeded-defects-benchmark.json",
            "--seed",
            "42",
        ])
        .assert()
        .success();

    assert!(
        json_out.exists(),
        "seeded defects benchmark json should exist"
    );
    assert!(
        manifest_out.exists(),
        "seeded defects benchmark manifest should exist"
    );

    let json_text =
        fs::read_to_string(json_out).expect("seeded defects benchmark json should be readable");
    assert!(json_text.contains("crypto_seeded_defect.py"));
    assert!(json_text.contains("numerical_seeded_defect.py"));
    assert!(json_text.contains("ml_seeded_defect.py"));
    assert!(json_text.contains("general_seeded_defect.py"));

    let manifest_text = fs::read_to_string(manifest_out)
        .expect("seeded defects benchmark manifest should be readable");
    assert!(manifest_text.contains("\"experiment_type\": \"benchmark\""));
    assert!(manifest_text.contains("\"input_path\": \"datasets/seeded_defects\""));
}
