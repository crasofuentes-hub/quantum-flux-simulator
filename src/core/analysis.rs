use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlgorithmClass {
    Crypto,
    Numerical,
    Ml,
    General,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileAnalysis {
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
    pub quantum_noise: f64,
    pub relativistic_beta: f64,
    pub target_temp_kelvin: f64,
    pub stability_score: f64,
    pub singularity_risk: f64,
    pub recommendation: String,
}

pub fn analyze_file(
    path: &Path,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    class_override: Option<AlgorithmClass>,
) -> Result<FileAnalysis> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read source file: {}", path.display()))?;

    let language = detect_language(path);
    let functions = count_matches(&text, &["fn ", "def ", "function "]);
    let fors = count_matches(&text, &["for "]);
    let whiles = count_matches(&text, &["while "]);
    let max_nesting = estimate_max_nesting(&text);
    let has_recursion = detect_recursion(&text, &language);

    let crypto_hits = detect_keywords(
        &text,
        &["aes", "rsa", "ecc", "sha", "kyber", "lattice", "ed25519"],
    );
    let numerical_hits = detect_keywords(
        &text,
        &[
            "matmul",
            "fft",
            "conv",
            "solve",
            "integrate",
            "jacobi",
            "rk4",
        ],
    );
    let ml_hits = detect_keywords(
        &text,
        &[
            "gradient",
            "backprop",
            "optimizer",
            "relu",
            "softmax",
            "loss",
            "tensor",
        ],
    );

    let algorithm_class =
        class_override.unwrap_or_else(|| classify(&crypto_hits, &numerical_hits, &ml_hits));

    let complexity = functions as f64
        + (fors as f64 * 1.4)
        + (whiles as f64 * 1.7)
        + (max_nesting as f64 * 1.8)
        + if has_recursion { 2.0 } else { 0.0 };

    let domain_pressure = match algorithm_class {
        AlgorithmClass::Crypto => 1.20,
        AlgorithmClass::Numerical => 1.00,
        AlgorithmClass::Ml => 1.10,
        AlgorithmClass::General => 0.85,
    };

    let stress = (quantum_noise * 18.0
        + relativistic_beta * 9.0
        + (target_temp_kelvin / 300.0)
        + complexity / 20.0)
        * domain_pressure;

    let stability_score = (100.0 - stress * 8.0).clamp(0.0, 100.0);
    let singularity_risk = (stress / 12.0).clamp(0.0, 1.0);

    let recommendation = build_recommendation(algorithm_class, singularity_risk, stability_score);

    Ok(FileAnalysis {
        path: path.display().to_string(),
        language,
        algorithm_class,
        functions,
        fors,
        whiles,
        max_nesting,
        has_recursion,
        crypto_hits,
        numerical_hits,
        ml_hits,
        quantum_noise,
        relativistic_beta,
        target_temp_kelvin,
        stability_score,
        singularity_risk,
        recommendation,
    })
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

fn build_recommendation(
    algorithm_class: AlgorithmClass,
    singularity_risk: f64,
    stability_score: f64,
) -> String {
    match algorithm_class {
        AlgorithmClass::Crypto if singularity_risk > 0.45 => {
            "Elevated instability detected in crypto-oriented code. Reduce branching hotspots and evaluate lattice-based migration margin.".to_string()
        }
        AlgorithmClass::Crypto => {
            "Crypto-oriented profile is acceptable. Preserve deterministic paths and review high-cost loops around key operations.".to_string()
        }
        AlgorithmClass::Numerical if stability_score < 70.0 => {
            "Numerical workload shows moderate instability. Reduce nesting depth and isolate iterative kernels for tighter control.".to_string()
        }
        AlgorithmClass::Numerical => {
            "Numerical workload is stable enough for the current regime. Focus on kernel extraction and loop regularity.".to_string()
        }
        AlgorithmClass::Ml => {
            "ML-oriented structure detected. Prioritize matrix kernels, gradient flow hotspots, and bounded iterative behavior.".to_string()
        }
        AlgorithmClass::General => {
            "General-purpose workload detected. Improve structural regularity before running more aggressive simulation layers.".to_string()
        }
    }
}
