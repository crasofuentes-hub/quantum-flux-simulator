use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use flux_sim::core::analysis::{
    analyze_file_with_seed, AlgorithmClass, FileAnalysis, ANALYSIS_VERSION, REPORT_SCHEMA_VERSION,
};
use flux_sim::core::benchmark::run_synthetic_benchmark;
use flux_sim::core::reporting::{
    print_text_summary, write_batch_json_report, write_json_report, BatchReport,
};
use flux_sim::core::solver::summarize_batch;
use flux_sim::core::visualization::write_png_report;
use serde::Serialize;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPRO_MANIFEST_SCHEMA_VERSION: &str = "1.0.0";
const FNV1A64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x00000100000001B3;

#[derive(Parser, Debug)]
#[command(name = "flux-sim")]
#[command(version = "0.2.0")]
#[command(about = "Quantum Flux Simulator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Analyze {
        source_path: PathBuf,
        #[arg(long, default_value_t = 0.01)]
        quantum_noise: f64,
        #[arg(long, default_value = "0.0c")]
        relativistic: String,
        #[arg(long, default_value = "300K")]
        target_temp: String,
        #[arg(long)]
        json_out: Option<PathBuf>,
        #[arg(long)]
        plot: Option<PathBuf>,
        #[arg(long)]
        algorithm_class: Option<String>,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    Profile {
        source_path: PathBuf,
        #[arg(long)]
        algorithm_class: Option<String>,
        #[arg(long, default_value_t = 0.01)]
        quantum_noise: f64,
        #[arg(long, default_value = "0.0c")]
        relativistic: String,
        #[arg(long, default_value = "300K")]
        target_temp: String,
        #[arg(long)]
        plot: Option<PathBuf>,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    Batch {
        input_dir: PathBuf,
        #[arg(long, default_value_t = 0.01)]
        quantum_noise: f64,
        #[arg(long, default_value = "0.0c")]
        relativistic: String,
        #[arg(long, default_value = "300K")]
        target_temp: String,
        #[arg(long)]
        json_out: PathBuf,
        #[arg(long)]
        algorithm_class: Option<String>,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    Benchmark {
        input_dir: PathBuf,
        #[arg(long, default_value_t = 0.01)]
        quantum_noise: f64,
        #[arg(long, default_value = "0.0c")]
        relativistic: String,
        #[arg(long, default_value = "300K")]
        target_temp: String,
        #[arg(long)]
        json_out: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    Reproduce {
        input_path: PathBuf,
        #[arg(long, default_value_t = 0.01)]
        quantum_noise: f64,
        #[arg(long, default_value = "0.0c")]
        relativistic: String,
        #[arg(long, default_value = "300K")]
        target_temp: String,
        #[arg(long)]
        json_out: Option<PathBuf>,
        #[arg(long)]
        manifest_out: Option<PathBuf>,
        #[arg(long)]
        algorithm_class: Option<String>,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
}

#[derive(Debug, Clone)]
struct AnalysisRequest<'a> {
    source_path: &'a Path,
    quantum_noise: f64,
    relativistic: &'a str,
    target_temp: &'a str,
    json_out: Option<&'a Path>,
    plot_out: Option<&'a Path>,
    algorithm_class: Option<&'a str>,
    seed: u64,
}

#[derive(Debug, Clone)]
struct ReproduceRequest<'a> {
    input_path: &'a Path,
    quantum_noise: f64,
    relativistic: &'a str,
    target_temp: &'a str,
    json_out: Option<&'a Path>,
    manifest_out: Option<&'a Path>,
    algorithm_class: Option<&'a str>,
    seed: u64,
}

#[derive(Debug, Clone, Copy)]
enum ReproduceMode {
    Analyze,
    Batch,
}

impl ReproduceMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Analyze => "analyze",
            Self::Batch => "batch",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ReproManifest {
    manifest_schema_version: String,
    tool_version: String,
    mode: String,
    seed: u64,
    quantum_noise: f64,
    relativistic_beta: f64,
    target_temp_kelvin: f64,
    analysis_version: String,
    report_schema_version: String,
    input_path: String,
    input_kind: String,
    input_fingerprint: String,
    output_report_path: String,
    generated_outputs: Vec<String>,
}

fn parse_relativistic_fraction(input: &str) -> Result<f64> {
    let value = input.trim();
    if !value.ends_with('c') {
        bail!("relativistic must end with 'c', for example 0.8c");
    }

    let number = &value[..value.len() - 1];
    let beta: f64 = number
        .parse()
        .with_context(|| format!("invalid relativistic value: {input}"))?;

    if !(0.0..1.0).contains(&beta) && beta != 0.0 {
        bail!("relativistic fraction must be in [0.0, 1.0)");
    }

    Ok(beta)
}

fn parse_kelvin(input: &str) -> Result<f64> {
    let value = input.trim();
    if !value.ends_with('K') {
        bail!("target-temp must end with 'K', for example 77K");
    }

    let number = &value[..value.len() - 1];
    let kelvin: f64 = number
        .parse()
        .with_context(|| format!("invalid Kelvin value: {input}"))?;

    if kelvin < 0.0 {
        bail!("target temperature cannot be negative");
    }

    Ok(kelvin)
}

fn ensure_source_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("source file does not exist: {}", path.display());
    }
    if !path.is_file() {
        bail!("source path is not a file: {}", path.display());
    }
    Ok(())
}

