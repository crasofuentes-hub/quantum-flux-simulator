use std::path::Path;

#[derive(Debug, Clone)]
pub struct AnalysisRequest<'a> {
    pub source_path: &'a Path,
    pub quantum_noise: f64,
    pub relativistic: &'a str,
    pub target_temp: &'a str,
    pub json_out: Option<&'a Path>,
    pub plot_out: Option<&'a Path>,
    pub algorithm_class: Option<&'a str>,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct ReproduceRequest<'a> {
    pub input_path: &'a Path,
    pub quantum_noise: f64,
    pub relativistic: &'a str,
    pub target_temp: &'a str,
    pub json_out: Option<&'a Path>,
    pub manifest_out: Option<&'a Path>,
    pub algorithm_class: Option<&'a str>,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct AblationRequest<'a> {
    pub input_dir: &'a Path,
    pub quantum_noise: f64,
    pub relativistic: &'a str,
    pub target_temp: &'a str,
    pub json_out: &'a Path,
    pub markdown_out: &'a Path,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct ConsolidateRequest<'a> {
    pub input_dir: &'a Path,
    pub quantum_noise: f64,
    pub relativistic: &'a str,
    pub target_temp: &'a str,
    pub json_out: &'a Path,
    pub markdown_out: &'a Path,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum ReproduceMode {
    Analyze,
    Batch,
}

impl ReproduceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Analyze => "analyze",
            Self::Batch => "batch",
        }
    }
}
