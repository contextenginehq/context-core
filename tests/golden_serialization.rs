use context_core::document::{Document, DocumentId, Metadata};
use context_core::cache::{CacheManifest, CacheBuildConfig, ManifestDocumentEntry};
use serde_json::Value;

// Helper to make a specific document without re-running ingestion logic if possible, 
// strictly for serialization testing.
// Since `Document` fields are public, I can construct it if I have the parts, 
// but `Document` constructor is `ingest`. 
// I'll use `ingest` but verify the result matches the spec JSON.

#[test]
fn golden_document_serialization() {
    let source = "docs/deployment.md";
    let content = "# Deployment\n\nThis guide...";
    
    // We expect normalization: `docs/deployment.md`
    // To survive strip_prefix on ".", the source must start with "."
    let root = std::path::Path::new(".");
    let source_path = std::path::Path::new("./docs/deployment.md");
    let id = DocumentId::from_path(root, source_path).unwrap();

    let mut metadata = Metadata::new();
    metadata.insert_string("title", "Deployment");
    metadata.insert_number("byte_size", 1842);
    metadata.insert_number("line_count", 47);

    // ingest calculates version
    let doc = Document::ingest(
        id, 
        source.to_string(), 
        content.as_bytes().to_vec(),
        metadata
    ).unwrap();

    // Verify version calculation matches spec example if we had the exact content.
    // Spec example version: "sha256:a1b2c3d4e5f6..." is a placeholder.
    // We just want to check the JSON structure and field order.

    let json_str = serde_json::to_string(&doc).unwrap();
    
    // Check key order by looking at the string (brittle but strict for "golden" checks)
    // "id" -> "version" -> "source" -> "content" -> "metadata"
    
    let id_pos = json_str.find("\"id\":").unwrap();
    let ver_pos = json_str.find("\"version\":").unwrap();
    let src_pos = json_str.find("\"source\":").unwrap();
    let cnt_pos = json_str.find("\"content\":").unwrap();
    let meta_pos = json_str.find("\"metadata\":").unwrap();

    assert!(id_pos < ver_pos);
    assert!(ver_pos < src_pos);
    assert!(src_pos < cnt_pos);
    assert!(cnt_pos < meta_pos);
    
    // Valid JSON check
    let _parsed: Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn golden_manifest_serialization() {
    // Manually construct manifest to check field ordering without running FS ops
    
    let config = CacheBuildConfig {
        version: "1".to_string(),
        hash_algorithm: "sha256".to_string(),
    };
    
    // Mock entry
    let id_str = "docs/deployment.md";
    let root = std::path::Path::new(".");
    let id = DocumentId::from_path(root, std::path::Path::new("./docs/deployment.md")).unwrap();
    // manually construct mock version? No, need legit type.
    // Let's use ingest to get version
    let doc = Document::ingest(id.clone(), id_str.to_string(), "content".as_bytes().to_vec(), Metadata::new()).unwrap();
    
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
    
    // Validate key order for manifest
    // cache_version, build_config, created_at, document_count, documents
    
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
