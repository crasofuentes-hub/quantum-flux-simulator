# Release Checklist

## Code health
- [ ] cargo fmt --all
- [ ] cargo clippy --all-targets --all-features -- -D warnings
- [ ] cargo test
- [ ] cargo build --release

## Functional smoke
- [ ] cargo run -- analyze examples\my_crypto.py --quantum-noise 0.01 --relativistic 0.8c --target-temp 77K --json-out target\manual-report.json
- [ ] confirm JSON report exists
- [ ] confirm JSON includes physical_model
- [ ] confirm JSON includes solver_summary

## Repository hygiene
- [ ] working tree clean
- [ ] README updated
- [ ] LICENSE present
- [ ] CI workflow present
- [ ] examples present
- [ ] tests present
- [ ] scripts present

## Git
- [ ] commit created
- [ ] tag prepared
- [ ] release notes drafted