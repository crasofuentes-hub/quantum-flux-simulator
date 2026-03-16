# Synthetic benchmark baseline

This directory contains a minimal synthetic benchmark and ablation-oriented corpus for `flux-sim v0.2.0`.

## Goal
Provide:
- reproducible seed-controlled runs
- simple class separation checks
- first ablation-ready baseline

## Included files
- `dataset/crypto_heavy.py`
- `dataset/numerical_heavy.py`
- `dataset/ml_heavy.py`
- `dataset/general_light.py`

## Intended use
Run:
- single-file analysis
- batch analysis
- compare class separation
- compare stability and collapse metrics under fixed seed