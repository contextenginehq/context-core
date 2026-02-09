use std::fs;
use std::path::Path;

use chrono::{TimeZone, Utc};
use context_core::cache::{CacheBuildConfig, CacheBuilder, CacheManifest, ManifestDocumentEntry};
use context_core::document::{Document, DocumentId, Metadata};
use context_core::selection::ContextSelector;
use context_core::types::{Query, SelectionResult, SelectedDocument, SelectionMetadata, SelectionWhy};
use serde_json::Value;
use tempfile::tempdir;

fn make_id(s: &str) -> DocumentId {
    let root = Path::new("/root");
    let path = root.join(s);
    DocumentId::from_path(root, &path).unwrap()
}

fn make_doc(id_str: &str, content: &str) -> Document {
    let id = make_id(id_str);
    Document::ingest(
        id,
        id_str.to_string(),
        content.as_bytes().to_vec(),
        Metadata::default(),
    )
    .unwrap()
}

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
    let doc_start = json_str.find("\"documents\":").expect("Missing documents key");
    let sel_start = json_str.find("\"selection\":").expect("Missing selection key");

    assert!(doc_start < sel_start, "documents should appear before selection metadata");

    let id_pos = json_str.find("\"id\":").unwrap();
    let score_pos = json_str.find("\"score\":").unwrap();
    let why_pos = json_str.find("\"why\":").unwrap();

    assert!(id_pos < score_pos);
    assert!(score_pos < why_pos);

    // 6. JSON Snapshot Check
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

    let normalized_actual: String = json_str.chars().filter(|c| !c.is_whitespace()).collect();
    let normalized_expected: String = EXPECTED_JSON.chars().filter(|c| !c.is_whitespace()).collect();

    assert_eq!(normalized_actual, normalized_expected, "JSON structure mismatch against golden snapshot");

    // 7. Roundtrip check & Detailed Field Verification
    let deserialized: SelectionResult = serde_json::from_str(&json_str).expect("Deserialization failed");

    assert_eq!(deserialized.selection.budget, 4000);
    assert_eq!(deserialized.selection.query, "deployment");
    assert_eq!(deserialized.selection.tokens_used, 3241);
    assert_eq!(deserialized.selection.documents_considered, 42);
    assert_eq!(deserialized.selection.documents_selected, 3);
    assert_eq!(deserialized.selection.documents_excluded_by_budget, 9);

    assert_eq!(deserialized.documents.len(), 1);

    let doc = &deserialized.documents[0];
    assert_eq!(doc.id, "docs/deployment.md");
    assert_eq!(doc.version, "sha256:mock");
    assert_eq!(doc.content, "Content...");
    assert!((doc.score - 0.92).abs() < f32::EPSILON);
    assert_eq!(doc.tokens, 847);

    let why = &doc.why;
    assert_eq!(why.query_terms, vec!["deployment".to_string()]);
    assert_eq!(why.term_matches, 12);
    assert_eq!(why.total_words, 156);
}

