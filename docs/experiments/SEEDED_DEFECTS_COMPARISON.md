# Seeded defects comparison notes

This note records the intended comparative interpretation of the current `seeded_defects` benchmark set.

## Comparative expectations by class

### crypto
Expected to remain clearly crypto-oriented because of:
- hashing vocabulary
- verification flow
- looped record checks

### numerical
Expected to remain clearly numerical because of:
- iterative accumulation
- integration-like update flow
- aggregation bias defect

### ml
Expected to remain clearly ML-oriented because of:
- relu / softmax / loss / gradient vocabulary
- training-step structure
- stale update defect

### general
Expected to remain general rather than domain-specialized because of:
- normalization pipeline
- aggregation workflow
- empty-case reporting defect

## Benchmark interpretation boundary

This comparison layer is useful for:

- checking that the benchmark remains structurally coherent,
- preserving class-specific signals across seeded-defect cases,
- supporting consolidated internal comparison workflows.

It is not yet a real-world quality benchmark.