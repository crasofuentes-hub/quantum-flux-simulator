# Claims matrix

This document defines what `flux-sim` currently **implements**, what it has only **benchmarked internally**, and what it **does not claim**.

## Status categories

- **implemented**: functionality exists in code and is exercised by tests or smoke workflows
- **benchmarked internally**: functionality is compared only through internal or semi-real reproducible experiments
- **externally referenced**: an external tool or method is connected by artifact ingestion or documented workflow, but not executed natively by Rust
- **not validated externally**: no strong external scientific or industrial validation has been completed
- **not claimed**: the project explicitly does not claim this capability

## Current matrix

| Area | Status | Notes |
|---|---|---|
| Effective 2x2 state evolution | implemented | Explicit effective density evolution and stabilization are present |
| Basic state invariants | implemented | Trace / hermiticity / non-finite protection covered by tests |
| Full physically faithful quantum simulation | not claimed | Current model remains effective and simplified |
| Lindblad-like effective dissipation | implemented | Present as effective discrete evolution, not a full validated physical solver |
| Relativistic factor | implemented | Effective gamma-like factor is implemented and tested at a basic level |
| Wheeler-DeWitt-equivalent physics | not claimed | Only an explicit global constraint penalty exists; no true Wheeler-DeWitt solver is claimed |
| Reproducible analyze / batch / reproduce / ablation / consolidate flows | implemented | Seeded workflows and artifact generation are in place |
| Unified experimental artifact lineage across all commands | partially implemented | Reproduce has a dedicated manifest; broader unified experiment manifest is still pending |
| Synthetic benchmark | implemented | Internal reproducible benchmark exists |
| Semi-real benchmark corpus | implemented | Small curated corpus exists and is documented |
| Broad real-world validation corpus | not validated externally | Current corpus is too small and curated to claim broad validity |
| External Radon comparison | externally referenced | Artifact bootstrap exists and consolidated report can ingest generated comparison JSON |
| Native Rust execution of Radon / external baseline toolchain | not claimed | Rust only ingests external artifacts when present |
| Comparison against Semgrep / SonarQube / CodeQL | not validated externally | Not implemented yet |
| Large-scale multi-file performance validation | not validated externally | Current support exists, but no serious scaling evidence is claimed |
| Scientifically validated predictive defect detection | not claimed | The project does not currently claim validated defect prediction performance |

## Guidance for README / papers / release notes

The project should currently be described as:

> a reproducible research prototype with an explicit effective model, internal benchmark layers, semi-real corpus support, and optional ingestion of external baseline artifacts

It should **not** currently be described as:

- a physically faithful quantum simulator
- a validated Wheeler-DeWitt solver
- a production-grade defect detector validated on broad industrial datasets
- a native external-analysis orchestration platform