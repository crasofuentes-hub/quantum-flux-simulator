use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn consolidate_runs_on_semi_real_corpus() {
    let json_out = Path::new("target/semi-real-consolidated.json");
    let md_out = Path::new("target/semi-real-consolidated.md");

    if json_out.exists() {
        fs::remove_file(json_out).expect("old semi-real consolidated json should be removable");
    }
    if md_out.exists() {
        fs::remove_file(md_out).expect("old semi-real consolidated markdown should be removable");
    }

    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("consolidate")
        .arg("benchmarks/semi_real")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.2c")
        .arg("--target-temp")
        .arg("77K")
        .arg("--json-out")
        .arg("target/semi-real-consolidated.json")
        .arg("--markdown-out")
        .arg("target/semi-real-consolidated.md")
        .arg("--seed")
        .arg("42")
        .assert()
        .success();

    assert!(
        json_out.exists(),
        "semi-real consolidated JSON should exist"
    );
    assert!(
        md_out.exists(),
        "semi-real consolidated markdown should exist"
    );

    let json_text =
        fs::read_to_string(json_out).expect("semi-real consolidated JSON should be readable");
    assert!(json_text.contains("\"benchmark\""));
    assert!(json_text.contains("\"ablation\""));
    assert!(json_text.contains("\"external_baseline\""));
    assert!(json_text.contains("crypto_service.py"));
    assert!(json_text.contains("numerical_solver.py"));
    assert!(json_text.contains("ml_training_loop.py"));
    assert!(json_text.contains("general_pipeline.py"));
}
