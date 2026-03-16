use crate::core::analysis::FileAnalysis;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn write_json_report(path: &Path, analysis: &FileAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(analysis).context("failed to serialize JSON report")?;
    fs::write(path, json)
        .with_context(|| format!("failed to write JSON report: {}", path.display()))?;
    Ok(())
}

pub fn print_text_summary(analysis: &FileAnalysis) {
    println!("flux-sim analysis OK");
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
        "wheeler_dewitt_penalty={}",
        analysis.physical_model.wheeler_dewitt_penalty
    );
    println!(
        "recommended_qubit_budget={}",
        analysis.physical_model.recommended_qubit_budget
    );
    println!("mc_samples={}", analysis.solver_summary.samples);
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
