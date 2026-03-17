# Semi-real benchmark corpus

This corpus is a small curated set of **semi-realistic** Python cases.

It is not claimed to be a production-scale or statistically representative corpus.
Its purpose is to move beyond purely synthetic labels while preserving reproducibility and interpretability.

## Cases

- `crypto_service.py`
  - lightweight request-oriented crypto workflow
  - expected class: crypto

- `numerical_solver.py`
  - small integration / solve style numerical workflow
  - expected class: numerical

- `ml_training_loop.py`
  - lightweight ML-style training step with gradient/loss/optimizer vocabulary
  - expected class: ml

- `general_pipeline.py`
  - ordinary data normalization and aggregation pipeline
  - expected class: general

## Use

This corpus is suitable for:
- `flux-sim benchmark`
- `flux-sim ablation`
- `flux-sim consolidate`

## Limitation

These files are still compact synthetic-to-semi-real proxies.
They are more plausible than the original minimal dataset, but they are not a broad real-world benchmark yet.