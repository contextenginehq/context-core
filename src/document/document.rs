use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::identifiers::{DocumentId, DocumentVersion};
use super::metadata::Metadata;

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Content must be valid UTF-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// The atomic unit of content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub version: DocumentVersion,
    pub source: String,
    pub content: String,
    pub metadata: Metadata,
}

impl Document {
    /// Ingest raw bytes into a Document.
    ///
    /// This is the ONLY way to construct a Document.
    /// It enforces all invariants: validation, versioning, and immutability.
    pub fn ingest(
        id: DocumentId,
        source: String,
        raw_content: Vec<u8>,
        metadata: Metadata,
    ) -> Result<Self, DocumentError> {
        let content = String::from_utf8(raw_content)?;

        // Spec: Version computed on verified content
        let version = DocumentVersion::from_content(content.as_bytes());

        Ok(Document {
            id,
            version,
            source,
            content,
            metadata,
        })
    }
}
