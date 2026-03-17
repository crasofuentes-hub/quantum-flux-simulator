use crate::app::requests::ConsolidateRequest;
use crate::core::analysis::{ANALYSIS_VERSION, REPORT_SCHEMA_VERSION};
use crate::core::benchmark::{run_synthetic_ablation, run_synthetic_benchmark};
use crate::core::reporting::{
    render_consolidated_markdown, try_read_external_comparison_json,
    write_consolidated_json_report, ConsolidatedComparisonReport, ExternalBaselineReference,
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

    let external_baseline = if external_comparison_json.is_some() {
        ExternalBaselineReference {
            name: "radon".to_string(),
            integrated_automatically: true,
            asset_path: "external_baselines/run-radon-benchmark.ps1".to_string(),
            doc_path: "docs/experiments/EXTERNAL_BASELINE.md".to_string(),
            status: "ingested_from_json".to_string(),
            notes: "Rust did not execute Radon directly, but target/comparison-report.json was found and ingested into the consolidated report.".to_string(),
        }
    } else {
        ExternalBaselineReference {
            name: "radon".to_string(),
            integrated_automatically: false,
            asset_path: "external_baselines/run-radon-benchmark.ps1".to_string(),
            doc_path: "docs/experiments/EXTERNAL_BASELINE.md".to_string(),
            status: "reference_only".to_string(),
            notes: "External baseline assets exist, but target/comparison-report.json was not found at consolidate time.".to_string(),
        }
    };

    let report = ConsolidatedComparisonReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed: request.seed,
        benchmark,
        ablation,
        external_baseline,
        external_comparison_json,
    };

    write_consolidated_json_report(request.json_out, &report)?;

    let markdown = render_consolidated_markdown(&report);
    fs::write(request.markdown_out, markdown).with_context(|| {
        format!(
            "failed to write consolidated markdown report: {}",
            request.markdown_out.display()
        )
    })?;

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

    Ok(())
}
