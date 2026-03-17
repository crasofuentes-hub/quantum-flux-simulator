use crate::app::requests::AnalysisRequest;
use crate::core::analysis::{analyze_file_with_seed, AlgorithmClass, FileAnalysis};
use crate::core::reporting::write_json_report;
use crate::core::visualization::write_png_report;
use crate::util::params::{parse_kelvin, parse_relativistic_fraction};
use crate::util::paths::ensure_source_exists;
use anyhow::Result;

pub fn execute_analysis(
    request: &AnalysisRequest<'_>,
    class_override: Option<AlgorithmClass>,
) -> Result<FileAnalysis> {
    ensure_source_exists(request.source_path)?;
    let beta = parse_relativistic_fraction(request.relativistic)?;
    let kelvin = parse_kelvin(request.target_temp)?;

    let analysis = analyze_file_with_seed(
        request.source_path,
        request.quantum_noise,
        beta,
        kelvin,
        class_override,
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
