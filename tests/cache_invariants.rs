use std::fs;
use std::path::Path;

use context_core::cache::{CacheBuildConfig, CacheBuildError, CacheBuilder};
use context_core::document::{Document, DocumentId, Metadata};
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
fn invariant_index_keys_are_sorted() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().join("cache_index_sorted");

    let docs = vec![
        make_doc("b.md", "beta"),
        make_doc("a.md", "alpha"),
        make_doc("c.md", "charlie"),
    ];

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);
    let _cache = builder.build(docs, &cache_dir).unwrap();

    let index_contents = fs::read_to_string(cache_dir.join("index.json")).unwrap();

    let a_pos = index_contents.find("\"a.md\"").expect("missing a.md key");
    let b_pos = index_contents.find("\"b.md\"").expect("missing b.md key");
    let c_pos = index_contents.find("\"c.md\"").expect("missing c.md key");

    assert!(a_pos < b_pos, "index keys must be sorted lexicographically");
    assert!(b_pos < c_pos, "index keys must be sorted lexicographically");
}

#[test]
fn invariant_filename_collision_is_fatal() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().join("cache_collision");

    let doc_a = make_doc("a.md", "same-content");
    let doc_b = make_doc("b.md", "same-content");

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);

    let result = builder.build(vec![doc_a, doc_b], &cache_dir);

    match result {
        Err(CacheBuildError::FilenameCollision(_)) => {}
        other => panic!("expected filename collision error, got {other:?}"),
    }

    assert!(!cache_dir.exists(), "cache output must not be created on failure");
}
