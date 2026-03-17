# ROADMAP

This roadmap reflects the current intended direction of the project:

> a reproducible, physics-inspired research tool for code analysis

It does **not** assume that the repository is a physically faithful simulator or a production-grade defect detector.

## Current status

### Implemented
- modular Rust CLI
- analyze / profile / batch / benchmark / ablation / reproduce / consolidate
- effective 2x2 density-state model
- Lindblad-like dissipative evolution
- effective relativistic gamma-like factor
- explicit global constraint penalty
- reproducibility manifests
- synthetic benchmark
- semi-real labeled corpus
- seeded defects dataset
- optional Radon and Semgrep artifact ingestion
- claims matrix and effective equations documentation

### Internally validated
- state invariants tests
- reproducibility smoke tests
- consolidated comparison workflows
- benchmark / ablation / seeded-defects execution paths

### Not yet externally validated
- broad real-world defect detection usefulness
- calibrated comparison against multiple industrial analyzers
- strong large-scale project evaluation
- formal physical validation beyond effective-model documentation

## Phase 1 — Comparative reporting quality
Goal: make consolidated reports more useful for direct comparison.

Planned work:
- summarize Semgrep findings in consolidated markdown
- add clearer cross-file comparison tables
- expose external artifact presence more explicitly in reports

## Phase 2 — Better benchmark evidence
Goal: increase internal evidence quality.

Planned work:
- expand seeded defects dataset
- add more hypotheses per case
- add per-dataset comparison tables
- improve reproducible benchmark summaries

## Phase 3 — Physics-inspired model hardening
Goal: improve the mathematical grounding of the effective model without overstating claims.

Planned work:
- make effective parameters more traceable
- add more numerical tests across parameter regimes
- document implemented equations more rigorously
- improve numerical integration and stabilization discussion

## Phase 4 — External comparison depth
Goal: move beyond bootstrap-level external baselines.

Planned work:
- stronger Semgrep comparison workflow
- more informative Radon comparison ingestion
- optional future comparison against additional analyzers
- explicit metric definitions for cross-tool comparison

## Phase 5 — Parsing and input quality
Goal: improve source understanding beyond extension + keyword heuristics.

Planned work:
- evaluate tree-sitter-based parsing
- improve language-sensitive signal extraction
- strengthen Rust self-analysis workflows
- improve multi-file project handling

## Phase 6 — Presentation and distribution
Goal: improve usability and visibility once technical direction is more stable.

Planned work:
- stronger README examples
- demo notebook / Colab
- metadata polish
- possible rename if final positioning requires it
- optional crates.io publication after scope stabilizes

## Explicit non-goals for the current stage
The project should not currently prioritize:
- marketing-first renaming without stronger validation
- crates.io publication before positioning and docs are fully settled
- exaggerated scientific claims
- large feature expansion without improving evidence quality first