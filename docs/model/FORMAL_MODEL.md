# flux-sim formal model (v0.2.0 draft)

## Scope

This document defines the **implemented mathematical model** used by `flux-sim`.
It does **not** claim full physical realism.
It defines an **effective open-system model** with explicit equations.

## 1. Effective state per critical block

Each critical block is assigned a 2x2 density matrix:

rho = [[rho00, rho01],
       [rho10, rho11]]

Subject to:
- rho = rho^\dagger
- tr(rho) = 1
- rho approximately positive semidefinite after clipping and renormalization

Initial state:
rho_0 = [[p, 0],
         [0, 1-p]]

where p is derived from normalized information density.

## 2. Effective Hamiltonian

For each block we define:

H = [[E, g],
     [g, -E]]

where:
- E = effective_hamiltonian
- g = coupling_strength

This is a minimal two-level effective system.

## 3. Lindblad evolution

We evolve rho using the discrete step:

rho_(t+1) = rho_t + dt * (
  -i [H, rho_t]
  + sum_k (L_k rho_t L_k^\dagger - 1/2 {L_k^\dagger L_k, rho_t})
)

Implemented Lindblad operators:
- dephasing:
  L_phi = sqrt(gamma_phi) * [[1, 0], [0, -1]]
- amplitude damping:
  L_amp = sqrt(gamma_amp) * [[0, 1], [0, 0]]

After each step:
- Hermitian symmetrization is applied
- trace is renormalized to 1
- small negative eigenvalue drift is clipped indirectly through diagonal stabilization

## 4. Von Neumann entropy

For the 2x2 density matrix, entropy is computed from eigenvalues lambda_i:

S(rho) = - sum_i lambda_i ln(lambda_i)

with:
- real-part extraction for small numerical imaginary drift
- clipping lambda_i >= 1e-12
- normalization if needed

## 5. Effective relativistic factor

We define:

gamma_eff = 1 / sqrt(1 - beta^2)

where beta in [0, 1).

This is used as an effective time dilation factor only.
It is not claimed as a full relativistic spacetime solver.

## 6. Global constraint penalty

We replace the misleading Wheeler-DeWitt naming with an explicit global constraint:

C_global = alpha * sum_i H_i
         + beta_p * sum_i P_i
         + beta_d * decoherence_rate

where:
- H_i = computational energy cost proxy of block i
- P_i = coherence penalty of block i
- decoherence_rate = mean dissipative rate across blocks

This is an effective global consistency penalty, not a canonical quantum gravity equation.

## 7. Monte Carlo layer

Monte Carlo operates on top of the effective block model using deterministic pseudo-random perturbations:
- quantum perturbation
- thermal perturbation
- relativistic perturbation
- tail renormalization

This layer estimates stress distribution, collapse probability, and singularity-style instability risk.

## 8. Claims allowed after this implementation

Allowed:
- effective Lindblad density evolution
- explicit von Neumann entropy calculation
- effective relativistic scaling term
- explicit global constraint penalty

Not allowed:
- full Lindblad many-body simulation
- Wheeler-DeWitt equation solved
- exact relativistic computation model
- hardware-faithful quantum simulation