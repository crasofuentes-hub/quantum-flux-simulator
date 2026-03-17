use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn seeded_defects_labels_expose_expected_contract() {
    let labels_path = Path::new("datasets/seeded_defects/labels.json");
    let docs_path = Path::new("docs/experiments/SEEDED_DEFECTS.md");

    assert!(labels_path.exists(), "seeded defects labels should exist");
    assert!(docs_path.exists(), "seeded defects doc should exist");

    let labels_text =
        fs::read_to_string(labels_path).expect("seeded defects labels should be readable");
    let payload: Value =
        serde_json::from_str(&labels_text).expect("seeded defects labels should be valid JSON");

    assert_eq!(payload["schema_version"], "1.1.0");
    assert_eq!(payload["dataset_id"], "seeded_defects_v1");

    let cases = payload["cases"]
        .as_array()
        .expect("cases should be an array");
    assert_eq!(
        cases.len(),
        4,
        "seeded defects dataset should contain 4 cases"
    );

    for case_item in cases {
        assert!(case_item["file"].is_string(), "file should be present");
        assert!(
            case_item["expected_class"].is_string(),
            "expected_class should be present"
        );
        assert!(
            case_item["defect_family"].is_string(),
            "defect_family should be present"
        );
        assert!(
            case_item["expected_severity"].is_string(),
            "expected_severity should be present"
        );
        assert!(
            case_item["hypothesis"].is_string(),
            "hypothesis should be present"
        );
        assert!(
            case_item["rationale"].is_string(),
            "rationale should be present"
        );
    }

    let docs_text = fs::read_to_string(docs_path).expect("seeded defects doc should be readable");
    assert!(docs_text.contains("expected severity"));
    assert!(docs_text.contains("rationale"));
    assert!(docs_text.contains("internal reproducible seeded-defect dataset"));
}
