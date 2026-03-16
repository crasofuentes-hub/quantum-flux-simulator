# State invariants for the effective 2x2 density model

This project currently uses an **effective** 2x2 density-state model per critical block.
It is not claimed to be a full physically faithful quantum simulation.

## Implemented numerical invariants

After each effective Lindblad-Euler update, the state is stabilized to enforce:

- finite matrix entries only,
- Hermitian symmetry,
- positive real diagonal floor,
- trace renormalization to approximately 1,
- bounded coherence magnitude `|rho01| <= sqrt(rho00 * rho11)`.

## What this does and does not mean

These checks are a **numerical hardening layer** for the current effective model.
They reduce obviously non-physical states such as NaN, Inf, broken Hermiticity, or gross 2x2 positivity violations.

They do **not** prove that the full discrete update is an exact completely-positive trace-preserving solver.
The current evolution remains an effective first-order approximation used for reproducible research experiments.