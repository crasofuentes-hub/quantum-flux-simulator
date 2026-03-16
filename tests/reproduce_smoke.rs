use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    dir.push(format!("flux_sim_{prefix}_{nanos}"));
    fs::create_dir_all(&dir).expect("temp dir should be creatable");
    dir
}

#[test]
fn reproduce_file_generates_report_and_manifest() {
    let root = unique_temp_dir("reproduce_file");
    let input = root.join("sample.py");
    let report = root.join("sample-report.json");
    let manifest = root.join("sample-manifest.json");

    fs::write(
        &input,
        "def add(a, b):\n    return a + b\n\nfor i in range(3):\n    print(add(i, i))\n",
    )
    .expect("input file should be writable");

    Command::cargo_bin("flux-sim")
        .expect("binary should build")
        .args([
            "reproduce",
            input.to_str().expect("utf8 path"),
            "--seed",
            "123",
            "--json-out",
            report.to_str().expect("utf8 path"),
            "--manifest-out",
            manifest.to_str().expect("utf8 path"),
        ])
        .assert()
        .success();

    assert!(report.exists(), "report JSON should exist");
    assert!(manifest.exists(), "manifest JSON should exist");

    let manifest_json = fs::read_to_string(&manifest).expect("manifest should be readable");
    assert!(manifest_json.contains("\"mode\": \"analyze\""));
    assert!(manifest_json.contains("\"seed\": 123"));
    assert!(manifest_json.contains("\"input_kind\": \"file\""));
    assert!(manifest_json.contains("\"input_fingerprint\":"));
}

#[test]
fn reproduce_directory_generates_batch_report_and_manifest() {
    let root = unique_temp_dir("reproduce_dir");
    let input_dir = root.join("corpus");
    let report = root.join("corpus-report.json");
    let manifest = root.join("corpus-manifest.json");

    fs::create_dir_all(&input_dir).expect("input dir should be creatable");
    fs::write(input_dir.join("a.py"), "def f(x):\n    return x * x\n")
        .expect("first file should be writable");
    fs::write(
        input_dir.join("b.rs"),
        "fn square(x: i32) -> i32 { x * x }\n",
    )
    .expect("second file should be writable");

    Command::cargo_bin("flux-sim")
        .expect("binary should build")
        .args([
            "reproduce",
            input_dir.to_str().expect("utf8 path"),
            "--seed",
            "321",
            "--json-out",
            report.to_str().expect("utf8 path"),
            "--manifest-out",
            manifest.to_str().expect("utf8 path"),
        ])
        .assert()
        .success();

    assert!(report.exists(), "batch report JSON should exist");
    assert!(manifest.exists(), "manifest JSON should exist");

    let manifest_json = fs::read_to_string(&manifest).expect("manifest should be readable");
    assert!(manifest_json.contains("\"mode\": \"batch\""));
    assert!(manifest_json.contains("\"seed\": 321"));
    assert!(manifest_json.contains("\"input_kind\": \"directory\""));
    assert!(manifest_json.contains("\"output_report_path\":"));
}
