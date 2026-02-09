// This is intentionally thin:
// no mutation
// no “update” methods
// runtime reads only

use std::path::PathBuf;
use crate::cache::CacheManifest;
use crate::document::Document;
use crate::types::identifiers::DocumentVersion;

#[derive(Debug)]
pub struct ContextCache {
    pub root: PathBuf,
    pub manifest: CacheManifest,
}

impl ContextCache {
    pub fn load_documents(&self) -> Result<Vec<Document>, std::io::Error> {
        let mut loaded_docs = Vec::with_capacity(self.manifest.documents.len());
        for entry in &self.manifest.documents {
            let path = self.root.join(&entry.file);
            let f = std::fs::File::open(&path)?;
            let doc: Document = serde_json::from_reader(f)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            
            // Verify ID matches manifest
            if doc.id != entry.id {
                 return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Document ID mismatch"));
            }

            // Verify version matches manifest (recompute from content)
            let expected_version = DocumentVersion::from_content(doc.content.as_bytes());
            if expected_version != entry.version {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Document version mismatch for {}: manifest says {}, content hashes to {}",
                        entry.id.as_str(),
                        entry.version.as_str(),
                        expected_version.as_str(),
                    ),
                ));
            }
            loaded_docs.push(doc);
        }
        Ok(loaded_docs)
    }
}
