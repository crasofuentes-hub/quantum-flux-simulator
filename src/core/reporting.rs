use crate::core::analysis::FileAnalysis;
use crate::core::benchmark::{AblationReport, BenchmarkReport};
use crate::core::solver::BatchAggregateSummary;
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn write_json_report(path: &Path, analysis: &FileAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(analysis).context("failed to serialize JSON report")?;
    fs::write(path, json)
        .with_context(|| format!("failed to write JSON report: {}", path.display()))?;
    Ok(())
}

pub fn write_batch_json_report(path: &Path, payload: &BatchReport) -> Result<()> {
    let json =
        serde_json::to_string_pretty(payload).context("failed to serialize batch JSON report")?;
    fs::write(path, json)
        .with_context(|| format!("failed to write batch JSON report: {}", path.display()))?;
    Ok(())
}

pub fn write_consolidated_json_report(
    path: &Path,
    payload: &ConsolidatedComparisonReport,
) -> Result<()> {
    let json = serde_json::to_string_pretty(payload)
        .context("failed to serialize consolidated comparison JSON report")?;
    fs::write(path, json).with_context(|| {
        format!(
            "failed to write consolidated comparison JSON report: {}",
            path.display()
        )
    })?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct BatchReport {
    pub report_schema_version: String,
    pub analysis_version: String,
    pub seed: u64,
    pub aggregate: BatchAggregateSummary,
    pub files: Vec<FileAnalysis>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalBaselineReference {
    pub name: String,
    pub integrated_automatically: bool,
    pub asset_path: String,
    pub doc_path: String,
    pub status: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalComparisonSnapshot {
    pub source_path: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemgrepSummarySnapshot {
    pub source_path: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsolidatedComparisonReport {
    pub report_schema_version: String,
    pub analysis_version: String,
    pub seed: u64,
    pub benchmark: BenchmarkReport,
    pub ablation: AblationReport,
    pub external_baseline: ExternalBaselineReference,
    pub external_comparison_json: Option<ExternalComparisonSnapshot>,
    pub semgrep_summary_json: Option<SemgrepSummarySnapshot>,
}

pub fn try_read_external_comparison_json(
    path: &Path,
) -> Result<Option<ExternalComparisonSnapshot>> {
    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read external comparison JSON: {}",
            path.display()
        )
    })?;

    let payload: Value = serde_json::from_str(&text).with_context(|| {
        format!(
            "failed to parse external comparison JSON: {}",
            path.display()
        )
    })?;

    Ok(Some(ExternalComparisonSnapshot {
        source_path: path.to_string_lossy().replace('\\', "/"),
        payload,
    }))
}

pub fn try_read_semgrep_summary_json(path: &Path) -> Result<Option<SemgrepSummarySnapshot>> {
    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read Semgrep summary JSON: {}", path.display()))?;

    let payload: Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse Semgrep summary JSON: {}", path.display()))?;

    Ok(Some(SemgrepSummarySnapshot {
        source_path: path.to_string_lossy().replace('\\', "/"),
        payload,
    }))
}

pub fn render_consolidated_markdown(report: &ConsolidatedComparisonReport) -> String {
    let benchmark = &report.benchmark.aggregate;
    let mut out = String::new();

    out.push_str("# Consolidated comparison report\n\n");
    out.push_str(&format!(
        "- report_schema_version: {}\n",
        report.report_schema_version
    ));
    out.push_str(&format!(
        "- analysis_version: {}\n",
        report.analysis_version
    ));
    out.push_str(&format!("- seed: {}\n\n", report.seed));

    out.push_str("## Internal benchmark summary\n\n");
    out.push_str("| metric | value |\n");
    out.push_str("|---|---:|\n");
    out.push_str(&format!(
        "| files_analyzed | {} |\n",
        benchmark.files_analyzed
    ));
    out.push_str(&format!(
        "| class_accuracy | {:.6} |\n",
        benchmark.class_accuracy
    ));
    out.push_str(&format!(
        "| mean_baseline_risk | {:.6} |\n",
        benchmark.mean_baseline_risk
    ));
    out.push_str(&format!(
        "| mean_model_risk | {:.6} |\n",
        benchmark.mean_model_risk
    ));
    out.push_str(&format!(
        "| mean_baseline_stability | {:.6} |\n",
        benchmark.mean_baseline_stability
    ));
    out.push_str(&format!(
        "| mean_model_stability | {:.6} |\n",
        benchmark.mean_model_stability
    ));
    out.push_str(&format!(
        "| mean_collapse_probability | {:.6} |\n",
        benchmark.mean_collapse_probability
    ));

    out.push_str("\n## Ablation summary\n\n");
    out.push_str("| variant | files_analyzed | class_accuracy | mean_stability_score | mean_singularity_risk | mean_collapse_probability |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|\n");
    for row in &report.ablation.aggregate {
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

    out.push_str("\n## External baseline status\n\n");
    out.push_str("| field | value |\n");
    out.push_str("|---|---|\n");
    out.push_str(&format!("| name | {} |\n", report.external_baseline.name));
    out.push_str(&format!(
        "| integrated_automatically | {} |\n",
        report.external_baseline.integrated_automatically
    ));
    out.push_str(&format!(
        "| asset_path | {} |\n",
        report.external_baseline.asset_path
    ));
    out.push_str(&format!(
        "| doc_path | {} |\n",
        report.external_baseline.doc_path
    ));
    out.push_str(&format!(
        "| status | {} |\n",
        report.external_baseline.status
    ));
    out.push_str(&format!("| notes | {} |\n", report.external_baseline.notes));

    out.push_str("\n## Radon comparison ingestion\n\n");
    if let Some(snapshot) = &report.external_comparison_json {
        out.push_str(&format!(
            "- Radon comparison JSON loaded from: {}\n",
            snapshot.source_path
        ));
    } else {
        out.push_str("- Radon comparison JSON not loaded\n");
    }

    out.push_str("\n## Semgrep summary ingestion\n\n");
    if let Some(snapshot) = &report.semgrep_summary_json {
        out.push_str(&format!(
            "- Semgrep summary JSON loaded from: {}\n",
            snapshot.source_path
        ));
    } else {
        out.push_str("- Semgrep summary JSON not loaded\n");
    }

    out.push_str("\n## Interpretation boundary\n\n");
    out.push_str("This consolidated report joins the internal structural baseline, the current effective flux-sim model, and the automated internal ablation outputs.\n");
    out.push_str("When available, it also embeds externally generated comparison artifacts such as Radon comparison JSON and Semgrep summary JSON.\n");
    out.push_str("Rust still does not execute those external tools directly; it only ingests their artifacts when present.\n");

    out
}

pub fn print_text_summary(analysis: &FileAnalysis) {
    println!("flux-sim analysis OK");
    println!(
        "schema_version={}",
        analysis.run_metadata.report_schema_version
    );
    println!(
        "analysis_version={}",
        analysis.run_metadata.analysis_version
    );
    println!("seed={}", analysis.run_metadata.seed);
    println!(
        "input_fingerprint={}",
        analysis.run_metadata.input_fingerprint
    );
    println!("path={}", analysis.path);
    println!("language={}", analysis.language);
    println!("algorithm_class={:?}", analysis.algorithm_class);
    println!("functions={}", analysis.functions);
    println!("fors={}", analysis.fors);
    println!("whiles={}", analysis.whiles);
    println!("max_nesting={}", analysis.max_nesting);
    println!("has_recursion={}", analysis.has_recursion);
    println!("hotspots={}", analysis.hotspots.join(", "));
    println!(
        "critical_blocks={}",
        analysis.intermediate_model.critical_blocks.len()
    );
    println!(
        "information_channels={}",
        analysis.intermediate_model.information_channels.join(", ")
    );
    println!(
        "structural_complexity={}",
        analysis.intermediate_model.structural_complexity
    );
    println!(
        "decoherence_rate={}",
        analysis.physical_model.decoherence_rate
    );
    println!(
        "runtime_dilation={}",
        analysis.physical_model.effective_runtime_dilation
    );
    println!(
        "von_neumann_entropy={}",
        analysis.physical_model.von_neumann_entropy
    );
    println!(
        "global_constraint_penalty={}",
        analysis.physical_model.global_constraint_penalty
    );
    println!(
        "recommended_qubit_budget={}",
        analysis.physical_model.recommended_qubit_budget
    );
    println!("mc_samples={}", analysis.solver_summary.samples);
    println!("mc_seed={}", analysis.solver_summary.seed);
    println!("mean_stress={}", analysis.solver_summary.mean_stress);
    println!(
        "stress_variance={}",
        analysis.solver_summary.stress_variance
    );
    println!("p05_stress={}", analysis.solver_summary.p05_stress);
    println!("p50_stress={}", analysis.solver_summary.p50_stress);
    println!("p95_stress={}", analysis.solver_summary.p95_stress);
    println!(
        "collapse_probability={}",
        analysis.solver_summary.collapse_probability
    );
    println!(
        "computational_singularity_risk={}",
        analysis.solver_summary.computational_singularity_risk
    );
    println!(
        "solver_stability_score={}",
        analysis.solver_summary.solver_stability_score
    );
    println!("quantum_noise={}", analysis.quantum_noise);
    println!("relativistic_beta={}", analysis.relativistic_beta);
    println!("target_temp_kelvin={}", analysis.target_temp_kelvin);
    println!("stability_score={}", analysis.stability_score);
    println!("singularity_risk={}", analysis.singularity_risk);
    println!("recommendation={}", analysis.recommendation);
}
