use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn consolidate_writes_json_and_markdown_reports() {
    let json_out = Path::new("target/consolidated-report.json");
    let md_out = Path::new("target/consolidated-report.md");
    let manifest_out = Path::new("target/consolidated-report.manifest.json");
    let external_comparison = Path::new("target/comparison-report.json");
    let semgrep_summary = Path::new("target/semgrep-summary.json");

    if json_out.exists() {
        fs::remove_file(json_out).expect("old consolidated json should be removable");
    }
    if md_out.exists() {
        fs::remove_file(md_out).expect("old consolidated markdown should be removable");
    }
    if manifest_out.exists() {
        fs::remove_file(manifest_out).expect("old consolidated manifest should be removable");
    }
    if external_comparison.exists() {
        fs::remove_file(external_comparison)
            .expect("stale external comparison json should be removable");
    }
    if semgrep_summary.exists() {
        fs::remove_file(semgrep_summary).expect("stale semgrep summary json should be removable");
    }

    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("consolidate")
        .arg("benchmarks/dataset")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.2c")
        .arg("--target-temp")
        .arg("77K")
        .arg("--json-out")
        .arg("target/consolidated-report.json")
        .arg("--markdown-out")
        .arg("target/consolidated-report.md")
        .arg("--seed")
        .arg("42")
        .assert()
        .success();

    assert!(json_out.exists(), "consolidated JSON report should exist");
    assert!(md_out.exists(), "consolidated markdown report should exist");
    assert!(manifest_out.exists(), "consolidated manifest should exist");

    let json_text = fs::read_to_string(json_out).expect("consolidated JSON should be readable");
    assert!(json_text.contains("\"benchmark\""));
    assert!(json_text.contains("\"ablation\""));
    assert!(json_text.contains("\"external_baseline\""));
    assert!(json_text.contains("\"integrated_automatically\": false"));
    assert!(json_text.contains("\"reference_only\""));

    let md_text = fs::read_to_string(md_out).expect("consolidated markdown should be readable");
    assert!(md_text.contains("# Consolidated comparison report"));
    assert!(md_text.contains("## Internal benchmark summary"));
    assert!(md_text.contains("## Ablation summary"));
    assert!(md_text.contains("## External baseline status"));
    assert!(md_text.contains("## Cross-file comparison"));
    assert!(md_text.contains("| file | expected_class | detected_class |"));
    assert!(md_text.contains("crypto_heavy.py"));

    let manifest_text =
        fs::read_to_string(manifest_out).expect("consolidated manifest should be readable");
    assert!(manifest_text.contains("\"experiment_type\": \"consolidate\""));
    assert!(manifest_text.contains("\"external_comparison_ingested\": false"));
}
