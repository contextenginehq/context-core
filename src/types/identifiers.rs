use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(String);

#[derive(Debug, Error)]
pub enum DocumentIdError {
    #[error("Source path is outside the ingestion root")]
    OutsideRoot,
    #[error("Path involves invalid UTF-8")]
    InvalidUtf8,
}

impl DocumentId {
    /// Create a DocumentId from a source path and a single ingestion root.
    pub fn from_path(root: &Path, source: &Path) -> Result<Self, DocumentIdError> {
        let rel = source
            .strip_prefix(root)
            .map_err(|_| DocumentIdError::OutsideRoot)?;

        let normalized = normalize_path(rel)?;

        Ok(DocumentId(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Normalize path according to spec rules
fn normalize_path(path: &Path) -> Result<String, DocumentIdError> {
    let s = path.to_str().ok_or(DocumentIdError::InvalidUtf8)?;

    let normalized = s
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_lowercase();

    Ok(normalized)
}

/// Content hash version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentVersion(String);

impl DocumentVersion {
    pub fn from_content(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);

        let hash = hasher.finalize();
        let hex = hex::encode(hash);

        DocumentVersion(format!("sha256:{hex}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
