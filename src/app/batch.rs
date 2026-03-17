use crate::core::analysis::{
    analyze_file_with_seed, AlgorithmClass, ANALYSIS_VERSION, REPORT_SCHEMA_VERSION,
};
use crate::core::reporting::{write_batch_json_report, BatchReport};
use crate::core::solver::summarize_batch;
use crate::util::params::{parse_kelvin, parse_relativistic_fraction};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn execute_batch(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic: &str,
    target_temp: &str,
    json_out: &Path,
    class_override: Option<AlgorithmClass>,
    seed: u64,
) -> Result<()> {
    if !input_dir.exists() {
        bail!("input_dir does not exist: {}", input_dir.display());
    }
    if !input_dir.is_dir() {
        bail!("input_dir is not a directory: {}", input_dir.display());
    }

    let beta = parse_relativistic_fraction(relativistic)?;
    let kelvin = parse_kelvin(target_temp)?;

    let mut files = Vec::new();
    for entry in fs::read_dir(input_dir)
        .with_context(|| format!("failed to read dir: {}", input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if [
                "py", "rs", "cpp", "cc", "cxx", "js", "mjs", "cjs", "ts", "tsx",
            ]
            .contains(&ext.as_str())
            {
                files.push(path);
            }
        }
    }

    files.sort();

    let mut analyses = Vec::new();
    for path in files {
        analyses.push(analyze_file_with_seed(
            &path,
            quantum_noise,
            beta,
            kelvin,
            class_override,
            seed,
        )?);
    }

    let stabilities: Vec<f64> = analyses.iter().map(|a| a.stability_score).collect();
    let risks: Vec<f64> = analyses.iter().map(|a| a.singularity_risk).collect();
    let collapses: Vec<f64> = analyses
        .iter()
        .map(|a| a.solver_summary.collapse_probability)
        .collect();

    let aggregate = summarize_batch(&stabilities, &risks, &collapses);

    let report = BatchReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed,
        aggregate,
        files: analyses,
    };

    write_batch_json_report(json_out, &report)?;
    println!("flux-sim batch OK");
    println!("files_analyzed={}", report.aggregate.files_analyzed);
    println!(
        "mean_stability_score={}",
        report.aggregate.mean_stability_score
    );
    println!(
        "max_singularity_risk={}",
        report.aggregate.max_singularity_risk
    );
    println!(
        "mean_collapse_probability={}",
        report.aggregate.mean_collapse_probability
    );

    Ok(())
}
