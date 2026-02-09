use context_core::types::{
  SelectedDocument, SelectionMetadata, SelectionResult, SelectionWhy
};

// This test verifies that the selection output structs serialize 
// exactly as the spec requires, ensuring the API contract is met.
// It constructs the types manually to avoid dependency on logic implementation.

#[test]
fn golden_selection_output_serialization() {
    // 1. Construct a mock "SelectedDocument"
    let why = SelectionWhy {
        query_terms: vec!["deployment".to_string()],
        term_matches: 12,
        total_words: 156,
    };

    let doc = SelectedDocument {
        id: "docs/deployment.md".to_string(),
        version: "sha256:mock".to_string(),
        content: "Content...".to_string(),
        score: 0.92,
        tokens: 847,
        why,
    };

    // 2. Construct SelectionMetadata
    let meta = SelectionMetadata {
        query: "deployment".to_string(),
        budget: 4000,
        tokens_used: 3241,
        documents_considered: 42,
        documents_selected: 3,
        documents_excluded_by_budget: 9,
    };

    // 3. Construct SelectionResult
    let result = SelectionResult {
        documents: vec![doc],
        selection: meta,
    };

    // 4. Serialize
    let json_str = serde_json::to_string_pretty(&result).unwrap();

    // 5. Verify Structure & Key Order (Golden Check)
    // We check that specific keys appear in expected relative order 
    // or at least exist. Since struct field order determines JSON 
    // order in default serde, this serves as a regression test for struct layout.

    let doc_start = json_str.find("\"documents\":").expect("Missing documents key");
    let sel_start = json_str.find("\"selection\":").expect("Missing selection key");

    // The spec example shows documents array first, then selection metadata.
    // Let's verify our struct order produces this.
    // struct SelectionResult { documents: ..., selection: ... }
    assert!(doc_start < sel_start, "documents should appear before selection metadata");

    // Verify fields inside a document
    // id, version, content, score, tokens, why
    // Note: Serde default ordering is Definition Order.
    let id_pos = json_str.find("\"id\":").unwrap();
    let score_pos = json_str.find("\"score\":").unwrap();
    let why_pos = json_str.find("\"why\":").unwrap();
    
    assert!(id_pos < score_pos);
    assert!(score_pos < why_pos);

    // 6. JSON Snapshot Check
    // Enforce byte-level determinism (ignoring whitespace differences in formatting)
    const EXPECTED_JSON: &str = r#"{
      "documents": [
        {
          "id": "docs/deployment.md",
          "version": "sha256:mock",
          "content": "Content...",
          "score": 0.92,
          "tokens": 847,
          "why": {
            "query_terms": [
              "deployment"
            ],
            "term_matches": 12,
            "total_words": 156
          }
        }
      ],
      "selection": {
        "query": "deployment",
        "budget": 4000,
        "tokens_used": 3241,
        "documents_considered": 42,
        "documents_selected": 3,
        "documents_excluded_by_budget": 9
      }
    }"#;

    // Normalize strings for comparison (remove all whitespace)
    let normalized_actual: String = json_str.chars().filter(|c| !c.is_whitespace()).collect();
    let normalized_expected: String = EXPECTED_JSON.chars().filter(|c| !c.is_whitespace()).collect();

    assert_eq!(normalized_actual, normalized_expected, "JSON structure mismatch against golden snapshot");

    // 7. Roundtrip check & Detailed Field Verification
    let deserialized: SelectionResult = serde_json::from_str(&json_str).expect("Deserialization failed");
    
    // Check SelectionMetadata fields
    assert_eq!(deserialized.selection.budget, 4000);
    assert_eq!(deserialized.selection.query, "deployment");
    assert_eq!(deserialized.selection.tokens_used, 3241);
    assert_eq!(deserialized.selection.documents_considered, 42);
    assert_eq!(deserialized.selection.documents_selected, 3);
    assert_eq!(deserialized.selection.documents_excluded_by_budget, 9);
    
    assert_eq!(deserialized.documents.len(), 1);

    // Check all document fields explicitly
    let doc = &deserialized.documents[0];
    assert_eq!(doc.id, "docs/deployment.md");
    assert_eq!(doc.version, "sha256:mock");
    assert_eq!(doc.content, "Content...");
    assert!((doc.score - 0.92).abs() < f32::EPSILON);
    assert_eq!(doc.tokens, 847);

    // Check why fields fully
    let why = &doc.why;
    assert_eq!(why.query_terms, vec!["deployment".to_string()]);
    assert_eq!(why.term_matches, 12);
    assert_eq!(why.total_words, 156);
}
