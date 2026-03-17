# Semgrep baseline

This experiment adds an initial external static-analysis baseline using Semgrep.

## Scope

The current baseline targets:

- `datasets/seeded_defects/`

and produces:

- `target/semgrep-results.json`
- `target/semgrep-summary.json`

## Purpose

The purpose is not to prove that `flux-sim` outperforms Semgrep.

The purpose is to establish a first reproducible comparison surface between:

- Flux internal effective-model signals
- externally generated Semgrep findings

## Current limitations

This baseline is currently:

- externally executed through PowerShell and Python
- not yet ingested into Rust-native consolidated comparison
- dependent on Semgrep default/auto rules rather than a project-specific tuned ruleset

## Correct claim

At this stage, the repository can claim:

- Semgrep external baseline bootstrap exists
- Semgrep outputs can be generated reproducibly for the seeded defects dataset

It should **not** yet claim:

- full comparative evaluation quality
- calibrated defect-detection benchmarking
- Rust-native Semgrep orchestration