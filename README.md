[![CI](https://github.com/crasofuentes-hub/quantum-flux-simulator/actions/workflows/ci.yml/badge.svg)](https://github.com/crasofuentes-hub/quantum-flux-simulator/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

# quantum-flux-simulator

**flux-sim** is a physics-inspired research CLI for reproducible code analysis and benchmark-driven comparison workflows.

It analyzes source-code structure, builds an explicit effective state model, and emits reproducible reports for internal comparison, experimentation, and tool-to-tool benchmarking.

## Status

This project should currently be described as:

- a **physics-inspired research tool**
- a **reproducible effective-model prototype**
- an **internal benchmark and comparison framework**

This project should **not** currently be described as:

- a physically faithful quantum simulator
- a Wheeler-DeWitt implementation
- a production-grade defect detector
- an externally validated scientific predictor

## What it does

`flux-sim` combines:

- structural source analysis
- classification into `crypto`, `numerical`, `ml`, `general`
- intermediate modeling with critical blocks, hotspots, and information channels
- effective 2x2 density-state evolution
- Lindblad-like dissipative terms
- effective relativistic gamma-like scaling
- explicit global constraint penalty
- Monte Carlo-style stress summaries
- reproducible benchmark, ablation, and consolidated comparison workflows

## Current capabilities

- single-file analysis
- directory batch analysis
- synthetic benchmark workflow
- ablation workflow
- reproduce command with manifest output
- consolidated comparison report generation
- semi-real labeled corpus
- seeded-defects dataset
- optional ingestion of Radon and Semgrep artifacts
- JSON output for downstream tooling
- PNG output for quick inspection
- practical handling of Python, Rust, JavaScript, TypeScript, and TSX files

## Installation

### Build locally

~~~bash
cargo build
~~~

### Run without installing

~~~bash
cargo run -- analyze examples/my_crypto.py --json-out target/analysis.json --seed 42
~~~

### Install from local source

~~~bash
cargo install --path .
~~~

### crates.io status

`flux-sim` is **not yet published** on crates.io.

When published, installation should look like:

~~~bash
cargo install flux-sim
~~~

## Quick start

### Analyze a single file

~~~bash
cargo run -- analyze examples/my_crypto.py --json-out target/analysis.json --seed 42
~~~

### Reproduce a single-file run with manifest

~~~bash
cargo run -- reproduce examples/my_crypto.py --seed 42
~~~

### Benchmark a dataset

~~~bash
cargo run -- benchmark datasets/seeded_defects --quantum-noise 0.01 --relativistic 0.2c --target-temp 77K --json-out target/seeded-benchmark.json --seed 42
~~~

### Run ablation comparison

~~~bash
cargo run -- ablation benchmarks/dataset --quantum-noise 0.01 --relativistic 0.2c --target-temp 77K --json-out target/ablation-report.json --markdown-out target/ablation-report.md --seed 42
~~~

### Build a consolidated comparison report

~~~bash
cargo run -- consolidate benchmarks/dataset --quantum-noise 0.01 --relativistic 0.2c --target-temp 77K --json-out target/consolidated-report.json --markdown-out target/consolidated-report.md --seed 42
~~~

## Commands

| Command | Purpose |
|---|---|
| `analyze` | Analyze one source file and optionally emit JSON / PNG |
| `profile` | Analyze one file and print a summary with optional PNG |
| `batch` | Analyze supported files in a directory |
| `benchmark` | Run the internal benchmark workflow on a dataset |
| `ablation` | Compare full model and ablated variants |
| `reproduce` | Generate a reproducible report and manifest from a file or directory |
| `consolidate` | Build a report joining benchmark, ablation, and optional external artifacts |

## Main flags

| Flag | Meaning |
|---|---|
| `--quantum-noise` | Effective noise parameter used by the model and solver |
| `--relativistic` | Effective beta parameter, formatted like `0.2c` |
| `--target-temp` | Effective target temperature, formatted like `77K` |
| `--json-out` | Output path for JSON report |
| `--markdown-out` | Output path for Markdown report |
| `--plot` | PNG output path for `analyze` / `profile` |
| `--seed` | Reproducibility seed |
| `--algorithm-class` | Optional class override: `crypto`, `numerical`, `ml`, `general` |

## Command-specific usage

### `analyze`

Analyze a single file and optionally emit JSON and PNG.

~~~bash
cargo run -- analyze <source_path> [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--json-out <path>] [--plot <path>] [--algorithm-class <class>] [--seed <u64>]
~~~

### `profile`

Analyze a single file and print a summary, with optional PNG.

~~~bash
cargo run -- profile <source_path> [--algorithm-class <class>] [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--plot <path>] [--seed <u64>]
~~~

### `batch`

Analyze supported files in a directory.

~~~bash
cargo run -- batch <input_dir> --json-out <path> [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--algorithm-class <class>] [--seed <u64>]
~~~

### `benchmark`

Run the internal benchmark workflow on a dataset.

~~~bash
cargo run -- benchmark <input_dir> --json-out <path> [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--seed <u64>]
~~~

### `ablation`

Run ablation comparison and emit JSON + Markdown.

~~~bash
cargo run -- ablation <input_dir> --json-out <path> --markdown-out <path> [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--seed <u64>]
~~~

### `reproduce`

Generate a reproducible report and manifest from a file or directory.

~~~bash
cargo run -- reproduce <input_path> [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--json-out <path>] [--manifest-out <path>] [--algorithm-class <class>] [--seed <u64>]
~~~

### `consolidate`

Build a consolidated comparison report.

~~~bash
cargo run -- consolidate <input_dir> --json-out <path> --markdown-out <path> [--quantum-noise <f64>] [--relativistic <beta>] [--target-temp <kelvin>] [--seed <u64>]
~~~

## How to interpret the metrics

These outputs are **physics-inspired code-analysis signals**, not measured physical quantities.

### `stability_score`

Higher means the analyzed workload looks more stable under the current effective model and solver summary.

### `singularity_risk`

A normalized risk-like score in `[0, 1]`. Higher means the current model detects more concentrated structural and solver stress.

### `collapse_probability`

A Monte Carlo-style fraction of simulated samples exceeding an internal collapse threshold.

### `decoherence_rate`

Effective dissipation pressure derived from the block-level model. This is a model quantity, not physical decoherence from a real system.

### `von_neumann_entropy`

Entropy of the effective 2x2 density-like state after model evolution and stabilization.

### `global_constraint_penalty`

A scalar regularization term derived from energy, coherence penalties, and decoherence rate. It is **not** a Wheeler-DeWitt solution.

## Datasets and benchmark layers

### Synthetic benchmark

- `benchmarks/dataset/`

### Semi-real corpus

- `benchmarks/semi_real/`
- `benchmarks/semi_real/labels.json`

### Seeded defects

- `datasets/seeded_defects/`
- `datasets/seeded_defects/labels.json`

## External baselines

### Radon

- `external_baselines/run-radon-benchmark.ps1`
- `docs/experiments/EXTERNAL_BASELINE.md`

### Semgrep

- `external_baselines/run-semgrep-benchmark.ps1`
- `docs/experiments/SEMGREP_BASELINE.md`

Rust does **not** execute these external tools directly.  
`consolidate` ingests their generated artifacts when present in `target/`.

## Key documentation

- `docs/science/CLAIMS_MATRIX.md`
- `docs/science/EFFECTIVE_EQUATIONS.md`
- `docs/model/FORMAL_MODEL.md`
- `docs/architecture/PARSER_EVOLUTION.md`
- `ROADMAP.md`

## Reproducibility

The project emphasizes reproducibility through:

- seeded execution
- input fingerprinting
- unified experiment manifests
- internal smoke tests
- invariants checks
- dataset contract checks

## Current limitations

The most important current limitations are:

- the model is effective and simplified, not physically faithful
- benchmark evidence is still mostly internal
- external comparison is bootstrap-level rather than deeply integrated
- datasets are still relatively small and curated
- AST-grade parsing is not implemented yet
- large-scale project validation is still pending

## Development validation

~~~bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
~~~

## License

MIT