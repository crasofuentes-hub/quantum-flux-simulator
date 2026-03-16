# External baseline bootstrap

This experiment introduces an external Python baseline using Radon.

## Why Radon
Radon provides:
- cyclomatic complexity
- maintainability index
- JSON export

## Goal
Compare, on the same synthetic dataset:
- structural baseline
- flux-sim model metrics
- Radon complexity and maintainability metrics

## Interpretation
This is not a claim that flux-sim replaces static analyzers.
This experiment is intended to show:
- where flux-sim agrees with conventional structural metrics,
- where it differs,
- and whether its signals appear complementary rather than redundant.