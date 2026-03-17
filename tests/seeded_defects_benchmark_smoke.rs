use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn benchmark_runs_on_seeded_defects_with_expected_dataset_shape() {
    let json_out = Path::new("target/seeded-defects-benchmark-shape.json");
    let manifest_out = Path::new("target/seeded-defects-benchmark-shape.manifest.json");

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
            "target/seeded-defects-benchmark-shape.json",
            "--seed",
            "42",
        ])
        .assert()
        .success();

    assert!(json_out.exists(), "benchmark json should exist");
    assert!(manifest_out.exists(), "benchmark manifest should exist");

    let json_text = fs::read_to_string(json_out).expect("benchmark json should be readable");

    assert!(json_text.contains("\"files_analyzed\": 4"));
    assert!(json_text.contains("\"class_accuracy\""));
    assert!(json_text.contains("\"expected_class\": \"crypto\""));
    assert!(json_text.contains("\"expected_class\": \"numerical\""));
    assert!(json_text.contains("\"expected_class\": \"ml\""));
    assert!(json_text.contains("\"expected_class\": \"general\""));
}
