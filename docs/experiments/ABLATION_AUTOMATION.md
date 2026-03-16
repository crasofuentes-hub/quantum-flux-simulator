# Automated ablation report

The `flux-sim ablation` command generates a comparative report over the current synthetic benchmark dataset.

## Implemented variants

- structural_baseline
- full_model
- no_lindblad
- no_global_constraint
- no_relativistic_factor
- no_solver

## Interpretation boundary

These variants are implemented as comparative ablations over the current effective scoring stack.

That means:

- the structural baseline is a direct structural comparator,
- the full model uses the current effective analysis path,
- the remaining variants remove selected contribution terms from the effective scoring composition.

This is useful for internal signal attribution and reproducible comparisons.

It is not presented as a full re-simulation with independently re-derived physics for each variant.