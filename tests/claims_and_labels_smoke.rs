use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn claims_matrix_and_semi_real_labels_exist_and_are_coherent() {
    let claims_path = Path::new("docs/science/CLAIMS_MATRIX.md");
    let labels_path = Path::new("benchmarks/semi_real/labels.json");

    assert!(claims_path.exists(), "claims matrix should exist");
    assert!(labels_path.exists(), "semi-real labels should exist");

    let claims_text = fs::read_to_string(claims_path).expect("claims matrix should be readable");
    assert!(claims_text.contains("Effective 2x2 state evolution"));
    assert!(claims_text.contains("Full physically faithful quantum simulation"));
    assert!(claims_text.contains("not claimed"));
    assert!(claims_text.contains("externally referenced"));

    let labels_text = fs::read_to_string(labels_path).expect("semi-real labels should be readable");
    let payload: Value =
        serde_json::from_str(&labels_text).expect("semi-real labels should be valid JSON");

    assert_eq!(payload["schema_version"], "1.0.0");
    assert_eq!(payload["corpus_id"], "semi_real_v1");

    let cases = payload["cases"]
        .as_array()
        .expect("cases should be an array");
    assert_eq!(
        cases.len(),
        4,
        "semi-real corpus should expose 4 labeled cases"
    );

    let expected = [
        ("crypto_service.py", "crypto"),
        ("numerical_solver.py", "numerical"),
        ("ml_training_loop.py", "ml"),
        ("general_pipeline.py", "general"),
    ];

    for (file, class_name) in expected {
        let found = cases.iter().any(|case_item| {
            case_item["file"] == file && case_item["expected_class"] == class_name
        });
        assert!(found, "missing labeled case: {file} -> {class_name}");
    }
}
