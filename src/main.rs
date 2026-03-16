use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use flux_sim::core::analysis::{
    analyze_file_with_seed, AlgorithmClass, FileAnalysis, ANALYSIS_VERSION, REPORT_SCHEMA_VERSION,
};
use flux_sim::core::reporting::{
    print_text_summary, write_batch_json_report, write_json_report, BatchReport,
};
use flux_sim::core::solver::summarize_batch;
use flux_sim::core::visualization::write_png_report;
use std::fs;
use std::path::{Path, PathBuf};

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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_kelvin, parse_relativistic_fraction};

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
}
