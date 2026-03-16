use crate::core::physics::{build_effective_physical_model, EffectivePhysicalModel};
use crate::core::solver::{run_effective_solver, MonteCarloSummary};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub const REPORT_SCHEMA_VERSION: &str = "0.2.0";
pub const ANALYSIS_VERSION: &str = "0.2.0";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlgorithmClass {
    Crypto,
    Numerical,
    Ml,
    General,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Function,
    Loop,
    CryptoPrimitive,
    NumericalKernel,
    MlKernel,
}

#[derive(Debug, Clone, Serialize)]
pub struct CriticalBlock {
    pub name: String,
    pub kind: BlockKind,
    pub estimated_cost: f64,
    pub estimated_logical_qubits: u32,
    pub information_density: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntermediateModel {
    pub critical_blocks: Vec<CriticalBlock>,
    pub hotspots: Vec<String>,
    pub information_channels: Vec<String>,
    pub structural_complexity: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunMetadata {
    pub report_schema_version: String,
    pub analysis_version: String,
    pub seed: u64,
    pub input_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileAnalysis {
    pub run_metadata: RunMetadata,
    pub path: String,
    pub language: String,
    pub algorithm_class: AlgorithmClass,
    pub functions: usize,
    pub fors: usize,
    pub whiles: usize,
    pub max_nesting: usize,
    pub has_recursion: bool,
    pub crypto_hits: Vec<String>,
    pub numerical_hits: Vec<String>,
    pub ml_hits: Vec<String>,
    pub hotspots: Vec<String>,
    pub intermediate_model: IntermediateModel,
    pub physical_model: EffectivePhysicalModel,
    pub solver_summary: MonteCarloSummary,
    pub quantum_noise: f64,
    pub relativistic_beta: f64,
    pub target_temp_kelvin: f64,
    pub stability_score: f64,
    pub singularity_risk: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
struct AnalysisSignals {
    functions: usize,
    fors: usize,
    whiles: usize,
    max_nesting: usize,
    has_recursion: bool,
    crypto_hits: Vec<String>,
    numerical_hits: Vec<String>,
    ml_hits: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct AnalysisAblation {
    pub include_lindblad: bool,
    pub include_global_constraint: bool,
    pub include_relativistic_factor: bool,
    pub include_solver: bool,
}

impl AnalysisAblation {
    pub fn full_model() -> Self {
        Self {
            include_lindblad: true,
            include_global_constraint: true,
            include_relativistic_factor: true,
            include_solver: true,
        }
    }

    pub fn no_lindblad() -> Self {
        Self {
            include_lindblad: false,
            include_global_constraint: true,
            include_relativistic_factor: true,
            include_solver: true,
        }
    }

    pub fn no_global_constraint() -> Self {
        Self {
            include_lindblad: true,
            include_global_constraint: false,
            include_relativistic_factor: true,
            include_solver: true,
        }
    }

    pub fn no_relativistic_factor() -> Self {
        Self {
            include_lindblad: true,
            include_global_constraint: true,
            include_relativistic_factor: false,
            include_solver: true,
        }
    }

    pub fn no_solver() -> Self {
        Self {
            include_lindblad: true,
            include_global_constraint: true,
            include_relativistic_factor: true,
            include_solver: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EffectiveScoreEstimate {
    pub stress: f64,
    pub heuristic_stability: f64,
    pub stability_score: f64,
    pub singularity_risk: f64,
    pub collapse_probability: f64,
}

pub fn analyze_file(
    path: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    class_override: Option<AlgorithmClass>,
) -> Result<FileAnalysis> {
    analyze_file_with_seed(
        path,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        class_override,
        42,
    )
}

pub fn analyze_file_with_seed(
    path: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    class_override: Option<AlgorithmClass>,
    seed: u64,
) -> Result<FileAnalysis> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read source file: {}", path.display()))?;

    let language = detect_language(path);
    let input_fingerprint = compute_input_fingerprint(
        path,
        &text,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        seed,
    );

    let signals = collect_signals(&text, &language);

    let algorithm_class = class_override.unwrap_or_else(|| {
        classify(
            &signals.crypto_hits,
            &signals.numerical_hits,
            &signals.ml_hits,
        )
    });

    let structural_complexity = signals.functions as f64
        + (signals.fors as f64 * 1.4)
        + (signals.whiles as f64 * 1.7)
        + (signals.max_nesting as f64 * 1.8)
        + if signals.has_recursion { 2.0 } else { 0.0 };

    let hotspots = build_hotspots(&signals);
    let critical_blocks = build_critical_blocks(&signals, structural_complexity);

    let information_channels = build_information_channels(
        &algorithm_class,
        &signals.crypto_hits,
        &signals.numerical_hits,
        &signals.ml_hits,
    );

    let intermediate_model = IntermediateModel {
        critical_blocks,
        hotspots: hotspots.clone(),
        information_channels,
        structural_complexity,
    };

    let physical_model = build_effective_physical_model(
        &intermediate_model.critical_blocks,
        algorithm_class,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
    );

    let solver_summary = run_effective_solver(
        &physical_model,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        seed,
    );

    let estimate = estimate_effective_scores(
        &FileAnalysis {
            run_metadata: RunMetadata {
                report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
                analysis_version: ANALYSIS_VERSION.to_string(),
                seed,
                input_fingerprint: input_fingerprint.clone(),
            },
            path: path.display().to_string(),
            language: language.clone(),
            algorithm_class,
            functions: signals.functions,
            fors: signals.fors,
            whiles: signals.whiles,
            max_nesting: signals.max_nesting,
            has_recursion: signals.has_recursion,
            crypto_hits: signals.crypto_hits.clone(),
            numerical_hits: signals.numerical_hits.clone(),
            ml_hits: signals.ml_hits.clone(),
            hotspots: hotspots.clone(),
            intermediate_model: intermediate_model.clone(),
            physical_model: physical_model.clone(),
            solver_summary: solver_summary.clone(),
            quantum_noise,
            relativistic_beta,
            target_temp_kelvin,
            stability_score: 0.0,
            singularity_risk: 0.0,
            recommendation: String::new(),
        },
        AnalysisAblation::full_model(),
    );

    let recommendation = build_recommendation(
        algorithm_class,
        solver_summary.collapse_probability,
        estimate.singularity_risk,
        estimate.stability_score,
        physical_model.recommended_qubit_budget,
    );

    Ok(FileAnalysis {
        run_metadata: RunMetadata {
            report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
            analysis_version: ANALYSIS_VERSION.to_string(),
            seed,
            input_fingerprint,
        },
        path: path.display().to_string(),
        language,
        algorithm_class,
        functions: signals.functions,
        fors: signals.fors,
        whiles: signals.whiles,
        max_nesting: signals.max_nesting,
        has_recursion: signals.has_recursion,
        crypto_hits: signals.crypto_hits,
        numerical_hits: signals.numerical_hits,
        ml_hits: signals.ml_hits,
        hotspots,
        intermediate_model,
        physical_model,
        solver_summary,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        stability_score: estimate.stability_score,
        singularity_risk: estimate.singularity_risk,
        recommendation,
    })
}

pub fn estimate_effective_scores(
    analysis: &FileAnalysis,
    ablation: AnalysisAblation,
) -> EffectiveScoreEstimate {
    let domain_pressure = match analysis.algorithm_class {
        AlgorithmClass::Crypto => 1.20,
        AlgorithmClass::Numerical => 1.00,
        AlgorithmClass::Ml => 1.10,
        AlgorithmClass::General => 0.85,
    };

    let hotspot_pressure = (analysis.hotspots.len() as f64) * 0.18;
    let block_pressure = (analysis.intermediate_model.critical_blocks.len() as f64) * 0.12;

    let mut physical_pressure = 0.0;
    if ablation.include_lindblad {
        physical_pressure += analysis.physical_model.decoherence_rate * 6.0;
        physical_pressure += analysis.physical_model.von_neumann_entropy * 0.75;
    }
    if ablation.include_global_constraint {
        physical_pressure += analysis.physical_model.global_constraint_penalty * 0.08;
    }
    if ablation.include_relativistic_factor {
        physical_pressure += (analysis.physical_model.effective_runtime_dilation - 1.0) * 2.5;
    }

    let mut solver_pressure = 0.0;
    let collapse_probability = if ablation.include_solver {
        analysis.solver_summary.collapse_probability
    } else {
        0.0
    };

    if ablation.include_solver {
        solver_pressure += analysis.solver_summary.mean_stress * 1.8;
        solver_pressure += analysis.solver_summary.collapse_probability * 4.5;
        solver_pressure += analysis.solver_summary.computational_singularity_risk * 5.5;
    }

    let mut stress = analysis.quantum_noise * 18.0
        + (analysis.target_temp_kelvin / 300.0)
        + analysis.intermediate_model.structural_complexity / 20.0
        + hotspot_pressure
        + block_pressure
        + physical_pressure
        + solver_pressure;

    if ablation.include_relativistic_factor {
        stress += analysis.relativistic_beta * 9.0;
    }

    stress *= domain_pressure;

    let heuristic_stability = (100.0 - stress * 8.0).clamp(0.0, 100.0);

    let stability_score = if ablation.include_solver {
        ((heuristic_stability * 0.45) + (analysis.solver_summary.solver_stability_score * 0.55))
            .clamp(0.0, 100.0)
    } else {
        heuristic_stability
    };

    let singularity_risk = if ablation.include_solver {
        ((stress / 12.0) * 0.40 + analysis.solver_summary.computational_singularity_risk * 0.60)
            .clamp(0.0, 1.0)
    } else {
        (stress / 12.0).clamp(0.0, 1.0)
    };

    EffectiveScoreEstimate {
        stress,
        heuristic_stability,
        stability_score,
        singularity_risk,
        collapse_probability,
    }
}

fn collect_signals(text: &str, language: &str) -> AnalysisSignals {
    AnalysisSignals {
        functions: count_matches(text, &["fn ", "def ", "function "]),
        fors: count_matches(text, &["for "]),
        whiles: count_matches(text, &["while "]),
        max_nesting: estimate_max_nesting(text),
        has_recursion: detect_recursion(text, language),
        crypto_hits: detect_keywords(
            text,
            &["aes", "rsa", "ecc", "sha", "kyber", "lattice", "ed25519"],
        ),
        numerical_hits: detect_keywords(
            text,
            &[
                "matmul",
                "fft",
                "conv",
                "solve",
                "integrate",
                "jacobi",
                "rk4",
            ],
        ),
        ml_hits: detect_keywords(
            text,
            &[
                "gradient",
                "backprop",
                "optimizer",
                "relu",
                "softmax",
                "loss",
                "tensor",
            ],
        ),
    }
}

fn detect_language(path: &Path) -> String {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "py" => "python".to_string(),
        "rs" => "rust".to_string(),
        "cpp" | "cc" | "cxx" | "hpp" | "h" => "cpp".to_string(),
        "js" | "mjs" | "cjs" => "javascript".to_string(),
        _ => "unknown".to_string(),
    }
}

fn compute_input_fingerprint(
    path: &Path,
    text: &str,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    seed: u64,
) -> String {
    let mut hasher = DefaultHasher::new();
    path.display().to_string().hash(&mut hasher);
    text.hash(&mut hasher);
    quantum_noise.to_bits().hash(&mut hasher);
    relativistic_beta.to_bits().hash(&mut hasher);
    target_temp_kelvin.to_bits().hash(&mut hasher);
    seed.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn count_matches(text: &str, needles: &[&str]) -> usize {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .map(|needle| lower.matches(needle).count())
        .sum()
}

fn detect_keywords(text: &str, keywords: &[&str]) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    keywords
        .iter()
        .filter(|k| lower.contains(**k))
        .map(|k| (*k).to_string())
        .collect()
}

fn estimate_max_nesting(text: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;

    for ch in text.chars() {
        if ch == '{' {
            depth += 1;
            if depth > max_depth {
                max_depth = depth;
            }
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
        }
    }

    if max_depth == 0 {
        let mut indent_max = 0usize;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let spaces = line.chars().take_while(|c| *c == ' ').count();
            let level = spaces / 4;
            if level > indent_max {
                indent_max = level;
            }
        }
        indent_max
    } else {
        max_depth
    }
}

fn detect_recursion(text: &str, language: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();

    match language {
        "python" => {
            for line in &lines {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed.strip_prefix("def ") {
                    let name = rest.split('(').next().unwrap_or("").trim();
                    if !name.is_empty() {
                        let needle = format!("{name}(");
                        let count = text.matches(&needle).count();
                        if count > 1 {
                            return true;
                        }
                    }
                }
            }
            false
        }
        "rust" => {
            for line in &lines {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed.strip_prefix("fn ") {
                    let name = rest.split('(').next().unwrap_or("").trim();
                    if !name.is_empty() {
                        let needle = format!("{name}(");
                        let count = text.matches(&needle).count();
                        if count > 1 {
                            return true;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn classify(
    crypto_hits: &[String],
    numerical_hits: &[String],
    ml_hits: &[String],
) -> AlgorithmClass {
    if !crypto_hits.is_empty() {
        return AlgorithmClass::Crypto;
    }

    if !ml_hits.is_empty() {
        return AlgorithmClass::Ml;
    }

    if !numerical_hits.is_empty() {
        return AlgorithmClass::Numerical;
    }

    AlgorithmClass::General
}

fn build_hotspots(signals: &AnalysisSignals) -> Vec<String> {
    let mut hotspots = Vec::new();

    if signals.functions >= 2 {
        hotspots.push("multi-function control surface".to_string());
    }
    if signals.fors + signals.whiles >= 2 {
        hotspots.push("iterative pressure".to_string());
    }
    if signals.max_nesting >= 2 {
        hotspots.push("nested control depth".to_string());
    }
    if signals.has_recursion {
        hotspots.push("recursive feedback path".to_string());
    }
    if !signals.crypto_hits.is_empty() {
        hotspots.push("cryptographic primitive concentration".to_string());
    }
    if !signals.numerical_hits.is_empty() {
        hotspots.push("numerical kernel concentration".to_string());
    }
    if !signals.ml_hits.is_empty() {
        hotspots.push("gradient or training signal concentration".to_string());
    }

    hotspots
}

fn build_critical_blocks(
    signals: &AnalysisSignals,
    structural_complexity: f64,
) -> Vec<CriticalBlock> {
    let mut blocks = Vec::new();

    if signals.functions > 0 {
        blocks.push(CriticalBlock {
            name: "function-surface".to_string(),
            kind: BlockKind::Function,
            estimated_cost: structural_complexity * 0.25,
            estimated_logical_qubits: ((signals.functions as f64 * 6.0).ceil() as u32).max(4),
            information_density: 0.45,
        });
    }

    if signals.fors + signals.whiles > 0 {
        blocks.push(CriticalBlock {
            name: "iterative-core".to_string(),
            kind: BlockKind::Loop,
            estimated_cost: ((signals.fors + signals.whiles) as f64) * 2.4,
            estimated_logical_qubits: (((signals.fors + signals.whiles) as f64 * 8.0).ceil()
                as u32)
                .max(6),
            information_density: 0.62,
        });
    }

    if !signals.crypto_hits.is_empty() {
        blocks.push(CriticalBlock {
            name: "crypto-core".to_string(),
            kind: BlockKind::CryptoPrimitive,
            estimated_cost: (signals.crypto_hits.len() as f64) * 3.2,
            estimated_logical_qubits: ((signals.crypto_hits.len() as f64 * 16.0).ceil() as u32)
                .max(16),
            information_density: 0.88,
        });
    }

    if !signals.numerical_hits.is_empty() {
        blocks.push(CriticalBlock {
            name: "numerical-core".to_string(),
            kind: BlockKind::NumericalKernel,
            estimated_cost: (signals.numerical_hits.len() as f64) * 2.8,
            estimated_logical_qubits: ((signals.numerical_hits.len() as f64 * 12.0).ceil() as u32)
                .max(12),
            information_density: 0.81,
        });
    }

    if !signals.ml_hits.is_empty() {
        blocks.push(CriticalBlock {
            name: "ml-core".to_string(),
            kind: BlockKind::MlKernel,
            estimated_cost: (signals.ml_hits.len() as f64) * 2.6,
            estimated_logical_qubits: ((signals.ml_hits.len() as f64 * 14.0).ceil() as u32).max(14),
            information_density: 0.84,
        });
    }

    blocks
}

fn build_information_channels(
    algorithm_class: &AlgorithmClass,
    crypto_hits: &[String],
    numerical_hits: &[String],
    ml_hits: &[String],
) -> Vec<String> {
    let mut channels = Vec::new();

    channels.push("control-flow".to_string());

    match algorithm_class {
        AlgorithmClass::Crypto => channels.push("key-material".to_string()),
        AlgorithmClass::Numerical => channels.push("state-vector".to_string()),
        AlgorithmClass::Ml => channels.push("gradient-flow".to_string()),
        AlgorithmClass::General => channels.push("general-dataflow".to_string()),
    }

    if !crypto_hits.is_empty() {
        channels.push("crypto-transform".to_string());
    }
    if !numerical_hits.is_empty() {
        channels.push("numerical-propagation".to_string());
    }
    if !ml_hits.is_empty() {
        channels.push("parameter-update".to_string());
    }

    channels
}

fn build_recommendation(
    algorithm_class: AlgorithmClass,
    collapse_probability: f64,
    singularity_risk: f64,
    stability_score: f64,
    recommended_qubit_budget: u32,
) -> String {
    match algorithm_class {
        AlgorithmClass::Crypto if singularity_risk > 0.45 || collapse_probability > 0.35 => {
            format!(
                "Elevated instability detected in crypto-oriented code. Reduce branching hotspots and evaluate lattice-based migration margin. Recommended effective qubit budget: {}.",
                recommended_qubit_budget
            )
        }
        AlgorithmClass::Crypto => {
            format!(
                "Crypto-oriented profile is acceptable. Preserve deterministic paths and review high-cost loops around key operations. Recommended effective qubit budget: {}.",
                recommended_qubit_budget
            )
        }
        AlgorithmClass::Numerical if stability_score < 70.0 => {
            format!(
                "Numerical workload shows moderate instability. Reduce nesting depth and isolate iterative kernels for tighter control. Recommended effective qubit budget: {}.",
                recommended_qubit_budget
            )
        }
        AlgorithmClass::Numerical => {
            format!(
                "Numerical workload is stable enough for the current regime. Focus on kernel extraction and loop regularity. Recommended effective qubit budget: {}.",
                recommended_qubit_budget
            )
        }
        AlgorithmClass::Ml => {
            format!(
                "ML-oriented structure detected. Prioritize matrix kernels, gradient flow hotspots, and bounded iterative behavior. Recommended effective qubit budget: {}.",
                recommended_qubit_budget
            )
        }
        AlgorithmClass::General => {
            format!(
                "General-purpose workload detected. Improve structural regularity before running more aggressive simulation layers. Recommended effective qubit budget: {}.",
                recommended_qubit_budget
            )
        }
    }
}
