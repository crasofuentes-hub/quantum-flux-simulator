use crate::app::analyze::execute_analysis;
use crate::app::batch::execute_batch;
use crate::app::requests::{AnalysisRequest, ReproduceMode, ReproduceRequest};
use crate::core::analysis::{AlgorithmClass, ANALYSIS_VERSION, REPORT_SCHEMA_VERSION};
use crate::util::fingerprint::fingerprint_path;
use crate::util::params::{parse_kelvin, parse_relativistic_fraction};
use crate::util::paths::{
    default_reproduce_output_path, detect_input_kind, ensure_parent_dir, normalize_display_path,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPRO_MANIFEST_SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize)]
pub struct ReproManifest {
    pub manifest_schema_version: String,
    pub tool_version: String,
    pub mode: String,
    pub seed: u64,
    pub quantum_noise: f64,
    pub relativistic_beta: f64,
    pub target_temp_kelvin: f64,
    pub analysis_version: String,
    pub report_schema_version: String,
    pub input_path: String,
    pub input_kind: String,
    pub input_fingerprint: String,
    pub output_report_path: String,
    pub generated_outputs: Vec<String>,
}

pub fn execute_reproduce(
    request: &ReproduceRequest<'_>,
    class_override: Option<AlgorithmClass>,
) -> Result<()> {
    if !request.input_path.exists() {
        bail!(
            "input_path does not exist: {}",
            request.input_path.display()
        );
    }

    let beta = parse_relativistic_fraction(request.relativistic)?;
    let kelvin = parse_kelvin(request.target_temp)?;
    let input_kind = detect_input_kind(request.input_path)?;
    let mode = if input_kind == "file" {
        ReproduceMode::Analyze
    } else {
        ReproduceMode::Batch
    };

    let report_path = request
        .json_out
        .map(PathBuf::from)
        .unwrap_or_else(|| default_reproduce_output_path(request.input_path, mode, "json"));

    let manifest_path = request.manifest_out.map(PathBuf::from).unwrap_or_else(|| {
        default_reproduce_output_path(request.input_path, mode, "manifest.json")
    });

    ensure_parent_dir(&report_path)?;
    ensure_parent_dir(&manifest_path)?;

    match mode {
        ReproduceMode::Analyze => {
            let analysis_request = AnalysisRequest {
                source_path: request.input_path,
                quantum_noise: request.quantum_noise,
                relativistic: request.relativistic,
                target_temp: request.target_temp,
                json_out: Some(report_path.as_path()),
                plot_out: None,
                algorithm_class: request.algorithm_class,
                seed: request.seed,
            };
            let _ = execute_analysis(&analysis_request, class_override)?;
        }
        ReproduceMode::Batch => {
            execute_batch(
                request.input_path,
                request.quantum_noise,
                request.relativistic,
                request.target_temp,
                &report_path,
                class_override,
                request.seed,
            )?;
        }
    }

    let input_fingerprint = fingerprint_path(request.input_path)?;
    let manifest = ReproManifest {
        manifest_schema_version: REPRO_MANIFEST_SCHEMA_VERSION.to_string(),
        tool_version: TOOL_VERSION.to_string(),
        mode: mode.as_str().to_string(),
        seed: request.seed,
        quantum_noise: request.quantum_noise,
        relativistic_beta: beta,
        target_temp_kelvin: kelvin,
        analysis_version: ANALYSIS_VERSION.to_string(),
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        input_path: normalize_display_path(request.input_path),
        input_kind: input_kind.to_string(),
        input_fingerprint,
        output_report_path: normalize_display_path(&report_path),
        generated_outputs: vec![
            normalize_display_path(&report_path),
            normalize_display_path(&manifest_path),
        ],
    };

    write_pretty_json(&manifest_path, &manifest)?;

    println!("flux-sim reproduce OK");
    println!("mode={}", mode.as_str());
    println!("input_kind={input_kind}");
    println!("seed={}", request.seed);
    println!("output_report={}", report_path.display());
    println!("output_manifest={}", manifest_path.display());

    Ok(())
}

fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("failed to serialize JSON for {}", path.display()))?;
    fs::write(path, json).with_context(|| format!("failed to write JSON: {}", path.display()))?;
    Ok(())
}
