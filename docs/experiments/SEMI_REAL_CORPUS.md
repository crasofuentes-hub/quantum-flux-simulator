# Semi-real corpus

This experiment stage introduces a small **semi-real** corpus under `benchmarks/semi_real/`.

## Why this corpus exists

The original benchmark dataset is intentionally small and synthetic.
This semi-real corpus is meant to provide a slightly more plausible bridge layer before attempting a broader real-world benchmark.

## Case hypotheses

### 1. crypto_service.py
Hypothesis:
- the classifier should detect crypto-oriented structure,
- structural and effective-model risk should remain moderate rather than trivial,
- ablation should show some sensitivity to the full effective stack.

### 2. numerical_solver.py
Hypothesis:
- the classifier should detect numerical structure,
- iterative signals should appear through loops and numerical keywords,
- the effective model should differ from the structural baseline but not collapse into the crypto profile.

### 3. ml_training_loop.py
Hypothesis:
- the classifier should detect ML-oriented vocabulary,
- the case should show hotspot concentration around gradient/loss/optimizer style terms,
- ablation should highlight the contribution of solver and effective-model terms.

### 4. general_pipeline.py
Hypothesis:
- the classifier should remain in the general class,
- scores should be comparatively milder than the specialized cases,
- this file acts as a control-like workload.

## Interpretation boundary

This corpus improves realism relative to the initial synthetic dataset, but it still remains:
- small,
- curated,
- and non-representative of broad industrial software.

It should be described as a semi-real reproducible benchmark layer, not as external validation.