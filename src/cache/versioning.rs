use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::types::identifiers::{DocumentId, DocumentVersion};

// Key point:
// Serializable
// Comparable
// Explicit defaults
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheBuildConfig {
    pub version: String,
    pub hash_algorithm: String,
}

impl CacheBuildConfig {
    pub fn v0() -> Self {
        Self {
            version: "1".into(),
            hash_algorithm: "sha256".into(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManifestDocumentEntry {
    pub id: DocumentId,
    pub version: DocumentVersion,
    pub file: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheManifest {
    pub cache_version: String,
    pub build_config: CacheBuildConfig,
    pub created_at: DateTime<Utc>, // informational only
    pub document_count: usize,
    pub documents: Vec<ManifestDocumentEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CacheIndex {
    entries: BTreeMap<DocumentId, String>,
}

impl CacheIndex {
    pub fn new(entries: BTreeMap<DocumentId, String>) -> Self {
        Self { entries }
    }
}
