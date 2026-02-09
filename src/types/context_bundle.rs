use serde::Serialize;

use crate::document::Document;

/// A fully qualified, normalized query.
/// Normalization rules:
/// - Lowercase
/// - Split on whitespace
/// - Empty terms handled by scorer (score 0.0)
#[derive(Debug, Clone)]
pub struct Query {
    pub raw: String,
    pub terms: Vec<String>,
}

impl Query {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let terms = raw
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Self { raw, terms }
    }
}

/// A selected document returned in the output.
/// Fully self-contained and serializable.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SelectedDocument {
    pub id: String,
    pub version: String,
    /// We own the content here because it's part of the final output payload
    pub content: String,

    pub score: f32,
    pub tokens: usize,

    pub why: SelectionWhy,
}

/// Explanation for why a document received its score.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SelectionWhy {
    pub query_terms: Vec<String>,
    pub term_matches: usize,
    pub total_words: usize,
}

/// Metadata describing the outcome of the selection process.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SelectionMetadata {
    pub query: String,
    pub budget: usize,

    pub tokens_used: usize,

    pub documents_considered: usize,
    pub documents_selected: usize,
    pub documents_excluded_by_budget: usize,
}

/// The final result of a context resolution operation.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SelectionResult {
    pub documents: Vec<SelectedDocument>,
    pub selection: SelectionMetadata,
}

/// Internal: A document that has been scored and tokenized but not yet selected.
/// Holds a reference to the original document to avoid cloning content prematurely.
#[derive(Debug, Clone)]
pub struct ScoredDocument<'a> {
    pub document: &'a Document,

    pub score: f32,
    pub score_details: ScoreDetails,

    pub token_count: usize,
}

/// Internal: Detailed scoring components before serialization.
#[derive(Debug, Clone)]
pub struct ScoreDetails {
    pub query_terms: Vec<String>,
    pub term_matches: usize,
    pub total_words: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    #[error("Invalid budget: {0}")]
    InvalidBudget(usize),

    #[error("Cache integrity error")]
    CacheError,
}
