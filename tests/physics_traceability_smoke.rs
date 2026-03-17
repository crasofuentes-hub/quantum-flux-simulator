use flux_sim::core::analysis::analyze_file_with_seed;
use std::path::Path;

#[test]
fn effective_physical_blocks_expose_traceable_parameter_components() {
    let analysis = analyze_file_with_seed(
        Path::new("examples/my_crypto.py"),
        0.01,
        0.2,
        77.0,
        None,
        42,
    )
    .expect("analysis should succeed");

    assert!(
        !analysis.physical_model.blocks.is_empty(),
        "physical model should contain blocks"
    );

    for block in &analysis.physical_model.blocks {
        assert!(block.hamiltonian_trace.estimated_cost.is_finite());
        assert!(block.hamiltonian_trace.information_density.is_finite());
        assert!(block.hamiltonian_trace.kind_factor.is_finite());
        assert!(block.hamiltonian_trace.domain_factor.is_finite());
        assert!(block.hamiltonian_trace.resulting_energy.is_finite());

        assert!(block.coupling_trace.base_coupling.is_finite());
        assert!(block.coupling_trace.density_contribution.is_finite());
        assert!(block.coupling_trace.resulting_coupling.is_finite());

        assert!(block.lindblad_trace.quantum_noise.is_finite());
        assert!(block.lindblad_trace.thermal_factor.is_finite());
        assert!(block.lindblad_trace.relativistic_beta.is_finite());
        assert!(block.lindblad_trace.information_density.is_finite());
        assert!(block.lindblad_trace.dephasing_gamma.is_finite());
        assert!(block.lindblad_trace.amplitude_gamma.is_finite());

        assert!(
            (block.hamiltonian_trace.resulting_energy - block.effective_hamiltonian_energy).abs()
                < 1e-12
        );
        assert!((block.coupling_trace.resulting_coupling - block.coupling_strength).abs() < 1e-12);
        assert!(
            (block.lindblad_trace.dephasing_gamma - block.lindblad_rates.dephasing_gamma).abs()
                < 1e-12
        );
        assert!(
            (block.lindblad_trace.amplitude_gamma - block.lindblad_rates.amplitude_gamma).abs()
                < 1e-12
        );
    }
}
