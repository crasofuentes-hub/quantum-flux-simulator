# Seeded defects dataset

This dataset introduces a first internal benchmark layer with explicit seeded defects.

## Scope

Location:

- `datasets/seeded_defects/`

Contents:

- 4 small Python files
- 4 labeled expected classes
- 4 seeded defect families
- machine-readable labels in `labels.json`

## Why this exists

The goal is not to claim real-world defect detection performance yet.

The goal is to provide:

- a more explicit benchmark target than purely synthetic class stubs,
- better experiment traceability,
- a bridge toward future real-world or semi-real defect validation.

## Current limitations

This dataset is still:

- small,
- curated,
- artificial,
- and not externally validated.

It should be described as an internal reproducible seeded-defect dataset, not as a real-world benchmark.