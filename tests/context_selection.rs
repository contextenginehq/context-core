use context_core::cache::{CacheBuilder, CacheBuildConfig};
use context_core::selection::ContextSelector;
use context_core::types::Query;
use context_core::document::{Document, DocumentId, Metadata};
use tempfile::tempdir;
use std::path::Path;

fn make_id(s: &str) -> DocumentId {
    let root = Path::new("/root");
    let path = root.join(s);
    DocumentId::from_path(root, &path).unwrap()
}

fn make_doc(id_str: &str, content: &str) -> Document {
    let id = make_id(id_str);
    Document::ingest(
        id,
        "test_source".to_string(),
        content.as_bytes().to_vec(),
        Metadata::default(),
    )
    .unwrap()
}

#[test]
fn test_selection_budget_zero() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().join("cache_zero");

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);

    let doc = make_doc("test.md", "hello world");
    let cache = builder.build(vec![doc], &cache_dir).unwrap();

    let selector = ContextSelector::default();
    let query = Query::new("hello");

    let result = selector.select(&cache, query, 0).unwrap();

    assert_eq!(result.documents.len(), 0, "No documents should be selected with budget 0");
    assert_eq!(result.selection.tokens_used, 0);
    assert_eq!(result.selection.documents_considered, 1);
    assert_eq!(result.selection.documents_excluded_by_budget, 1);
}

#[test]
fn test_selection_sorting_and_budget() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().join("cache_sort");

    // ApproxTokenCounter: ceil(len/4)
    // A: "a" (len 1) -> 1 token. Score 1.0 (query "a", 1/1).
    let doc_a = make_doc("a.md", "a");

    // B: "a bbbb" (len 6) -> 2 tokens. Score 0.5 (query "a", 1/2).
    let doc_b = make_doc("b.md", "a bbbb");

    // C: "c" (len 1) -> 1 token. Score 0.0.
    let doc_c = make_doc("c.md", "c");

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);
    let cache = builder.build(vec![doc_a, doc_b, doc_c], &cache_dir).unwrap();

    let selector = ContextSelector::default();
    let query = Query::new("a");

    let result_full = selector.select(&cache, query.clone(), 1000).unwrap();

    assert_eq!(result_full.documents.len(), 3);
    assert_eq!(result_full.documents[0].id, "a.md");
    assert_eq!(result_full.documents[1].id, "b.md");
    assert_eq!(result_full.documents[2].id, "c.md");

    let result_constrained = selector.select(&cache, query, 2).unwrap();

    let ids: Vec<String> = result_constrained
        .documents
        .iter()
        .map(|d| d.id.clone())
        .collect();
    assert_eq!(ids, vec!["a.md", "c.md"], "B should be skipped due to budget constraint");
    assert_eq!(result_constrained.selection.tokens_used, 2);
    assert_eq!(result_constrained.selection.documents_excluded_by_budget, 1);
}

#[test]
fn test_selection_sorting_determinism_on_tie() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().join("cache_tie");

    // Distinct content to avoid hash collision in builder
    // Same score logic: "apple 1" (2 words) vs "apple 2" (2 words). 1 match. 0.5 score.
    let doc_1 = make_doc("z.md", "apple 1");
    let doc_2 = make_doc("a.md", "apple 2");

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);
    let cache = builder.build(vec![doc_1, doc_2], &cache_dir).unwrap();

    let selector = ContextSelector::default();
    let query = Query::new("apple");

    let result = selector.select(&cache, query, 100).unwrap();

    assert_eq!(result.documents[0].id, "a.md");
    assert_eq!(result.documents[1].id, "z.md");
}