fn resolve_algorithm_class(input: Option<&str>) -> Option<AlgorithmClass> {
    match input {
        Some("crypto") => Some(AlgorithmClass::Crypto),
        Some("numerical") => Some(AlgorithmClass::Numerical),
        Some("ml") => Some(AlgorithmClass::Ml),
        Some("general") => Some(AlgorithmClass::General),
        _ => None,
    }
}

fn execute_analysis(request: &AnalysisRequest<'_>) -> Result<FileAnalysis> {
    ensure_source_exists(request.source_path)?;
    let beta = parse_relativistic_fraction(request.relativistic)?;
    let kelvin = parse_kelvin(request.target_temp)?;
    let override_class = resolve_algorithm_class(request.algorithm_class);

    let analysis = analyze_file_with_seed(
        request.source_path,
        request.quantum_noise,
        beta,
        kelvin,
        override_class,
        request.seed,
    )?;

    if let Some(path) = request.json_out {
        write_json_report(path, &analysis)?;
    }
    if let Some(path) = request.plot_out {
        write_png_report(path, &analysis)?;
    }
    Ok(analysis)
}

fn execute_batch(
    input_dir: &Path,
    quantum_noise: f64,
    relativistic: &str,
    target_temp: &str,
    json_out: &Path,
    algorithm_class: Option<&str>,
    seed: u64,
) -> Result<()> {
    if !input_dir.exists() {
        bail!("input_dir does not exist: {}", input_dir.display());
    }
    if !input_dir.is_dir() {
        bail!("input_dir is not a directory: {}", input_dir.display());
    }

    let beta = parse_relativistic_fraction(relativistic)?;
    let kelvin = parse_kelvin(target_temp)?;
    let override_class = resolve_algorithm_class(algorithm_class);

    let mut files = Vec::new();
    for entry in fs::read_dir(input_dir)
        .with_context(|| format!("failed to read dir: {}", input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if ["py", "rs", "cpp", "cc", "cxx", "js", "mjs", "cjs"].contains(&ext.as_str()) {
                files.push(path);
            }
        }
    }

    files.sort();

    let mut analyses = Vec::new();
    for path in files {
        analyses.push(analyze_file_with_seed(
            &path,
            quantum_noise,
            beta,
            kelvin,
            override_class,
            seed,
        )?);
    }

    let stabilities: Vec<f64> = analyses.iter().map(|a| a.stability_score).collect();
    let risks: Vec<f64> = analyses.iter().map(|a| a.singularity_risk).collect();
    let collapses: Vec<f64> = analyses
        .iter()
        .map(|a| a.solver_summary.collapse_probability)
        .collect();

    let aggregate = summarize_batch(&stabilities, &risks, &collapses);

    let report = BatchReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed,
        aggregate,
        files: analyses,
    };

    write_batch_json_report(json_out, &report)?;
    println!("flux-sim batch OK");
    println!("files_analyzed={}", report.aggregate.files_analyzed);
    println!(
        "mean_stability_score={}",
        report.aggregate.mean_stability_score
    );
    println!(
        "max_singularity_risk={}",
        report.aggregate.max_singularity_risk
    );
    println!(
        "mean_collapse_probability={}",
        report.aggregate.mean_collapse_probability
    );

    Ok(())
}

