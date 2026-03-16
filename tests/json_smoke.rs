use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn analyze_writes_json_report() {
    let out = Path::new("target/test-report.json");
    if out.exists() {
        fs::remove_file(out).expect("old report should be removable");
    }

    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("analyze")
        .arg("examples/my_crypto.py")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.8c")
        .arg("--target-temp")
        .arg("77K")
        .arg("--json-out")
        .arg("target/test-report.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("flux-sim analysis OK"));

    assert!(out.exists(), "JSON report should exist");

    let text = fs::read_to_string(out).expect("report should be readable");
    assert!(text.contains("\"algorithm_class\""));
    assert!(text.contains("\"crypto_hits\""));
}
