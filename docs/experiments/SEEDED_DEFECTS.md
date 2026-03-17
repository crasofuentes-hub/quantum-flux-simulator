# Seeded defects dataset

This dataset introduces an internal benchmark layer with explicit seeded defects.

## Scope

Location:

- `datasets/seeded_defects/`

Contents:

- 4 small Python files
- 4 labeled expected classes
- 4 seeded defect families
- machine-readable labels in `labels.json`
- explicit expected severity and rationale per case

## Why this exists

The goal is not to claim real-world defect detection performance.

The goal is to provide:

- a more explicit benchmark target than purely synthetic class stubs,
- better experiment traceability,
- a bridge toward future real-world or semi-real defect validation,
- and clearer per-case reasoning about what each sample is meant to stress.

## Label contract

Each case currently records:

- `file`
- `expected_class`
- `defect_family`
- `expected_severity`
- `hypothesis`
- `rationale`

## Current limitations

This dataset is still:

- small,
- curated,
- artificial,
- and not externally validated.

It should be described as an internal reproducible seeded-defect dataset, not as a real-world benchmark.

## Correct current use

Use this dataset for:

- benchmark regression checks,
- consolidated comparison runs,
- Flux vs external-baseline exploratory comparisons,
- checking whether classification and stress signals remain stable across known seeded cases.