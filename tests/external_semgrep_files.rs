use std::path::Path;

#[test]
fn semgrep_external_baseline_assets_exist() {
    assert!(Path::new("external_baselines/run-semgrep-benchmark.ps1").exists());
    assert!(Path::new("docs/experiments/SEMGREP_BASELINE.md").exists());
    assert!(Path::new("datasets/seeded_defects/labels.json").exists());
}
