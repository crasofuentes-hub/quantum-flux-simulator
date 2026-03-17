use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn consolidate_ingests_external_comparison_json_when_present() {
    let target_dir = Path::new("target");
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).expect("target dir should be creatable");
    }

    let comparison_path = target_dir.join("comparison-report.json");
    let json_out = target_dir.join("consolidated-with-external.json");
    let md_out = target_dir.join("consolidated-with-external.md");

    fs::write(
        &comparison_path,
        r#"{
  "benchmark_source": "synthetic_dataset_v0",
  "seed": 42,
  "aggregate": {
    "files_analyzed": 4,
    "class_accuracy": 1.0,
    "mean_flux_stability": 75.0
  },
  "entries": [
    {
      "path": "benchmarks\\dataset\\crypto_heavy.py",
      "expected_class": "crypto",
      "detected_class": "crypto",
      "class_match": true
    }
  ]
}"#,
    )
    .expect("comparison-report.json should be writable");

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
            "target/consolidated-with-external.json",
            "--markdown-out",
            "target/consolidated-with-external.md",
            "--seed",
            "42",
        ])
        .assert()
        .success();

    let consolidated = fs::read_to_string(&json_out).expect("consolidated json should be readable");
    assert!(consolidated.contains("\"external_comparison_json\""));
    assert!(consolidated.contains("\"ingested_from_json\""));
    assert!(consolidated.contains("\"integrated_automatically\": true"));
    assert!(consolidated.contains("\"benchmark_source\": \"synthetic_dataset_v0\""));

    let markdown = fs::read_to_string(&md_out).expect("consolidated markdown should be readable");
    assert!(markdown.contains("external comparison JSON loaded from"));

    if comparison_path.exists() {
        fs::remove_file(&comparison_path)
            .expect("comparison-report.json should be removable after test");
    }
}
