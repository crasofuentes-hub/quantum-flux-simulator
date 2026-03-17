use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn consolidate_ingests_semgrep_summary_json_when_present() {
    let target_dir = Path::new("target");
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).expect("target dir should be creatable");
    }

    let semgrep_path = target_dir.join("semgrep-summary.json");
    let json_out = target_dir.join("consolidated-with-semgrep.json");
    let md_out = target_dir.join("consolidated-with-semgrep.md");

    fs::write(
        &semgrep_path,
        r#"{
  "dataset": "seeded_defects_v1",
  "tool": "semgrep",
  "total_findings": 3,
  "files_scanned": 2,
  "files": {
    "benchmarks/dataset/crypto_heavy.py": {
      "findings": 2,
      "checks": ["rule.a", "rule.b"],
      "severities": ["WARNING", "ERROR"]
    }
  }
}"#,
    )
    .expect("semgrep-summary.json should be writable");

    if json_out.exists() {
        fs::remove_file(&json_out).expect("old consolidated json should be removable");
    }
    if md_out.exists() {
        fs::remove_file(&md_out).expect("old consolidated markdown should be removable");
    }

    Command::cargo_bin("flux-sim")
        .expect("binary should exist")
        .args([
            "consolidate",
            "benchmarks/dataset",
            "--quantum-noise",
            "0.01",
            "--relativistic",
            "0.2c",
            "--target-temp",
            "77K",
            "--json-out",
            "target/consolidated-with-semgrep.json",
            "--markdown-out",
            "target/consolidated-with-semgrep.md",
            "--seed",
            "42",
        ])
        .assert()
        .success();

    let consolidated = fs::read_to_string(&json_out).expect("consolidated json should be readable");
    assert!(consolidated.contains("\"semgrep_summary_json\""));
    assert!(consolidated.contains("\"tool\": \"semgrep\""));
    assert!(consolidated.contains("\"integrated_automatically\": true"));

    let markdown = fs::read_to_string(&md_out).expect("consolidated markdown should be readable");
    assert!(markdown.contains("Semgrep summary JSON loaded from"));
    assert!(markdown.contains("| total_findings | 3 |"));
    assert!(markdown.contains("| files_scanned | 2 |"));
    assert!(markdown.contains("| benchmarks/dataset/crypto_heavy.py | crypto | crypto | yes |"));
    assert!(markdown.contains("| 2 |"));

    if semgrep_path.exists() {
        fs::remove_file(&semgrep_path)
            .expect("semgrep-summary.json should be removable after test");
    }
}
