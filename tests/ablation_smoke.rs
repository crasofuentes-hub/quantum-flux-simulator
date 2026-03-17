use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn ablation_writes_json_and_markdown_reports() {
    let json_out = Path::new("target/ablation-report.json");
    let md_out = Path::new("target/ablation-report.md");
    let manifest_out = Path::new("target/ablation-report.manifest.json");

    if json_out.exists() {
        fs::remove_file(json_out).expect("old ablation json should be removable");
    }
    if md_out.exists() {
        fs::remove_file(md_out).expect("old ablation markdown should be removable");
    }
    if manifest_out.exists() {
        fs::remove_file(manifest_out).expect("old ablation manifest should be removable");
    }

    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("ablation")
        .arg("benchmarks/dataset")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.2c")
        .arg("--target-temp")
        .arg("77K")
        .arg("--json-out")
        .arg("target/ablation-report.json")
        .arg("--markdown-out")
        .arg("target/ablation-report.md")
        .arg("--seed")
        .arg("42")
        .assert()
        .success();

    assert!(json_out.exists(), "ablation JSON report should exist");
    assert!(md_out.exists(), "ablation markdown report should exist");
    assert!(manifest_out.exists(), "ablation manifest should exist");

    let json_text = fs::read_to_string(json_out).expect("ablation JSON should be readable");
    assert!(json_text.contains("\"aggregate\""));
    assert!(json_text.contains("\"structural_baseline\""));
    assert!(json_text.contains("\"full_model\""));
    assert!(json_text.contains("\"no_lindblad\""));
    assert!(json_text.contains("\"no_global_constraint\""));
    assert!(json_text.contains("\"no_relativistic_factor\""));
    assert!(json_text.contains("\"no_solver\""));

    let md_text = fs::read_to_string(md_out).expect("ablation markdown should be readable");
    assert!(md_text.contains("| variant | files_analyzed |"));
    assert!(md_text.contains("structural_baseline"));
    assert!(md_text.contains("full_model"));

    let manifest_text =
        fs::read_to_string(manifest_out).expect("ablation manifest should be readable");
    assert!(manifest_text.contains("\"experiment_type\": \"ablation\""));
    assert!(manifest_text.contains("\"input_kind\": \"directory\""));
}
