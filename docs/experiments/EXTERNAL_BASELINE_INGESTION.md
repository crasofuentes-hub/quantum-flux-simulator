# External baseline ingestion

The consolidated report can now ingest `target/comparison-report.json` when that file already exists.

## Current behavior

- Rust still does not invoke Radon directly.
- The PowerShell bootstrap script remains responsible for generating:
  - `target/radon-cc.json`
  - `target/radon-mi.json`
  - `target/benchmark-report.json`
  - `target/comparison-report.json`
  - `target/comparison-summary.md`

## What Rust does now

When `flux-sim consolidate` runs:

- if `target/comparison-report.json` is absent, the external baseline remains reference-only,
- if `target/comparison-report.json` is present and valid JSON, it is embedded into the consolidated report.

## Interpretation boundary

This improves traceability and reporting integration.

It is still not the same as a full Rust-native execution of the external baseline toolchain.