use crate::core::benchmark::run_synthetic_benchmark;
use crate::util::experiment_manifest::{
    build_experiment_manifest, default_manifest_path, write_experiment_manifest,
    ExperimentManifestSpec,
};
use crate::util::params::{parse_kelvin, parse_relativistic_fraction};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn execute_benchmark(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic: &str,
    target_temp: &str,
    json_out: &Path,
    seed: u64,
) -> Result<()> {
    let beta = parse_relativistic_fraction(relativistic)?;
    let kelvin = parse_kelvin(target_temp)?;

    let report = run_synthetic_benchmark(input_dir, quantum_noise, beta, kelvin, seed)?;
    let json =
        serde_json::to_string_pretty(&report).context("failed to serialize benchmark report")?;
    fs::write(json_out, json)
        .with_context(|| format!("failed to write benchmark JSON: {}", json_out.display()))?;

    let manifest_path = default_manifest_path(json_out);
    let manifest = build_experiment_manifest(&ExperimentManifestSpec {
        experiment_type: "benchmark",
        input_path: input_dir,
        quantum_noise,
        relativistic_beta: beta,
        target_temp_kelvin: kelvin,
        seed,
        generated_outputs: vec![
            json_out.to_string_lossy().replace('\\', "/"),
            manifest_path.to_string_lossy().replace('\\', "/"),
        ],
        external_comparison_ingested: None,
    })?;
    write_experiment_manifest(&manifest_path, &manifest)?;

    println!("flux-sim benchmark OK");
    println!("files_analyzed={}", report.aggregate.files_analyzed);
    println!("class_accuracy={}", report.aggregate.class_accuracy);
    println!("mean_baseline_risk={}", report.aggregate.mean_baseline_risk);
    println!("mean_model_risk={}", report.aggregate.mean_model_risk);
    println!(
        "mean_baseline_stability={}",
        report.aggregate.mean_baseline_stability
    );
    println!(
        "mean_model_stability={}",
        report.aggregate.mean_model_stability
    );
    println!(
        "mean_collapse_probability={}",
        report.aggregate.mean_collapse_probability
    );
    println!("manifest_out={}", manifest_path.display());

    Ok(())
}
