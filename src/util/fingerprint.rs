use crate::util::paths::normalize_display_path;
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const FNV1A64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x00000100000001B3;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Fnv1a64 {
    state: u64,
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Fnv1a64 {
    pub fn new() -> Self {
        Self {
            state: FNV1A64_OFFSET_BASIS,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(FNV1A64_PRIME);
        }
    }

    pub fn finish_hex(self) -> String {
        format!("{:016x}", self.state)
    }
}

pub fn fingerprint_path(path: &Path) -> Result<String> {
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
