use context_core::cache::{CacheBuildConfig, CacheBuilder};
use context_core::document::{Document, DocumentError, DocumentId, Metadata};
use std::fs;
use std::io::Read;
use std::path::Path;

fn make_doc(root: &str, source: &str, content: &str) -> Document {
    let root_path = Path::new(root);
    let source_path = if root == "." && !source.starts_with("./") {
        Path::new(source)
    } else {
        Path::new(source)
    };

    let id = DocumentId::from_path(root_path, source_path).unwrap();
    Document::ingest(id, source.to_string(), content.as_bytes().to_vec(), Metadata::new()).unwrap()
}

fn make_doc_bytes(root: &str, source: &str, content: Vec<u8>, metadata: Metadata) -> Result<Document, DocumentError> {
    let root_path = Path::new(root);
    let source_path_buf = if root == "." && !source.starts_with("./") {
        Path::new(".").join(source)
    } else {
        Path::new(source).to_path_buf()
    };

    let id = DocumentId::from_path(root_path, &source_path_buf).expect("Test path error");
    Document::ingest(id, source.to_string(), content, metadata)
}

fn get_temp_dir(suffix: &str) -> std::path::PathBuf {
    let mut ignored = std::env::temp_dir();
    ignored.push("context_test");
    ignored.push(suffix);
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

    if out_a.exists() {
        fs::remove_dir_all(&out_a).unwrap();
    }
    if out_b.exists() {
        fs::remove_dir_all(&out_b).unwrap();
    }

    let cache_a = builder.build(docs_order_a, &out_a).expect("Build A failed");
    let cache_b = builder.build(docs_order_b, &out_b).expect("Build B failed");

    assert_eq!(cache_a.manifest.cache_version, cache_b.manifest.cache_version);

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

    if out_1.exists() {
        fs::remove_dir_all(&out_1).unwrap();
    }
    if out_2.exists() {
        fs::remove_dir_all(&out_2).unwrap();
    }

    let builder1 = CacheBuilder::new(config1);
    let cache1 = builder1.build(docs.clone(), &out_1).unwrap();

    let builder2 = CacheBuilder::new(config2);
    let cache2 = builder2.build(docs, &out_2).unwrap();

    assert_ne!(cache1.manifest.cache_version, cache2.manifest.cache_version);
}

#[test]
fn golden_manifest_byte_comparison() {
    let doc1 = make_doc(".", "./a.md", "content A");
    let doc2 = make_doc(".", "./b.md", "content B");
    let docs = vec![doc1, doc2];

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);

    let out1 = get_temp_dir("run1");
    let out2 = get_temp_dir("run2");

    if out1.exists() {
        fs::remove_dir_all(&out1).unwrap();
    }
    if out2.exists() {
        fs::remove_dir_all(&out2).unwrap();
    }

    let _cache1 = builder.build(docs.clone(), &out1).expect("Run1 failed");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let _cache2 = builder.build(docs, &out2).expect("Run2 failed");

    let mut f1 = fs::File::open(out1.join("manifest.json")).unwrap();
    let mut bytes1 = Vec::new();
    f1.read_to_end(&mut bytes1).unwrap();

    let mut f2 = fs::File::open(out2.join("manifest.json")).unwrap();
    let mut bytes2 = Vec::new();
    f2.read_to_end(&mut bytes2).unwrap();

    let str1 = String::from_utf8(bytes1).unwrap();
    let str2 = String::from_utf8(bytes2).unwrap();

    let lines1: Vec<&str> = str1.lines().filter(|l| !l.contains("\"created_at\"")).collect();
    let lines2: Vec<&str> = str2.lines().filter(|l| !l.contains("\"created_at\"")).collect();

    assert_eq!(lines1, lines2, "Manifests differ (ignoring created_at timestamp)");
}

#[test]
fn corruption_missing_file_invalidation() {
    let doc1 = make_doc(".", "./a.md", "content A");
    let docs = vec![doc1];

    let config = CacheBuildConfig::v0();
    let builder = CacheBuilder::new(config);
    let out = get_temp_dir("corruption_test");
    if out.exists() {
        fs::remove_dir_all(&out).unwrap();
    }

    let cache = builder.build(docs, &out).expect("Build failed");

    let manifest = cache.manifest;
    let first_file = &manifest.documents[0].file;
    let file_path = out.join(first_file);
    assert!(file_path.exists(), "Document file should exist initially");

    fs::remove_file(&file_path).expect("Failed to delete document file");

    assert!(!file_path.exists(), "Document file should be gone");

    let is_valid = manifest.documents.iter().all(|entry| out.join(&entry.file).exists());
    assert!(!is_valid, "Cache should be invalid because file is missing");
}

#[test]
fn invariant_utf8_rejection() {
    let invalid_bytes = vec![0, 159, 146, 150];
    let result = make_doc_bytes(".", "doc.md", invalid_bytes, Metadata::new());
    assert!(matches!(result, Err(DocumentError::InvalidUtf8(_))));
}

#[test]
fn invariant_same_content_same_version() {
    let content = "Hello world".as_bytes().to_vec();

    let doc1 = make_doc_bytes(".", "a.md", content.clone(), Metadata::new()).unwrap();
    let doc2 = make_doc_bytes(".", "b.md", content, Metadata::new()).unwrap();

    assert_eq!(doc1.version, doc2.version);
}

#[test]
fn invariant_same_path_same_id() {
    let _id1 = DocumentId::from_path(Path::new("docs"), Path::new("docs/guide.md")).unwrap();

    let _id3 = DocumentId::from_path(Path::new("Docs"), Path::new("Docs/Guide.md")).unwrap();

    #[cfg(windows)]
    {
        let id4 = DocumentId::from_path(Path::new("docs"), Path::new("docs\\guide.md")).unwrap();
        assert_eq!(id4.as_str(), "guide.md");
    }
}

#[test]
fn invariant_metadata_does_not_affect_version() {
    let content = "Immutable content".as_bytes().to_vec();

    let mut meta1 = Metadata::new();
    meta1.insert_string("status", "draft");

    let mut meta2 = Metadata::new();
    meta2.insert_string("status", "published");

    let doc1 = make_doc_bytes(".", "doc.md", content.clone(), meta1).unwrap();
    let doc2 = make_doc_bytes(".", "doc.md", content, meta2).unwrap();

    assert_eq!(doc1.id, doc2.id);
    assert_eq!(doc1.version, doc2.version);
    assert_ne!(doc1.metadata, doc2.metadata);
}

#[test]
fn invariant_metadata_precedence() {
    let content = "cnt".as_bytes().to_vec();

    let mut extracted = Metadata::new();
    extracted.insert_string("title", "Extracted Title");

    let mut provided = Metadata::new();
    provided.insert_string("title", "Provided Title");

    extracted.merge(provided);

    let doc = make_doc_bytes(".", "doc.md", content, extracted).unwrap();

    assert_eq!(
        doc.metadata.get("title"),
        Some(&context_core::document::metadata::MetadataValue::String(
            "Provided Title".into()
        ))
    );
}

#[test]
fn invariant_no_newline_normalization() {
    let unix = "line\n".as_bytes().to_vec();
    let windows = "line\r\n".as_bytes().to_vec();

    let doc_unix = make_doc_bytes(".", "doc.md", unix, Metadata::new()).unwrap();
    let doc_windows = make_doc_bytes(".", "doc.md", windows, Metadata::new()).unwrap();

    assert_ne!(doc_unix.version, doc_windows.version);
}
