use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flux_sim::app::ablation::execute_ablation;
use flux_sim::app::analyze::execute_analysis;
use flux_sim::app::batch::execute_batch;
use flux_sim::app::consolidate::execute_consolidate;
use flux_sim::app::reproduce::execute_reproduce;
use flux_sim::app::requests::{
    AblationRequest, AnalysisRequest, ConsolidateRequest, ReproduceRequest,
};
use flux_sim::core::analysis::AlgorithmClass;
use flux_sim::core::benchmark::run_synthetic_benchmark;
use flux_sim::core::reporting::print_text_summary;
use flux_sim::util::params::{parse_kelvin, parse_relativistic_fraction};
use std::fs;
use std::path::PathBuf;

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

fn resolve_algorithm_class(input: Option<&str>) -> Option<AlgorithmClass> {
    match input {
        Some("crypto") => Some(AlgorithmClass::Crypto),
        Some("numerical") => Some(AlgorithmClass::Numerical),
        Some("ml") => Some(AlgorithmClass::Ml),
        Some("general") => Some(AlgorithmClass::General),
        _ => None,
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
            let analysis =
                execute_analysis(&request, resolve_algorithm_class(request.algorithm_class))?;
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
            let analysis =
                execute_analysis(&request, resolve_algorithm_class(request.algorithm_class))?;
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
                resolve_algorithm_class(algorithm_class.as_deref()),
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
            execute_reproduce(&request, resolve_algorithm_class(request.algorithm_class))?;
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
