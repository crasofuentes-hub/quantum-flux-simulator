# flux-sim

[![CI](https://github.com/crasofuentes-hub/quantum-flux-simulator/actions/workflows/ci.yml/badge.svg)](https://github.com/crasofuentes-hub/quantum-flux-simulator/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**flux-sim** is a Rust CLI for structurally analyzing source code and mapping it into an explicit, effective computational-physics model.

## Current capabilities

- structural source analysis
- classification into `crypto`, `numerical`, `ml`, `general`
- intermediate model with critical blocks, hotspots, and information channels
- effective physical model with Lindblad-inspired metrics
- relativistic runtime penalty
- Wheeler-DeWitt-like global penalty
- Monte Carlo stress simulation with renormalized tail behavior
- JSON output for downstream tooling
- PNG visualization output for quick inspection

## CLI examples

```powershell
cargo run -- analyze examples\my_crypto.py --quantum-noise 0.01 --relativistic 0.8c --target-temp 77K
cargo run -- analyze examples\my_crypto.py --quantum-noise 0.01 --relativistic 0.8c --target-temp 77K --json-out target\manual-report.json
cargo run -- analyze examples\my_crypto.py --quantum-noise 0.01 --relativistic 0.8c --target-temp 77K --plot target\manual-plot.png
cargo run -- profile examples\navier_stub.py --algorithm-class numerical --quantum-noise 0.02 --target-temp 300K --plot target\navier-plot.png