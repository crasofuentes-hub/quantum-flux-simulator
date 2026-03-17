# Semi-real labels

The semi-real corpus now includes a machine-readable label file:

- `benchmarks/semi_real/labels.json`

## Why this exists

The goal is to make the semi-real benchmark layer more explicit and auditable.

This label file records:

- expected class
- case type
- hypothesis
- short notes per case

## Current boundary

This does **not** transform the corpus into an externally validated benchmark.

It only improves:

- reproducibility
- explicit labeling
- experiment traceability
- clarity of internal benchmark assumptions