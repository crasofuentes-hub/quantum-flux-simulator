use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use flux_sim::core::analysis::{analyze_file, AlgorithmClass, FileAnalysis};
use flux_sim::core::reporting::{print_text_summary, write_json_report};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "flux-sim")]
#[command(version = "0.1.0")]
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
    },
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

fn execute_analysis(
    source_path: &Path,
    quantum_noise: f64,
    relativistic: &str,
    target_temp: &str,
    json_out: Option<&Path>,
    algorithm_class: Option<&str>,
) -> Result<FileAnalysis> {
    ensure_source_exists(source_path)?;
    let beta = parse_relativistic_fraction(relativistic)?;
    let kelvin = parse_kelvin(target_temp)?;
    let override_class = resolve_algorithm_class(algorithm_class);

    let analysis = analyze_file(source_path, quantum_noise, beta, kelvin, override_class)?;
    if let Some(path) = json_out {
        write_json_report(path, &analysis)?;
    }
    Ok(analysis)
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
            plot: _,
            algorithm_class,
        } => {
            let analysis = execute_analysis(
                &source_path,
                quantum_noise,
                &relativistic,
                &target_temp,
                json_out.as_deref(),
                algorithm_class.as_deref(),
            )?;
            print_text_summary(&analysis);
        }
        Commands::Profile {
            source_path,
            algorithm_class,
            quantum_noise,
            relativistic,
            target_temp,
            plot: _,
        } => {
            let analysis = execute_analysis(
                &source_path,
                quantum_noise,
                &relativistic,
                &target_temp,
                None,
                algorithm_class.as_deref(),
            )?;
            print_text_summary(&analysis);
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
