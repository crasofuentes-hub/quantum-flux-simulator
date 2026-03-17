use crate::core::analysis::{ANALYSIS_VERSION, REPORT_SCHEMA_VERSION};
use crate::util::fingerprint::fingerprint_path;
use crate::util::paths::{detect_input_kind, ensure_parent_dir, normalize_display_path};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const EXPERIMENT_MANIFEST_SCHEMA_VERSION: &str = "1.0.0";
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize)]
pub struct ExperimentManifest {
    pub manifest_schema_version: String,
    pub experiment_type: String,
    pub tool_version: String,
    pub analysis_version: String,
    pub report_schema_version: String,
    pub seed: u64,
    pub input_path: String,
    pub input_kind: String,
    pub input_fingerprint: String,
    pub quantum_noise: f64,
    pub relativistic_beta: f64,
    pub target_temp_kelvin: f64,
    pub generated_outputs: Vec<String>,
    pub external_comparison_ingested: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ExperimentManifestSpec<'a> {
    pub experiment_type: &'a str,
    pub input_path: &'a Path,
    pub quantum_noise: f64,
    pub relativistic_beta: f64,
    pub target_temp_kelvin: f64,
    pub seed: u64,
    pub generated_outputs: Vec<String>,
    pub external_comparison_ingested: Option<bool>,
}

pub fn build_experiment_manifest(spec: &ExperimentManifestSpec<'_>) -> Result<ExperimentManifest> {
    let input_kind = detect_input_kind(spec.input_path)?;
    let input_fingerprint = fingerprint_path(spec.input_path)?;

    Ok(ExperimentManifest {
        manifest_schema_version: EXPERIMENT_MANIFEST_SCHEMA_VERSION.to_string(),
        experiment_type: spec.experiment_type.to_string(),
        tool_version: TOOL_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        seed: spec.seed,
        input_path: normalize_display_path(spec.input_path),
        input_kind: input_kind.to_string(),
        input_fingerprint,
        quantum_noise: spec.quantum_noise,
        relativistic_beta: spec.relativistic_beta,
        target_temp_kelvin: spec.target_temp_kelvin,
        generated_outputs: spec.generated_outputs.clone(),
        external_comparison_ingested: spec.external_comparison_ingested,
    })
}

pub fn default_manifest_path(output_path: &Path) -> PathBuf {
    let parent = output_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let stem = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("report");

    parent.join(format!("{stem}.manifest.json"))
}

pub fn write_experiment_manifest(path: &Path, manifest: &ExperimentManifest) -> Result<()> {
    ensure_parent_dir(path)?;
    let json = serde_json::to_string_pretty(manifest).with_context(|| {
        format!(
            "failed to serialize experiment manifest: {}",
            path.display()
        )
    })?;
    fs::write(path, json)
        .with_context(|| format!("failed to write experiment manifest: {}", path.display()))?;
    Ok(())
}
