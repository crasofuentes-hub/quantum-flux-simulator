use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn benchmark_accepts_typescript_seeded_input() {
    let dataset_dir = Path::new("target/typescript-benchmark-dataset");
    if dataset_dir.exists() {
        fs::remove_dir_all(dataset_dir).expect("old dataset dir should be removable");
    }
    fs::create_dir_all(dataset_dir).expect("dataset dir should be creatable");

    let ts_file = dataset_dir.join("ml_case.ts");
    fs::write(
        &ts_file,
        r#"export function trainStep(tensor: number[], weights: number[]) {
  const gradient = tensor.map((x, i) => x * weights[i]);
  const loss = gradient.reduce((acc, value) => acc + value, 0);
  return { gradient, loss };
}
"#,
    )
    .expect("ts file should be writable");

    let json_out = Path::new("target/typescript-benchmark.json");
    let manifest_out = Path::new("target/typescript-benchmark.manifest.json");

    if json_out.exists() {
        fs::remove_file(json_out).expect("old benchmark json should be removable");
    }
    if manifest_out.exists() {
        fs::remove_file(manifest_out).expect("old benchmark manifest should be removable");
    }

    Command::cargo_bin("flux-sim")
        .expect("binary should exist")
        .args([
            "benchmark",
            "target/typescript-benchmark-dataset",
            "--quantum-noise",
            "0.01",
            "--relativistic",
            "0.2c",
            "--target-temp",
            "77K",
            "--json-out",
            "target/typescript-benchmark.json",
            "--seed",
            "42",
        ])
        .assert()
        .success();

    let json_text = fs::read_to_string(json_out).expect("benchmark json should be readable");
    assert!(json_text.contains("\"files_analyzed\": 1"));
    assert!(json_text.contains("ml_case.ts"));

    let manifest_text =
        fs::read_to_string(manifest_out).expect("benchmark manifest should be readable");
    assert!(manifest_text.contains("\"experiment_type\": \"benchmark\""));
}
