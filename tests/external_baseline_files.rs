use std::path::Path;

#[test]
fn external_baseline_assets_exist() {
    assert!(Path::new("external_baselines/run-radon-benchmark.ps1").exists());
    assert!(Path::new("docs/experiments/EXTERNAL_BASELINE.md").exists());
}
