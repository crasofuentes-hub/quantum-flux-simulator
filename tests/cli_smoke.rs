use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn analyze_smoke_works() {
    let mut cmd = Command::cargo_bin("flux-sim").expect("binary should exist");
    cmd.arg("analyze")
        .arg("examples/my_crypto.py")
        .arg("--quantum-noise")
        .arg("0.01")
        .arg("--relativistic")
        .arg("0.8c")
        .arg("--target-temp")
        .arg("77K")
        .assert()
        .success()
        .stdout(predicate::str::contains("flux-sim analysis OK"));
}