fn execute_reproduce(request: &ReproduceRequest<'_>) -> Result<()> {
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
            let _ = execute_analysis(&analysis_request)?;
        }
        ReproduceMode::Batch => {
            execute_batch(
                request.input_path,
                request.quantum_noise,
                request.relativistic,
                request.target_temp,
                &report_path,
                request.algorithm_class,
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

fn detect_input_kind(path: &Path) -> Result<&'static str> {
    if path.is_file() {
        Ok("file")
    } else if path.is_dir() {
        Ok("directory")
    } else {
        bail!(
            "input_path is neither a file nor a directory: {}",
            path.display()
        );
    }
}

fn default_reproduce_output_path(input_path: &Path, mode: ReproduceMode, suffix: &str) -> PathBuf {
    let parent = input_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let base_name = match mode {
        ReproduceMode::Analyze => input_path
            .file_stem()
            .map(OsString::from)
            .unwrap_or_else(|| OsString::from("analysis")),
        ReproduceMode::Batch => input_path
            .file_name()
            .map(OsString::from)
            .unwrap_or_else(|| OsString::from("batch")),
    };

    let file_name = format!("{}.reproduce.{}", base_name.to_string_lossy(), suffix);
    parent.join(file_name)
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create parent directory for {}", path.display())
            })?;
        }
    }
    Ok(())
}

fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("failed to serialize JSON for {}", path.display()))?;
    fs::write(path, json).with_context(|| format!("failed to write JSON: {}", path.display()))?;
    Ok(())
}

