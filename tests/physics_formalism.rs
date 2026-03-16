use flux_sim::core::analysis::{AlgorithmClass, BlockKind, CriticalBlock};
use flux_sim::core::physics::{
    build_effective_physical_model, compute_effective_relativistic_factor,
};

#[test]
fn relativistic_factor_is_physical_gamma_like() {
    let gamma = compute_effective_relativistic_factor(0.8);
    assert!(gamma > 1.0);
    assert!(gamma < 2.0);
}

#[test]
fn effective_model_contains_density_state_and_entropy() {
    let blocks = vec![
        CriticalBlock {
            name: "crypto-core".to_string(),
            kind: BlockKind::CryptoPrimitive,
            estimated_cost: 4.0,
            estimated_logical_qubits: 24,
            information_density: 0.88,
        },
        CriticalBlock {
            name: "iterative-core".to_string(),
            kind: BlockKind::Loop,
            estimated_cost: 2.0,
            estimated_logical_qubits: 12,
            information_density: 0.62,
        },
    ];

    let model = build_effective_physical_model(&blocks, AlgorithmClass::Crypto, 0.01, 0.2, 77.0);

    assert_eq!(model.blocks.len(), 2);
    assert!(model.von_neumann_entropy >= 0.0);
    assert!(model.global_constraint_penalty >= 0.0);
    assert!(model.recommended_qubit_budget > 0);

    for block in model.blocks {
        let trace = block.density_state.rho00_re + block.density_state.rho11_re;
        assert!((trace - 1.0).abs() < 1e-6);
        assert!(block.density_entropy >= 0.0);
    }
}
