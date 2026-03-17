use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn benchmark_writes_json_report() {
    let out = Path::new("target/benchmark-report.json");
    let manifest = Path::new("target/benchmark-report.manifest.json");

    if out.exists() {
        fs::remove_file(out).expect("old benchmark report should be removable");
    }
    if manifest.exists() {
        fs::remove_file(manifest).expect("old benchmark manifest should be removable");
    }

    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("benchmark")
        .arg("benchmarks/dataset")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.2c")
        .arg("--target-temp")
        .arg("77K")
        .arg("--json-out")
        .arg("target/benchmark-report.json")
        .arg("--seed")
        .arg("42")
        .assert()
        .success();

    assert!(out.exists(), "benchmark JSON report should exist");
    assert!(manifest.exists(), "benchmark manifest should exist");

    let text = fs::read_to_string(out).expect("benchmark report should be readable");
    assert!(text.contains("\"class_accuracy\""));
    assert!(text.contains("\"mean_baseline_risk\""));
    assert!(text.contains("\"mean_model_risk\""));
    assert!(text.contains("\"entries\""));

    let manifest_text =
        fs::read_to_string(manifest).expect("benchmark manifest should be readable");
    assert!(manifest_text.contains("\"experiment_type\": \"benchmark\""));
    assert!(manifest_text.contains("\"input_kind\": \"directory\""));
}
