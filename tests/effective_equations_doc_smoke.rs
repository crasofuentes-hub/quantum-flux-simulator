use std::fs;
use std::path::Path;

#[test]
fn effective_equations_doc_mentions_parameter_traceability() {
    let path = Path::new("docs/science/EFFECTIVE_EQUATIONS.md");
    assert!(path.exists(), "effective equations doc should exist");

    let text = fs::read_to_string(path).expect("effective equations doc should be readable");

    assert!(text.contains("Parameter traceability per block"));
    assert!(text.contains("Hamiltonian trace"));
    assert!(text.contains("Coupling trace"));
    assert!(text.contains("Lindblad trace"));
    assert!(text.contains("parameter traceability"));
    assert!(text.contains("small regime sweep"));
}
