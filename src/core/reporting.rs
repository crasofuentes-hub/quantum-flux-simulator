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
    println!("quantum_noise={}", analysis.quantum_noise);
    println!("relativistic_beta={}", analysis.relativistic_beta);
    println!("target_temp_kelvin={}", analysis.target_temp_kelvin);
    println!("stability_score={}", analysis.stability_score);
    println!("singularity_risk={}", analysis.singularity_risk);
    println!("recommendation={}", analysis.recommendation);
}