#[test]
fn golden_end_to_end_selection_is_deterministic() {
    // ------------------------------------------------------------
    // 1. Prepare deterministic input documents
    // ------------------------------------------------------------
    let docs = vec![
        make_doc(
            "docs/deployment.md",
            "Deployment deployment deployment guide.",
        ),
        make_doc("docs/security.md", "Security hardening guide."),
        make_doc("docs/overview.md", "Overview of the system."),
    ];

    let config = CacheBuildConfig::v0();

    // ------------------------------------------------------------
    // 2. Build cache twice in two separate directories
    // ------------------------------------------------------------
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    let cache_path1 = dir1.path().join("cache");
    let cache_path2 = dir2.path().join("cache");

    let builder = CacheBuilder::new(config.clone());

    let cache1 = builder.build(docs.clone(), &cache_path1).unwrap();
    let cache2 = builder.build(docs.clone(), &cache_path2).unwrap();

    assert_eq!(
        cache1.manifest.cache_version,
        cache2.manifest.cache_version
    );

    // Cache bytes must match (manifest + index + document files)
    // Normalize created_at before comparing manifests (informational field)
    let manifest_bytes_1 = fs::read(cache_path1.join("manifest.json")).unwrap();
    let manifest_bytes_2 = fs::read(cache_path2.join("manifest.json")).unwrap();

    let mut manifest_1: CacheManifest = serde_json::from_slice(&manifest_bytes_1).unwrap();
    let mut manifest_2: CacheManifest = serde_json::from_slice(&manifest_bytes_2).unwrap();
    let fixed_time = Utc.timestamp_opt(0, 0).unwrap();
    manifest_1.created_at = fixed_time;
    manifest_2.created_at = fixed_time;

    let normalized_manifest_1 = serde_json::to_string_pretty(&manifest_1).unwrap();
    let normalized_manifest_2 = serde_json::to_string_pretty(&manifest_2).unwrap();
    assert_eq!(normalized_manifest_1, normalized_manifest_2, "manifest.json mismatch");

    let index_bytes_1 = fs::read(cache_path1.join("index.json")).unwrap();
    let index_bytes_2 = fs::read(cache_path2.join("index.json")).unwrap();
    assert_eq!(index_bytes_1, index_bytes_2, "index.json mismatch");

    for entry in &cache1.manifest.documents {
        let doc_bytes_1 = fs::read(cache_path1.join(&entry.file)).unwrap();
        let doc_bytes_2 = fs::read(cache_path2.join(&entry.file)).unwrap();
        assert_eq!(doc_bytes_1, doc_bytes_2, "document file mismatch: {}", entry.file);
    }

    // ------------------------------------------------------------
    // 3. Run selection twice (separate cache instances)
    // ------------------------------------------------------------
    let selector = ContextSelector::default();

    let query = Query::new("deployment");
    let budget = 10_000;

    let result1: SelectionResult = selector.select(&cache1, query.clone(), budget).unwrap();
    let result2: SelectionResult = selector.select(&cache2, query.clone(), budget).unwrap();

    // ------------------------------------------------------------
    // 4. Serialize both results
    // ------------------------------------------------------------
    let json1 = serde_json::to_string_pretty(&result1).unwrap();
    let json2 = serde_json::to_string_pretty(&result2).unwrap();

    // ------------------------------------------------------------
    // 5. Byte-for-byte determinism check
    // ------------------------------------------------------------
    assert_eq!(json1, json2, "Selection output is not deterministic");

    // ------------------------------------------------------------
    // 6. Snapshot assertion (freeze contract)
    // ------------------------------------------------------------
    let expected = r#"{
  "documents": [
    {
      "id": "docs/deployment.md",
      "version": "sha256:19835fc46fd47b1e6bc19778f76396e900c217191ff1bef2cb4e138308da1a72",
      "content": "Deployment deployment deployment guide.",
      "score": 0.75,
      "tokens": 10,
      "why": {
        "query_terms": [
          "deployment"
        ],
        "term_matches": 3,
        "total_words": 4
      }
    },
    {
      "id": "docs/overview.md",
      "version": "sha256:02569adbe8a19f6b28f7bb9a31863c070126ad6cbc809222cbe725c8c3c30325",
      "content": "Overview of the system.",
      "score": 0.0,
      "tokens": 6,
      "why": {
        "query_terms": [
          "deployment"
        ],
        "term_matches": 0,
        "total_words": 4
      }
    },
    {
      "id": "docs/security.md",
      "version": "sha256:c070a5c47a9fbba3e807b972cc753ea192b0dd6e6af86d2c3e2841b7bc0fd644",
      "content": "Security hardening guide.",
      "score": 0.0,
      "tokens": 7,
      "why": {
        "query_terms": [
          "deployment"
        ],
        "term_matches": 0,
        "total_words": 3
      }
    }
  ],
  "selection": {
    "query": "deployment",
    "budget": 10000,
    "tokens_used": 23,
    "documents_considered": 3,
    "documents_selected": 3,
    "documents_excluded_by_budget": 0
  }
}"#;

    assert_eq!(json1.trim(), expected.trim(), "Golden snapshot mismatch");
}

#[test]
fn golden_document_serialization() {
    let source = "docs/deployment.md";
    let content = "# Deployment\n\nThis guide...";

    let root = std::path::Path::new(".");
    let source_path = std::path::Path::new("./docs/deployment.md");
    let id = DocumentId::from_path(root, source_path).unwrap();

    let mut metadata = Metadata::new();
    metadata.insert_string("title", "Deployment");
    metadata.insert_number("byte_size", 1842);
    metadata.insert_number("line_count", 47);

    let doc = Document::ingest(
        id,
        source.to_string(),
        content.as_bytes().to_vec(),
        metadata,
    )
    .unwrap();

    let json_str = serde_json::to_string(&doc).unwrap();

    let id_pos = json_str.find("\"id\":").unwrap();
    let ver_pos = json_str.find("\"version\":").unwrap();
    let src_pos = json_str.find("\"source\":").unwrap();
    let cnt_pos = json_str.find("\"content\":").unwrap();
    let meta_pos = json_str.find("\"metadata\":").unwrap();

    assert!(id_pos < ver_pos);
    assert!(ver_pos < src_pos);
    assert!(src_pos < cnt_pos);
    assert!(cnt_pos < meta_pos);

    let _parsed: Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn golden_manifest_serialization() {
    let config = CacheBuildConfig {
        version: "1".to_string(),
        hash_algorithm: "sha256".to_string(),
    };

    let id_str = "docs/deployment.md";
    let root = std::path::Path::new(".");
    let id = DocumentId::from_path(root, std::path::Path::new("./docs/deployment.md")).unwrap();
    let doc = Document::ingest(
        id.clone(),
        id_str.to_string(),
        "content".as_bytes().to_vec(),
        Metadata::new(),
    )
    .unwrap();

    let entry = ManifestDocumentEntry {
        id,
        version: doc.version.clone(),
        file: "documents/abc.json".to_string(),
    };

    let manifest = CacheManifest {
        cache_version: "sha256:mock".to_string(),
        build_config: config,
        created_at: chrono::Utc::now(),
        document_count: 1,
        documents: vec![entry],
    };

    let json_str = serde_json::to_string(&manifest).unwrap();

    let cv_pos = json_str.find("\"cache_version\":").unwrap();
    let bc_pos = json_str.find("\"build_config\":").unwrap();
    let ca_pos = json_str.find("\"created_at\":").unwrap();
    let dc_pos = json_str.find("\"document_count\":").unwrap();
    let d_pos = json_str.find("\"documents\":").unwrap();

    assert!(cv_pos < bc_pos);
    assert!(bc_pos < ca_pos);
    assert!(ca_pos < dc_pos);
    assert!(dc_pos < d_pos);
}
