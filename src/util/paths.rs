use crate::app::requests::ReproduceMode;
use anyhow::{bail, Context, Result};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_source_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("source file does not exist: {}", path.display());
    }
    if !path.is_file() {
        bail!("source path is not a file: {}", path.display());
    }
    Ok(())
}

pub fn detect_input_kind(path: &Path) -> Result<&'static str> {
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

pub fn default_reproduce_output_path(
    input_path: &Path,
    mode: ReproduceMode,
    suffix: &str,
) -> PathBuf {
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

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create parent directory for {}", path.display())
            })?;
        }
    }
    Ok(())
}

pub fn normalize_display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
