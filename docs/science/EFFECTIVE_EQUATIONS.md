# Effective equations

This document records the **effective equations actually implemented** in `flux-sim`.

It is not a claim of full physical fidelity.
It is a precise statement of the current effective model.

## 1. Effective 2x2 density state

Each critical block is assigned a 2x2 complex density-like matrix

\[
\rho =
\begin{pmatrix}
\rho_{00} & \rho_{01} \\
\rho_{10} & \rho_{11}
\end{pmatrix}
\]

initialized from information density through a diagonal state

\[
\rho_0 =
\begin{pmatrix}
p & 0 \\
0 & 1-p
\end{pmatrix}
\quad\text{with}\quad
p = \mathrm{clamp}(\text{information\_density}, 0.05, 0.95)
\]

## 2. Effective Hamiltonian

For each block the effective Hamiltonian is

\[
H =
\begin{pmatrix}
E & g \\
g & -E
\end{pmatrix}
\]

where:

- \(E\) is an effective Hamiltonian energy derived from structural cost, information density, block kind and algorithm class,
- \(g\) is an effective coupling strength.

These are modeling quantities, not experimentally calibrated physical observables.

## 3. Effective dissipators

Two effective Lindblad-like operators are used:

### Dephasing-like operator
\[
L_{\phi} =
\begin{pmatrix}
\sqrt{\gamma_{\phi}} & 0 \\
0 & -\sqrt{\gamma_{\phi}}
\end{pmatrix}
\]

### Amplitude-damping-like operator
\[
L_{\mathrm{amp}} =
\begin{pmatrix}
0 & \sqrt{\gamma_{\mathrm{amp}}} \\
0 & 0
\end{pmatrix}
\]

where the rates are effective quantities derived from:

- quantum noise,
- target temperature,
- relativistic beta,
- logical-qubit estimate,
- information density.

## 4. Single effective evolution step

The implemented step is an explicit Euler update of the form

\[
\rho_{n+1} = \rho_n + \Delta t \left(
-i[H,\rho_n]
+
\sum_k \left(
L_k \rho_n L_k^\dagger
-
\frac{1}{2}\{L_k^\dagger L_k,\rho_n\}
\right)
\right)
\]

with a fixed effective step size \( \Delta t = 0.05 \) in the current model-building path.

## 5. Numerical stabilization

After each step, the implementation applies stabilization:

1. hermitianization
\[
\rho \leftarrow \frac{1}{2}(\rho + \rho^\dagger)
\]

2. replacement of non-finite entries by safe finite defaults

3. diagonal floor on real parts for numerical robustness

4. trace normalization
\[
\rho \leftarrow \rho / \mathrm{Re}(\mathrm{Tr}(\rho))
\]

This means the current implementation is designed to preserve a **reasonable physical region numerically**, not to provide a formal CPTP guarantee.

## 6. Effective entropy

Von Neumann entropy is computed from the 2x2 eigenvalue formula using

\[
\lambda_{1,2} = \frac{\mathrm{Tr}(\rho) \pm \sqrt{\mathrm{Tr}(\rho)^2 - 4\det(\rho)}}{2}
\]

and then

\[
S(\rho) = - \sum_i \lambda_i \ln \lambda_i
\]

with small positive flooring to avoid invalid logarithms under numerical error.

## 7. Relativistic factor

The effective relativistic factor is

\[
\gamma_{\mathrm{eff}} = \frac{1}{\sqrt{1-\beta^2}}
\]

with \(\beta\) clamped below 1 for numerical stability.

This is currently used as an effective scaling factor inside the model.
It is not a full relativistic simulation.

## 8. Global constraint

The current model includes an explicit global constraint penalty built from:

- total effective computational energy,
- coherence penalties,
- mean decoherence rate.

This is a scalar regularization term.
It is **not** a Wheeler-DeWitt solver and should not be described as one.

## 9. What is currently preserved or controlled

### Explicitly tested or numerically controlled
- near-unit trace after stabilization
- hermiticity after stabilization
- non-negative or near-physical entropy values
- finite state entries after repeated evolution
- reasonable 2x2 eigenvalue region under tested conditions

### Not formally guaranteed
- full CPTP proof
- convergence under arbitrary parameter regimes
- physical calibration against experimental systems
- exact positivity preservation without stabilization

## 10. Correct current claim

The correct claim is:

> `flux-sim` implements a reproducible effective 2x2 density-evolution model with Lindblad-like dissipative terms, explicit numerical stabilization, internal invariants testing, and documented limits.

The project does **not** currently claim:

- a fully faithful quantum simulator
- a rigorously validated open-quantum-system solver
- a true Wheeler-DeWitt implementation
- external physical validation