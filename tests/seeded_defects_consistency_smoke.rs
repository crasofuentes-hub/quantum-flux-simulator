use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn seeded_defects_docs_and_labels_remain_consistent() {
    let labels_path = Path::new("datasets/seeded_defects/labels.json");
    let benchmark_doc_path = Path::new("docs/experiments/SEEDED_DEFECTS_BENCHMARK.md");
    let comparison_doc_path = Path::new("docs/experiments/SEEDED_DEFECTS_COMPARISON.md");

    assert!(labels_path.exists(), "labels.json should exist");
    assert!(benchmark_doc_path.exists(), "benchmark doc should exist");
    assert!(comparison_doc_path.exists(), "comparison doc should exist");

    let labels_text = fs::read_to_string(labels_path).expect("labels.json should be readable");
    let payload: Value =
        serde_json::from_str(&labels_text).expect("labels.json should be valid JSON");

    let cases = payload["cases"]
        .as_array()
        .expect("cases should be an array");
    assert_eq!(
        cases.len(),
        4,
        "seeded defects should still contain 4 cases"
    );

    let benchmark_doc =
        fs::read_to_string(benchmark_doc_path).expect("benchmark doc should be readable");
    let comparison_doc =
        fs::read_to_string(comparison_doc_path).expect("comparison doc should be readable");

    for expected_class in ["crypto", "numerical", "ml", "general"] {
        let found = cases
            .iter()
            .any(|case_item| case_item["expected_class"] == expected_class);
        assert!(found, "missing expected_class in labels: {expected_class}");
        assert!(
            comparison_doc.contains(expected_class),
            "comparison doc should mention class: {expected_class}"
        );
    }

    assert!(benchmark_doc.contains("4 files"));
    assert!(benchmark_doc.contains("classes: crypto, numerical, ml, general"));
    assert!(comparison_doc.contains("not yet a real-world quality benchmark"));
}
