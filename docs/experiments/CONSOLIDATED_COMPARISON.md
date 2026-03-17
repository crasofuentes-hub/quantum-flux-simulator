# Consolidated comparison

The `flux-sim consolidate` command produces a single reproducible report that joins:

- the internal synthetic benchmark,
- the automated ablation comparison,
- and the current external baseline status.

## Current scope

This consolidated report includes:

- structural baseline metrics,
- full current flux-sim effective-model metrics,
- ablation summaries over the current effective scoring stack,
- explicit status for the Radon external baseline experiment.

## Important limitation

The external Radon baseline is currently represented as a tracked experimental reference.

This means:

- the bootstrap asset exists,
- the documentation exists,
- but automatic ingestion of Radon result data into the Rust-native consolidated report is not implemented yet.

Therefore, the consolidated report is currently strongest as an internally reproducible comparison layer, not yet as a fully automated external-tool comparison pipeline.