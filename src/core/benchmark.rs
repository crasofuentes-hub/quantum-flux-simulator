use crate::core::analysis::{
    analyze_file_with_seed, FileAnalysis, ANALYSIS_VERSION, REPORT_SCHEMA_VERSION,
};
use crate::core::baseline::{compute_structural_baseline, StructuralBaseline};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkEntry {
    pub path: String,
    pub expected_class: String,
    pub detected_class: String,
    pub class_match: bool,
    pub baseline: StructuralBaseline,
    pub stability_score: f64,
    pub singularity_risk: f64,
    pub collapse_probability: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkAggregate {
    pub files_analyzed: usize,
    pub class_accuracy: f64,
    pub mean_baseline_risk: f64,
    pub mean_model_risk: f64,
    pub mean_baseline_stability: f64,
    pub mean_model_stability: f64,
    pub mean_collapse_probability: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub report_schema_version: String,
    pub analysis_version: String,
    pub seed: u64,
    pub aggregate: BenchmarkAggregate,
    pub entries: Vec<BenchmarkEntry>,
}

pub fn run_synthetic_benchmark(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    seed: u64,
) -> Result<BenchmarkReport> {
    if !input_dir.exists() {
        bail!(
            "benchmark input_dir does not exist: {}",
            input_dir.display()
        );
    }
    if !input_dir.is_dir() {
        bail!(
            "benchmark input_dir is not a directory: {}",
            input_dir.display()
        );
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(input_dir)
        .with_context(|| format!("failed to read benchmark dir: {}", input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if ext == "py" {
                files.push(path);
            }
        }
    }

    files.sort();

    let mut analyses: Vec<(FileAnalysis, StructuralBaseline, String)> = Vec::new();

    for path in files {
        let expected = infer_expected_class(&path);
        let analysis = analyze_file_with_seed(
            &path,
            quantum_noise,
            relativistic_beta,
            target_temp_kelvin,
            None,
            seed,
        )?;
        let baseline = compute_structural_baseline(&analysis);
        analyses.push((analysis, baseline, expected));
    }

    let mut entries = Vec::new();
    let mut class_matches = 0usize;
    let mut baseline_risk_sum = 0.0;
    let mut model_risk_sum = 0.0;
    let mut baseline_stability_sum = 0.0;
    let mut model_stability_sum = 0.0;
    let mut collapse_sum = 0.0;

    for (analysis, baseline, expected) in analyses {
        let detected = format!("{:?}", analysis.algorithm_class).to_ascii_lowercase();
        let class_match = detected == expected;
        if class_match {
            class_matches += 1;
        }

        baseline_risk_sum += baseline.baseline_risk;
        model_risk_sum += analysis.singularity_risk;
        baseline_stability_sum += baseline.baseline_stability;
        model_stability_sum += analysis.stability_score;
        collapse_sum += analysis.solver_summary.collapse_probability;

        entries.push(BenchmarkEntry {
            path: analysis.path.clone(),
            expected_class: expected,
            detected_class: detected,
            class_match,
            baseline,
            stability_score: analysis.stability_score,
            singularity_risk: analysis.singularity_risk,
            collapse_probability: analysis.solver_summary.collapse_probability,
        });
    }

    let n = entries.len().max(1) as f64;

    Ok(BenchmarkReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed,
        aggregate: BenchmarkAggregate {
            files_analyzed: entries.len(),
            class_accuracy: class_matches as f64 / n,
            mean_baseline_risk: baseline_risk_sum / n,
            mean_model_risk: model_risk_sum / n,
            mean_baseline_stability: baseline_stability_sum / n,
            mean_model_stability: model_stability_sum / n,
            mean_collapse_probability: collapse_sum / n,
        },
        entries,
    })
}

fn infer_expected_class(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if name.contains("crypto") {
        "crypto".to_string()
    } else if name.contains("numerical") {
        "numerical".to_string()
    } else if name.contains("ml") {
        "ml".to_string()
    } else {
        "general".to_string()
    }
}