fn normalize_display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn fingerprint_path(path: &Path) -> Result<String> {
    if path.is_file() {
        let bytes =
            fs::read(path).with_context(|| format!("failed to read file: {}", path.display()))?;
        let mut hasher = Fnv1a64::new();
        hasher.update(normalize_display_path(path).as_bytes());
        hasher.update(&[0]);
        hasher.update(&bytes);
        return Ok(hasher.finish_hex());
    }

    if path.is_dir() {
        let mut files = Vec::new();
        collect_files_recursive(path, path, &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Fnv1a64::new();
        hasher.update(normalize_display_path(path).as_bytes());
        hasher.update(&[0xff]);

        for (relative, absolute) in files {
            let bytes = fs::read(&absolute)
                .with_context(|| format!("failed to read file: {}", absolute.display()))?;
            hasher.update(relative.as_bytes());
            hasher.update(&[0]);
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
            hasher.update(&[0xfe]);
        }

        return Ok(hasher.finish_hex());
    }

    bail!(
        "cannot fingerprint non-file non-directory path: {}",
        path.display()
    )
}

fn collect_files_recursive(
    root: &Path,
    current: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read dir: {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, out)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .with_context(|| format!("failed to relativize path: {}", path.display()))?;
            out.push((normalize_display_path(relative), path));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    fn new() -> Self {
        Self {
            state: FNV1A64_OFFSET_BASIS,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(FNV1A64_PRIME);
        }
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.state)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            source_path,
            quantum_noise,
            relativistic,
            target_temp,
            json_out,
            plot,
            algorithm_class,
            seed,
        } => {
            let request = AnalysisRequest {
                source_path: &source_path,
                quantum_noise,
                relativistic: &relativistic,
                target_temp: &target_temp,
                json_out: json_out.as_deref(),
                plot_out: plot.as_deref(),
                algorithm_class: algorithm_class.as_deref(),
                seed,
            };
            let analysis = execute_analysis(&request)?;
            print_text_summary(&analysis);
        }
        Commands::Profile {
            source_path,
            algorithm_class,
            quantum_noise,
            relativistic,
            target_temp,
            plot,
            seed,
        } => {
            let request = AnalysisRequest {
                source_path: &source_path,
                quantum_noise,
                relativistic: &relativistic,
                target_temp: &target_temp,
                json_out: None,
                plot_out: plot.as_deref(),
                algorithm_class: algorithm_class.as_deref(),
                seed,
            };
            let analysis = execute_analysis(&request)?;
            print_text_summary(&analysis);
        }
        Commands::Batch {
            input_dir,
            quantum_noise,
            relativistic,
            target_temp,
            json_out,
            algorithm_class,
            seed,
        } => {
            execute_batch(
                &input_dir,
                quantum_noise,
                &relativistic,
                &target_temp,
                &json_out,
                algorithm_class.as_deref(),
                seed,
            )?;
        }
        Commands::Benchmark {
            input_dir,
            quantum_noise,
            relativistic,
            target_temp,
            json_out,
            seed,
        } => {
            let beta = parse_relativistic_fraction(&relativistic)?;
            let kelvin = parse_kelvin(&target_temp)?;
            let report = run_synthetic_benchmark(&input_dir, quantum_noise, beta, kelvin, seed)?;
            let json = serde_json::to_string_pretty(&report)
                .context("failed to serialize benchmark report")?;
            fs::write(&json_out, json).with_context(|| {
                format!("failed to write benchmark JSON: {}", json_out.display())
            })?;
            println!("flux-sim benchmark OK");
            println!("files_analyzed={}", report.aggregate.files_analyzed);
            println!("class_accuracy={}", report.aggregate.class_accuracy);
            println!("mean_baseline_risk={}", report.aggregate.mean_baseline_risk);
            println!("mean_model_risk={}", report.aggregate.mean_model_risk);
            println!(
                "mean_baseline_stability={}",
                report.aggregate.mean_baseline_stability
            );
            println!(
                "mean_model_stability={}",
                report.aggregate.mean_model_stability
            );
            println!(
                "mean_collapse_probability={}",
                report.aggregate.mean_collapse_probability
            );
        }
        Commands::Reproduce {
            input_path,
            quantum_noise,
            relativistic,
            target_temp,
            json_out,
            manifest_out,
            algorithm_class,
            seed,
        } => {
            let request = ReproduceRequest {
                input_path: &input_path,
                quantum_noise,
                relativistic: &relativistic,
                target_temp: &target_temp,
                json_out: json_out.as_deref(),
                manifest_out: manifest_out.as_deref(),
                algorithm_class: algorithm_class.as_deref(),
                seed,
            };
            execute_reproduce(&request)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        default_reproduce_output_path, parse_kelvin, parse_relativistic_fraction, ReproduceMode,
    };
    use std::path::Path;

    #[test]
    fn parses_relativistic_fraction() {
        let beta = parse_relativistic_fraction("0.8c").expect("beta should parse");
        assert!((beta - 0.8).abs() < 1e-12);
    }

    #[test]
    fn parses_kelvin() {
        let kelvin = parse_kelvin("77K").expect("kelvin should parse");
        assert!((kelvin - 77.0).abs() < 1e-12);
    }

    #[test]
    fn derives_default_reproduce_output_for_file() {
        let path = Path::new("samples/example.py");
        let out = default_reproduce_output_path(path, ReproduceMode::Analyze, "json");
        assert_eq!(
            out.to_string_lossy().replace('\\', "/"),
            "samples/example.reproduce.json"
        );
    }

    #[test]
    fn derives_default_reproduce_output_for_directory() {
        let path = Path::new("samples/corpus");
        let out = default_reproduce_output_path(path, ReproduceMode::Batch, "manifest.json");
        assert_eq!(
            out.to_string_lossy().replace('\\', "/"),
            "samples/corpus.reproduce.manifest.json"
        );
    }
}
