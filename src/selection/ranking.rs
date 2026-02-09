use crate::document::Document;
use crate::types::context_bundle::{Query, ScoreDetails};

pub trait Scorer {
    fn score(&self, doc: &Document, query: &Query) -> ScoreDetails;

    fn score_value(&self, details: &ScoreDetails) -> f32 {
        let score = if details.total_words == 0 {
            0.0
        } else {
            details.term_matches as f32 / details.total_words as f32
        };
        debug_assert!((0.0..=1.0).contains(&score), "score {score} out of range [0.0, 1.0]");
        score
    }
}

/// v0: Simple Term Frequency Scorer
#[derive(Default)]
pub struct TermFrequencyScorer;

impl Scorer for TermFrequencyScorer {
    fn score(&self, doc: &Document, query: &Query) -> ScoreDetails {
        // Spec: total_words is defined as split(content, whitespace).len() after lowercasing.
        let content_lower = doc.content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();
        let total_words = words.len();

        let term_matches = if total_words == 0 || query.terms.is_empty() {
            0
        } else {
            let mut count = 0;
            // Naive count: occurrences of ANY query term
            for word in &words {
                for term in &query.terms {
                    if word == term {
                        count += 1;
                    }
                }
            }
            count
        };

        ScoreDetails {
            query_terms: query.terms.clone(),
            term_matches,
            total_words,
        }
    }
}

pub trait TokenCounter {
    fn count_tokens(&self, content: &str) -> usize;
}

/// v0: Approximate GPT-style tokenization
/// tokens(content) := ceil(len(content) / 4)
#[derive(Default)]
pub struct ApproxTokenCounter;

impl TokenCounter for ApproxTokenCounter {
    fn count_tokens(&self, content: &str) -> usize {
        // Integer division ceil(len / 4) equivalent to (len + 4 - 1) / 4
        if content.is_empty() {
            0
        } else {
            (content.len() + 3) / 4
        }
    }
}
