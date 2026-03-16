use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn batch_writes_json_report() {
    let out = Path::new("target/batch-report.json");
    if out.exists() {
        fs::remove_file(out).expect("old batch report should be removable");
    }

    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("batch")
        .arg("benchmarks/dataset")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.2c")
        .arg("--target-temp")
        .arg("77K")
        .arg("--json-out")
        .arg("target/batch-report.json")
        .arg("--seed")
        .arg("42")
        .assert()
        .success();

    assert!(out.exists(), "batch JSON report should exist");

    let text = fs::read_to_string(out).expect("batch report should be readable");
    assert!(text.contains("\"report_schema_version\""));
    assert!(text.contains("\"analysis_version\""));
    assert!(text.contains("\"aggregate\""));
    assert!(text.contains("\"files_analyzed\""));
}
