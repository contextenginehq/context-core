use std::path::Path;

use context_core::cache::{CacheBuildConfig, CacheBuilder};
use context_core::document::{Document, DocumentId, Metadata};
use context_core::selection::{ApproxTokenCounter, ContextSelector, TermFrequencyScorer, TokenCounter, Scorer};
use context_core::types::Query;
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
fn invariant_selection_bounded_explainable_complete() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().join("cache_selection_invariants");

    let docs = vec![
        make_doc("alpha.md", "alpha beta alpha"),
        make_doc("beta.md", "beta gamma"),
        make_doc("empty.md", ""),
    ];

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);
    let cache = builder.build(docs, &cache_dir).unwrap();

    let selector = ContextSelector::default();
    let query = Query::new("alpha beta");
    let budget = 8;

    let result = selector.select(&cache, query.clone(), budget).unwrap();

    let tokens_sum: usize = result.documents.iter().map(|doc| doc.tokens).sum();
    assert_eq!(tokens_sum, result.selection.tokens_used, "tokens_used must equal sum of selected tokens");
    assert!(result.selection.tokens_used <= budget, "token usage must never exceed budget");

    let loaded_docs = cache.load_documents().unwrap();
    let scorer = TermFrequencyScorer;
    let tokenizer = ApproxTokenCounter;

    for selected in &result.documents {
        let original = loaded_docs
            .iter()
            .find(|doc| doc.id.as_str() == selected.id)
            .expect("selected document must exist in cache");

        let details = scorer.score(original, &query);
        let score = scorer.score_value(&details);
        let token_count = tokenizer.count_tokens(&original.content);

        assert_eq!(selected.content, original.content, "selected document must include full content");
        assert_eq!(selected.version, original.version.as_str());
        assert_eq!(selected.tokens, token_count);
        assert!((selected.score - score).abs() < f32::EPSILON);
        assert_eq!(selected.why.query_terms, details.query_terms);
        assert_eq!(selected.why.term_matches, details.term_matches);
        assert_eq!(selected.why.total_words, details.total_words);
    }
}
