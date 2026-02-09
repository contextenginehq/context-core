use std::fs;
use std::path::Path;
use tempfile::tempdir;

use chrono::{TimeZone, Utc};
use context_core::cache::{CacheBuildConfig, CacheBuilder, CacheManifest};
use context_core::document::{Document, DocumentId, Metadata};
use context_core::selection::ContextSelector;
use context_core::types::{Query, SelectionResult};

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

    // Cache versions must match
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
