use crate::app::requests::AblationRequest;
use crate::core::benchmark::{render_ablation_markdown, run_synthetic_ablation};
use crate::util::experiment_manifest::{
    build_experiment_manifest, default_manifest_path, write_experiment_manifest,
    ExperimentManifestSpec,
};
use crate::util::params::{parse_kelvin, parse_relativistic_fraction};
use crate::util::paths::ensure_parent_dir;
use anyhow::{bail, Context, Result};
use std::fs;

pub fn execute_ablation(request: &AblationRequest<'_>) -> Result<()> {
    if !request.input_dir.exists() {
        bail!("input_dir does not exist: {}", request.input_dir.display());
    }
    if !request.input_dir.is_dir() {
        bail!(
            "input_dir is not a directory: {}",
            request.input_dir.display()
        );
    }

    let beta = parse_relativistic_fraction(request.relativistic)?;
    let kelvin = parse_kelvin(request.target_temp)?;

    ensure_parent_dir(request.json_out)?;
    ensure_parent_dir(request.markdown_out)?;

    let report = run_synthetic_ablation(
        request.input_dir,
        request.quantum_noise,
        beta,
        kelvin,
        request.seed,
    )?;

    let json =
        serde_json::to_string_pretty(&report).context("failed to serialize ablation report")?;
    fs::write(request.json_out, json).with_context(|| {
        format!(
            "failed to write ablation JSON report: {}",
            request.json_out.display()
        )
    })?;

    let markdown = render_ablation_markdown(&report);
    fs::write(request.markdown_out, markdown).with_context(|| {
        format!(
            "failed to write ablation markdown report: {}",
            request.markdown_out.display()
        )
    })?;

    let manifest_path = default_manifest_path(request.json_out);
    let manifest = build_experiment_manifest(&ExperimentManifestSpec {
        experiment_type: "ablation",
        input_path: request.input_dir,
        quantum_noise: request.quantum_noise,
        relativistic_beta: beta,
        target_temp_kelvin: kelvin,
        seed: request.seed,
        generated_outputs: vec![
            request.json_out.to_string_lossy().replace('\\', "/"),
            request.markdown_out.to_string_lossy().replace('\\', "/"),
            manifest_path.to_string_lossy().replace('\\', "/"),
        ],
        external_comparison_ingested: None,
    })?;
    write_experiment_manifest(&manifest_path, &manifest)?;

    println!("flux-sim ablation OK");
    println!("files_analyzed={}", report.entries.len());
    println!("variants={}", report.aggregate.len());
    println!("json_out={}", request.json_out.display());
    println!("markdown_out={}", request.markdown_out.display());
    println!("manifest_out={}", manifest_path.display());

    Ok(())
}
