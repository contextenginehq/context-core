use context_core::document::{Document, DocumentId, Metadata};
use context_core::cache::{CacheBuilder, CacheBuildConfig};
use std::path::Path;
use std::fs;

fn make_doc(root: &str, source: &str, content: &str) -> Document {
    let root_path = Path::new(root);
    // Handle the "./" prefix logic that helps with normalization tests
    let source_path = if root == "." && !source.starts_with("./") {
        Path::new(source)
    } else {
        Path::new(source)
    };
    
    let id = DocumentId::from_path(root_path, source_path).unwrap();
    Document::ingest(id, source.to_string(), content.as_bytes().to_vec(), Metadata::new()).unwrap()
}

fn get_temp_dir(suffix: &str) -> std::path::PathBuf {
    let mut ignored = std::env::temp_dir();
    ignored.push("context_test");
    ignored.push(suffix);
    // ensure clear start
    if ignored.exists() {
        let _ = fs::remove_dir_all(&ignored);
    }
    let _ = fs::create_dir_all(&ignored);
    ignored
}

#[test]
fn invariant_cache_version_determinism() {
    let doc1 = make_doc(".", "./file1.md", "content1");
    let doc2 = make_doc(".", "./file2.md", "content2");
    
    let docs_order_a = vec![doc1.clone(), doc2.clone()];
    let docs_order_b = vec![doc2.clone(), doc1.clone()];
    
    let config = CacheBuildConfig::v0();
    
    let builder = CacheBuilder::new(config.clone());
    
    let out_a = get_temp_dir("cache_a");
    let out_b = get_temp_dir("cache_b");

    // Clean up if exist (get_temp_dir does it, but to be sure for cache internal check)
    if out_a.exists() { fs::remove_dir_all(&out_a).unwrap(); }
    if out_b.exists() { fs::remove_dir_all(&out_b).unwrap(); }

    let cache_a = builder.build(docs_order_a, &out_a).expect("Build A failed");
    let cache_b = builder.build(docs_order_b, &out_b).expect("Build B failed");
    
    // Cache versions must be identical
    assert_eq!(cache_a.manifest.cache_version, cache_b.manifest.cache_version);
    
    // Document list in manifest must be sorted identically
    assert_eq!(cache_a.manifest.documents.len(), cache_b.manifest.documents.len());
    let doc_a_0 = &cache_a.manifest.documents[0];
    let doc_b_0 = &cache_b.manifest.documents[0];
    assert_eq!(doc_a_0.id, doc_b_0.id);
}

#[test]
fn invariant_config_change_changes_version() {
    let doc1 = make_doc(".", "./file1.md", "content1");
    let docs = vec![doc1];
    
    let mut config1 = CacheBuildConfig::v0();
    config1.version = "1".to_string();
    
    let mut config2 = CacheBuildConfig::v0();
    config2.version = "2".to_string();
    
    let out_1 = get_temp_dir("cache_1");
    let out_2 = get_temp_dir("cache_2");

    if out_1.exists() { fs::remove_dir_all(&out_1).unwrap(); }
    if out_2.exists() { fs::remove_dir_all(&out_2).unwrap(); }
    
    let builder1 = CacheBuilder::new(config1);
    let cache1 = builder1.build(docs.clone(), &out_1).unwrap();
    
    let builder2 = CacheBuilder::new(config2);
    let cache2 = builder2.build(docs, &out_2).unwrap();
    
    assert_ne!(cache1.manifest.cache_version, cache2.manifest.cache_version);
}
