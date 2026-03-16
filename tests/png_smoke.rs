use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn analyze_writes_png_report() {
    let out = Path::new("target/test-plot.png");
    if out.exists() {
        fs::remove_file(out).expect("old PNG should be removable");
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
        .arg("--plot")
        .arg("target/test-plot.png")
        .assert()
        .success();

    assert!(out.exists(), "PNG report should exist");

    let meta = fs::metadata(out).expect("PNG metadata should be readable");
    assert!(meta.len() > 0, "PNG report should not be empty");
}
