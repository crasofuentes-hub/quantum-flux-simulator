use std::fs;
use std::path::Path;

#[test]
fn parser_evolution_doc_records_tree_sitter_deferral_rationale() {
    let path = Path::new("docs/architecture/PARSER_EVOLUTION.md");
    assert!(path.exists(), "parser evolution doc should exist");

    let text = fs::read_to_string(path).expect("parser evolution doc should be readable");

    assert!(text.contains("tree-sitter"));
    assert!(text.contains("Why tree-sitter is not introduced yet"));
    assert!(text.contains("Trigger conditions for future tree-sitter adoption"));
    assert!(text.contains("practical multi-language file handling"));
}
