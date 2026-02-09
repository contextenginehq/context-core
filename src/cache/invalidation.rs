use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::cache::cache::ContextCache;
use crate::cache::versioning::{CacheBuildConfig, CacheIndex, CacheManifest, ManifestDocumentEntry};
use crate::document::Document;

#[derive(Debug, Error)]
pub enum CacheBuildError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Output directory already exists: {0}")]
    OutputExists(PathBuf),
    #[error("Filename collision detected for hash fragment: {0}")]
    FilenameCollision(String),
    #[error("Duplicate document ID: {0}")]
    DuplicateDocumentId(String),
    #[error("Invalid version format: {0}")]
    InvalidVersionFormat(String),
}

/// CacheBuilder is single-threaded and non-reentrant by design.
pub struct CacheBuilder {
    config: CacheBuildConfig,
}

impl CacheBuilder {
    pub fn new(config: CacheBuildConfig) -> Self {
        Self { config }
    }

    pub fn build(
        &self,
        documents: Vec<Document>,
        output_dir: &Path,
    ) -> Result<ContextCache, CacheBuildError> {
        if output_dir.exists() {
            return Err(CacheBuildError::OutputExists(output_dir.to_path_buf()));
        }

        // 1. Sort documents by ID to ensure determinism
        let mut sorted_docs = documents;
        sorted_docs.sort_by(|a, b| a.id.cmp(&b.id));

        // 1b. Check for duplicate document IDs (adjacent after sort)
        for pair in sorted_docs.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(CacheBuildError::DuplicateDocumentId(
                    pair[0].id.as_str().to_string(),
                ));
            }
        }

        // 2. Prepare structures and check for collisions
        // We store pairs of (Document, ManifestEntry) to guarantee alignment explicitly
        let mut doc_contexts = Vec::with_capacity(sorted_docs.len());
        let mut index_entries = BTreeMap::new();
        let mut seen_filenames = BTreeSet::new();

        // Used for cache version computation
        // "sorted(document_id + ":" + document_version)"
        let mut version_hasher = Sha256::new();

        // Hash the config
        let config_json = serde_json::to_vec(&self.config)?;
        version_hasher.update(&config_json);

        for doc in &sorted_docs {
            // Update cache version hash
            let line = format!("{}:{}", doc.id.as_str(), doc.version.as_str());
            version_hasher.update(line.as_bytes());

            // Determine filename: first 12 chars of version hash (without prefix)
            let full_hash = doc
                .version
                .as_str()
                .strip_prefix("sha256:")
                .ok_or_else(|| CacheBuildError::InvalidVersionFormat(doc.version.as_str().to_string()))?;

            if full_hash.len() < 12 {
                // Should not happen for sha256, but safe handling
                return Err(CacheBuildError::FilenameCollision(full_hash.to_string()));
            }
            let filename_stem = &full_hash[..12];
            let filename = format!("{}.json", filename_stem);

            // Check collision
            if seen_filenames.contains(filename_stem) {
                return Err(CacheBuildError::FilenameCollision(filename_stem.to_string()));
            }
            seen_filenames.insert(filename_stem.to_string());

            // Add to entries
            let relative_path = format!("documents/{}", filename);

            let entry = ManifestDocumentEntry {
                id: doc.id.clone(),
                version: doc.version.clone(),
                file: relative_path.clone(),
            };

            index_entries.insert(doc.id.clone(), relative_path);
            doc_contexts.push((doc, entry));
        }

        let hash_bytes = version_hasher.finalize();
        let cache_version = format!("sha256:{}", hex::encode(hash_bytes));

        // 3. Create Manifest
        // Collect manifest documents from our aligned context
        let mut manifest_documents: Vec<ManifestDocumentEntry> = doc_contexts
            .iter()
            .map(|(_, entry)| entry.clone())
            .collect();

        // Explicitly sort again just to be absolutely safe against refactors
        manifest_documents.sort_by(|a, b| a.id.cmp(&b.id));

        // Note: created_at is strictly informational
        let manifest = CacheManifest {
            cache_version: cache_version.clone(),
            build_config: self.config.clone(),
            created_at: Utc::now(),
            document_count: sorted_docs.len(),
            documents: manifest_documents,
        };

        let index = CacheIndex::new(index_entries);

        // 4. Write to temp dir
        // Use a deterministic-but-unique temp dir
        // We use the first 12 chars of the new cache version to avoid collisions
        // between different builds targeting the same parent dir (unlikely but safer)
        let temp_suffix = format!("tmp.{}", &cache_version[7..19]);
        let temp_dir = output_dir.with_extension(temp_suffix);

        // Clean up any stale temp dir from a crashed previous run of THIS specific version
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;
        fs::create_dir(temp_dir.join("documents"))?;

        // Write documents
        // doc_contexts guarantees alignment
        for (doc, entry) in doc_contexts {
            let path = temp_dir.join(&entry.file); // entry.file is "documents/..."
            let f = fs::File::create(path)?;
            serde_json::to_writer(&f, doc)?;
            f.sync_all()?;
        }

        // Write index.json
        let index_path = temp_dir.join("index.json");
        let f_idx = fs::File::create(index_path)?;
        // BTreeMap ensures lexicographical sort of keys
        serde_json::to_writer_pretty(&f_idx, &index)?;
        f_idx.sync_all()?;

        // Write manifest.json
        let manifest_path = temp_dir.join("manifest.json");
        let f_man = fs::File::create(manifest_path)?;
        serde_json::to_writer_pretty(&f_man, &manifest)?;
        f_man.sync_all()?;

        // 5. Atomic Rename
        fs::rename(&temp_dir, output_dir)?;

        Ok(ContextCache {
            root: output_dir.to_path_buf(),
            manifest,
        })
    }
}
