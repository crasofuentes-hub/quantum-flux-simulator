use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flux_sim::app::requests::{
    AblationRequest, AnalysisRequest, ConsolidateRequest, ReproduceMode, ReproduceRequest,
};
use flux_sim::core::analysis::{
    analyze_file_with_seed, AlgorithmClass, FileAnalysis, ANALYSIS_VERSION, REPORT_SCHEMA_VERSION,
};
use flux_sim::core::benchmark::{
    render_ablation_markdown, run_synthetic_ablation, run_synthetic_benchmark,
};
use flux_sim::core::reporting::{
    print_text_summary, render_consolidated_markdown, try_read_external_comparison_json,
    write_batch_json_report, write_consolidated_json_report, write_json_report, BatchReport,
    ConsolidatedComparisonReport, ExternalBaselineReference,
};
use flux_sim::core::solver::summarize_batch;
use flux_sim::core::visualization::write_png_report;
use flux_sim::util::fingerprint::fingerprint_path;
use flux_sim::util::params::{parse_kelvin, parse_relativistic_fraction};
use flux_sim::util::paths::{
    default_reproduce_output_path, detect_input_kind, ensure_parent_dir, ensure_source_exists,
    normalize_display_path,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPRO_MANIFEST_SCHEMA_VERSION: &str = "1.0.0";

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
    Ablation {
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
        markdown_out: PathBuf,
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
    Consolidate {
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
        markdown_out: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
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
        anyhow::bail!("input_dir does not exist: {}", input_dir.display());
    }
    if !input_dir.is_dir() {
        anyhow::bail!("input_dir is not a directory: {}", input_dir.display());
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

fn execute_ablation(request: &AblationRequest<'_>) -> Result<()> {
    if !request.input_dir.exists() {
        anyhow::bail!("input_dir does not exist: {}", request.input_dir.display());
    }
    if !request.input_dir.is_dir() {
        anyhow::bail!(
            "input_dir is not a directory: {}",
            request.input_dir.display()
        );
    }

    let beta = parse_relativistic_fraction(request.relativistic)?;
    let kelvin = parse_kelvin(request.target_temp)?;

    ensure_parent_dir(request.json_out)?;
    ensure_parent_dir(request.markdown_out)?;

    let report = run_synthetic_ablation(
        request.input_dir,
        request.quantum_noise,
        beta,
        kelvin,
        request.seed,
    )?;

    let json =
        serde_json::to_string_pretty(&report).context("failed to serialize ablation report")?;
    fs::write(request.json_out, json).with_context(|| {
        format!(
            "failed to write ablation JSON report: {}",
            request.json_out.display()
        )
    })?;

    let markdown = render_ablation_markdown(&report);
    fs::write(request.markdown_out, markdown).with_context(|| {
        format!(
            "failed to write ablation markdown report: {}",
            request.markdown_out.display()
        )
    })?;

    println!("flux-sim ablation OK");
    println!("files_analyzed={}", report.entries.len());
    println!("variants={}", report.aggregate.len());
    println!("json_out={}", request.json_out.display());
    println!("markdown_out={}", request.markdown_out.display());

    Ok(())
}

fn execute_consolidate(request: &ConsolidateRequest<'_>) -> Result<()> {
    if !request.input_dir.exists() {
        anyhow::bail!("input_dir does not exist: {}", request.input_dir.display());
    }
    if !request.input_dir.is_dir() {
        anyhow::bail!(
            "input_dir is not a directory: {}",
            request.input_dir.display()
        );
    }

    let beta = parse_relativistic_fraction(request.relativistic)?;
    let kelvin = parse_kelvin(request.target_temp)?;

    ensure_parent_dir(request.json_out)?;
    ensure_parent_dir(request.markdown_out)?;

    let benchmark = run_synthetic_benchmark(
        request.input_dir,
        request.quantum_noise,
        beta,
        kelvin,
        request.seed,
    )?;

    let ablation = run_synthetic_ablation(
        request.input_dir,
        request.quantum_noise,
        beta,
        kelvin,
        request.seed,
    )?;

    let external_comparison_path = Path::new("target/comparison-report.json");
    let external_comparison_json = try_read_external_comparison_json(external_comparison_path)?;

    let external_baseline = if external_comparison_json.is_some() {
        ExternalBaselineReference {
            name: "radon".to_string(),
            integrated_automatically: true,
            asset_path: "external_baselines/run-radon-benchmark.ps1".to_string(),
            doc_path: "docs/experiments/EXTERNAL_BASELINE.md".to_string(),
            status: "ingested_from_json".to_string(),
            notes: "Rust did not execute Radon directly, but target/comparison-report.json was found and ingested into the consolidated report.".to_string(),
        }
    } else {
        ExternalBaselineReference {
            name: "radon".to_string(),
            integrated_automatically: false,
            asset_path: "external_baselines/run-radon-benchmark.ps1".to_string(),
            doc_path: "docs/experiments/EXTERNAL_BASELINE.md".to_string(),
            status: "reference_only".to_string(),
            notes: "External baseline assets exist, but target/comparison-report.json was not found at consolidate time.".to_string(),
        }
    };

    let report = ConsolidatedComparisonReport {
        report_schema_version: REPORT_SCHEMA_VERSION.to_string(),
        analysis_version: ANALYSIS_VERSION.to_string(),
        seed: request.seed,
        benchmark,
        ablation,
        external_baseline,
        external_comparison_json,
    };

    write_consolidated_json_report(request.json_out, &report)?;

    let markdown = render_consolidated_markdown(&report);
    fs::write(request.markdown_out, markdown).with_context(|| {
        format!(
            "failed to write consolidated markdown report: {}",
            request.markdown_out.display()
        )
    })?;

    println!("flux-sim consolidate OK");
    println!(
        "benchmark_files={}",
        report.benchmark.aggregate.files_analyzed
    );
    println!("ablation_variants={}", report.ablation.aggregate.len());
    println!(
        "external_baseline_integrated={}",
        report.external_baseline.integrated_automatically
    );
    println!("json_out={}", request.json_out.display());
    println!("markdown_out={}", request.markdown_out.display());

    Ok(())
}

fn execute_reproduce(request: &ReproduceRequest<'_>) -> Result<()> {
    if !request.input_path.exists() {
        anyhow::bail!(
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

fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("failed to serialize JSON for {}", path.display()))?;
    fs::write(path, json).with_context(|| format!("failed to write JSON: {}", path.display()))?;
    Ok(())
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
        Commands::Ablation {
            input_dir,
            quantum_noise,
            relativistic,
            target_temp,
            json_out,
            markdown_out,
            seed,
        } => {
            let request = AblationRequest {
                input_dir: &input_dir,
                quantum_noise,
                relativistic: &relativistic,
                target_temp: &target_temp,
                json_out: &json_out,
                markdown_out: &markdown_out,
                seed,
            };
            execute_ablation(&request)?;
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
        Commands::Consolidate {
            input_dir,
            quantum_noise,
            relativistic,
            target_temp,
            json_out,
            markdown_out,
            seed,
        } => {
            let request = ConsolidateRequest {
                input_dir: &input_dir,
                quantum_noise,
                relativistic: &relativistic,
                target_temp: &target_temp,
                json_out: &json_out,
                markdown_out: &markdown_out,
                seed,
            };
            execute_consolidate(&request)?;
        }
    }

    Ok(())
}
