# Ablation plan (initial)

## Goal
Measure whether the full stack adds signal beyond structural analysis.

## Planned variants
1. structural only
2. structural + intermediate model
3. structural + physical model
4. structural + physical model + solver
5. full current stack

## Metrics
- stability_score
- singularity_risk
- collapse_probability
- class-separation behavior across synthetic dataset

## Immediate reproducibility controls
- fixed seed
- fixed benchmark corpus
- schema version recorded
- analysis version recorded