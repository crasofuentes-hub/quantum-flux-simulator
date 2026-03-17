use crate::core::analysis::{
    analyze_file_with_seed, estimate_effective_scores, AnalysisAblation, FileAnalysis,
    ANALYSIS_VERSION, REPORT_SCHEMA_VERSION,
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AblationVariant {
    StructuralBaseline,
    FullModel,
    NoLindblad,
    NoGlobalConstraint,
    NoRelativisticFactor,
    NoSolver,
}

impl AblationVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StructuralBaseline => "structural_baseline",
            Self::FullModel => "full_model",
            Self::NoLindblad => "no_lindblad",
            Self::NoGlobalConstraint => "no_global_constraint",
            Self::NoRelativisticFactor => "no_relativistic_factor",
            Self::NoSolver => "no_solver",
        }
    }

    fn all() -> [Self; 6] {
        [
            Self::StructuralBaseline,
            Self::FullModel,
            Self::NoLindblad,
            Self::NoGlobalConstraint,
            Self::NoRelativisticFactor,
            Self::NoSolver,
        ]
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationVariantEntry {
    pub variant: AblationVariant,
    pub stability_score: f64,
    pub singularity_risk: f64,
    pub collapse_probability: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationEntry {
    pub path: String,
    pub expected_class: String,
    pub detected_class: String,
    pub class_match: bool,
    pub baseline: StructuralBaseline,
    pub variants: Vec<AblationVariantEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationVariantAggregate {
    pub variant: AblationVariant,
    pub files_analyzed: usize,
    pub class_accuracy: f64,
    pub mean_stability_score: f64,
    pub mean_singularity_risk: f64,
    pub mean_collapse_probability: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationReport {
    pub report_schema_version: String,
    pub analysis_version: String,
    pub seed: u64,
    pub aggregate: Vec<AblationVariantAggregate>,
    pub entries: Vec<AblationEntry>,
}

pub fn run_synthetic_benchmark(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    seed: u64,
) -> Result<BenchmarkReport> {
    let analyses = collect_benchmark_inputs(
        input_dir,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        seed,
    )?;

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

pub fn run_synthetic_ablation(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    seed: u64,
) -> Result<AblationReport> {
    let analyses = collect_benchmark_inputs(
        input_dir,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        seed,
    )?;

    let mut entries = Vec::new();

    for (analysis, baseline, expected) in &analyses {
        let detected = format!("{:?}", analysis.algorithm_class).to_ascii_lowercase();
        let class_match = detected == *expected;

        let mut variants = Vec::new();
        for variant in AblationVariant::all() {
            let metrics = compute_variant_metrics(variant, analysis, baseline);
            variants.push(AblationVariantEntry {
                variant,
                stability_score: metrics.stability_score,
                singularity_risk: metrics.singularity_risk,
                collapse_probability: metrics.collapse_probability,
            });
        }

        entries.push(AblationEntry {
            path: analysis.path.clone(),
            expected_class: expected.clone(),
            detected_class: detected,
            class_match,
            baseline: baseline.clone(),
            variants,
        });
    }

    let mut aggregate = Vec::new();
    for variant in AblationVariant::all() {
        let mut class_matches = 0usize;
        let mut stability_sum = 0.0;
        let mut risk_sum = 0.0;
        let mut collapse_sum = 0.0;

        for entry in &entries {
            if entry.class_match {
                class_matches += 1;
            }

            let variant_entry = entry
                .variants
                .iter()
                .find(|candidate| candidate.variant == variant)
                .expect("all variants must exist on every entry");

            stability_sum += variant_entry.stability_score;
            risk_sum += variant_entry.singularity_risk;
            collapse_sum += variant_entry.collapse_probability;
        }

        let n = entries.len().max(1) as f64;
        aggregate.push(AblationVariantAggregate {
            variant,
            files_analyzed: entries.len(),
            class_accuracy: class_matches as f64 / n,
            mean_stability_score: stability_sum / n,
            mean_singularity_risk: risk_sum / n,
            mean_collapse_probability: collapse_sum / n,
        });
    }

    Ok(AblationReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed,
        aggregate,
        entries,
    })
}

pub fn render_ablation_markdown(report: &AblationReport) -> String {
    let mut out = String::new();
    out.push_str("# Synthetic ablation report\n\n");
    out.push_str(&format!(
        "- report_schema_version: {}\n",
        report.report_schema_version
    ));
    out.push_str(&format!(
        "- analysis_version: {}\n",
        report.analysis_version
    ));
    out.push_str(&format!("- seed: {}\n\n", report.seed));

    out.push_str("| variant | files_analyzed | class_accuracy | mean_stability_score | mean_singularity_risk | mean_collapse_probability |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|\n");

    for row in &report.aggregate {
        out.push_str(&format!(
            "| {} | {} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
            row.variant.as_str(),
            row.files_analyzed,
            row.class_accuracy,
            row.mean_stability_score,
            row.mean_singularity_risk,
            row.mean_collapse_probability
        ));
    }

    out.push_str("\n## Interpretation boundary\n\n");
    out.push_str(
        "These ablations are comparative views over the current effective scoring pipeline.\n",
    );
    out.push_str("They remove selected contribution terms from the effective analysis stack, but they do not claim a full alternative physical re-simulation for each variant.\n");

    out
}

fn compute_variant_metrics(
    variant: AblationVariant,
    analysis: &FileAnalysis,
    baseline: &StructuralBaseline,
) -> AblationVariantEntry {
    match variant {
        AblationVariant::StructuralBaseline => AblationVariantEntry {
            variant,
            stability_score: baseline.baseline_stability,
            singularity_risk: baseline.baseline_risk,
            collapse_probability: 0.0,
        },
        AblationVariant::FullModel => {
            let estimate = estimate_effective_scores(analysis, AnalysisAblation::full_model());
            AblationVariantEntry {
                variant,
                stability_score: estimate.stability_score,
                singularity_risk: estimate.singularity_risk,
                collapse_probability: estimate.collapse_probability,
            }
        }
        AblationVariant::NoLindblad => {
            let estimate = estimate_effective_scores(analysis, AnalysisAblation::no_lindblad());
            AblationVariantEntry {
                variant,
                stability_score: estimate.stability_score,
                singularity_risk: estimate.singularity_risk,
                collapse_probability: estimate.collapse_probability,
            }
        }
        AblationVariant::NoGlobalConstraint => {
            let estimate =
                estimate_effective_scores(analysis, AnalysisAblation::no_global_constraint());
            AblationVariantEntry {
                variant,
                stability_score: estimate.stability_score,
                singularity_risk: estimate.singularity_risk,
                collapse_probability: estimate.collapse_probability,
            }
        }
        AblationVariant::NoRelativisticFactor => {
            let estimate =
                estimate_effective_scores(analysis, AnalysisAblation::no_relativistic_factor());
            AblationVariantEntry {
                variant,
                stability_score: estimate.stability_score,
                singularity_risk: estimate.singularity_risk,
                collapse_probability: estimate.collapse_probability,
            }
        }
        AblationVariant::NoSolver => {
            let estimate = estimate_effective_scores(analysis, AnalysisAblation::no_solver());
            AblationVariantEntry {
                variant,
                stability_score: estimate.stability_score,
                singularity_risk: estimate.singularity_risk,
                collapse_probability: estimate.collapse_probability,
            }
        }
    }
}

fn collect_benchmark_inputs(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    seed: u64,
) -> Result<Vec<(FileAnalysis, StructuralBaseline, String)>> {
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
            if ["py", "ts", "tsx"].contains(&ext.as_str()) {
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

    Ok(analyses)
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
