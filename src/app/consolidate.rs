use crate::app::requests::ConsolidateRequest;
use crate::core::analysis::{ANALYSIS_VERSION, REPORT_SCHEMA_VERSION};
use crate::core::benchmark::{run_synthetic_ablation, run_synthetic_benchmark};
use crate::core::reporting::{
    render_consolidated_markdown, try_read_external_comparison_json, try_read_semgrep_summary_json,
    write_consolidated_json_report, ConsolidatedComparisonReport, ExternalBaselineReference,
};
use crate::util::experiment_manifest::{
    build_experiment_manifest, default_manifest_path, write_experiment_manifest,
    ExperimentManifestSpec,
};
use crate::util::params::{parse_kelvin, parse_relativistic_fraction};
use crate::util::paths::ensure_parent_dir;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn execute_consolidate(request: &ConsolidateRequest<'_>) -> Result<()> {
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

    let benchmark = run_synthetic_benchmark(
        request.input_dir,
        request.quantum_noise,
        beta,
        kelvin,
        request.seed,
    )?;

    let ablation = run_synthetic_ablation(
        request.input_dir,
        request.quantum_noise,
        beta,
        kelvin,
        request.seed,
    )?;

    let external_comparison_path = Path::new("target/comparison-report.json");
    let external_comparison_json = try_read_external_comparison_json(external_comparison_path)?;

    let semgrep_summary_path = Path::new("target/semgrep-summary.json");
    let semgrep_summary_json = try_read_semgrep_summary_json(semgrep_summary_path)?;

    let external_baseline = if external_comparison_json.is_some() || semgrep_summary_json.is_some()
    {
        ExternalBaselineReference {
            name: "radon+semgrep".to_string(),
            integrated_automatically: true,
            asset_path: "external_baselines/run-radon-benchmark.ps1; external_baselines/run-semgrep-benchmark.ps1".to_string(),
            doc_path: "docs/experiments/EXTERNAL_BASELINE.md; docs/experiments/SEMGREP_BASELINE.md".to_string(),
            status: "ingested_from_json".to_string(),
            notes: "Rust did not execute external tools directly, but one or more external comparison artifacts were found and ingested into the consolidated report.".to_string(),
        }
    } else {
        ExternalBaselineReference {
            name: "radon+semgrep".to_string(),
            integrated_automatically: false,
            asset_path: "external_baselines/run-radon-benchmark.ps1; external_baselines/run-semgrep-benchmark.ps1".to_string(),
            doc_path: "docs/experiments/EXTERNAL_BASELINE.md; docs/experiments/SEMGREP_BASELINE.md".to_string(),
            status: "reference_only".to_string(),
            notes: "External baseline assets exist, but no Radon or Semgrep artifact was found at consolidate time.".to_string(),
        }
    };

    let report = ConsolidatedComparisonReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed: request.seed,
        benchmark,
        ablation,
        external_baseline,
        external_comparison_json: external_comparison_json.clone(),
        semgrep_summary_json: semgrep_summary_json.clone(),
    };

    write_consolidated_json_report(request.json_out, &report)?;

    let markdown = render_consolidated_markdown(&report);
    fs::write(request.markdown_out, markdown).with_context(|| {
        format!(
            "failed to write consolidated markdown report: {}",
            request.markdown_out.display()
        )
    })?;

    let manifest_path = default_manifest_path(request.json_out);
    let manifest = build_experiment_manifest(&ExperimentManifestSpec {
        experiment_type: "consolidate",
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
        external_comparison_ingested: Some(
            external_comparison_json.is_some() || semgrep_summary_json.is_some(),
        ),
    })?;
    write_experiment_manifest(&manifest_path, &manifest)?;

    println!("flux-sim consolidate OK");
    println!(
        "benchmark_files={}",
        report.benchmark.aggregate.files_analyzed
    );
    println!("ablation_variants={}", report.ablation.aggregate.len());
    println!(
        "external_baseline_integrated={}",
        report.external_baseline.integrated_automatically
    );
    println!("json_out={}", request.json_out.display());
    println!("markdown_out={}", request.markdown_out.display());
    println!("manifest_out={}", manifest_path.display());

    Ok(())
}
