# Seeded defects benchmark

This document records the intended use of `datasets/seeded_defects/` as an internal reproducible benchmark input for `flux-sim benchmark`.

## Benchmark purpose

The purpose of this benchmark is to verify that the current pipeline can:

- ingest the seeded-defects dataset reproducibly,
- preserve the expected class labels in output,
- produce a stable benchmark artifact for comparison workflows.

## What this benchmark does validate

It validates that:

- the dataset is structurally usable by the benchmark command,
- the expected class labels are represented in the result,
- the benchmark artifact can be used in later comparison steps.

## What this benchmark does not validate

It does not by itself validate:

- real-world defect detection quality,
- calibrated severity prediction,
- external industrial usefulness,
- superiority against external analyzers.

## Expected current dataset shape

The benchmark should currently process:

- 4 files
- 4 labeled cases
- classes: crypto, numerical, ml, general