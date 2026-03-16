use crate::core::analysis::FileAnalysis;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StructuralBaseline {
    pub structural_score: f64,
    pub baseline_risk: f64,
    pub baseline_stability: f64,
}

pub fn compute_structural_baseline(analysis: &FileAnalysis) -> StructuralBaseline {
    let raw = analysis.functions as f64 * 1.0
        + analysis.fors as f64 * 1.5
        + analysis.whiles as f64 * 1.8
        + analysis.max_nesting as f64 * 2.0
        + if analysis.has_recursion { 3.0 } else { 0.0 }
        + analysis.hotspots.len() as f64 * 0.75;

    let baseline_risk = (raw / 20.0).clamp(0.0, 1.0);
    let baseline_stability = (100.0 - raw * 4.0).clamp(0.0, 100.0);

    StructuralBaseline {
        structural_score: raw,
        baseline_risk,
        baseline_stability,
    }
}
