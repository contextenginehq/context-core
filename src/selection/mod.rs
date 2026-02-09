pub mod filters;
pub mod ranking;
pub mod budgeting;

use std::cmp::Ordering;

use crate::cache::ContextCache;
use crate::types::context_bundle::{
	Query, ScoredDocument, SelectionError, SelectionMetadata, SelectionResult,
};
pub use ranking::{ApproxTokenCounter, Scorer, TermFrequencyScorer, TokenCounter};
pub use budgeting::{apply_budget, BudgetResult};

pub struct ContextSelector<S, T> {
	scorer: S,
	tokenizer: T,
}

impl Default for ContextSelector<TermFrequencyScorer, ApproxTokenCounter> {
	fn default() -> Self {
		Self {
			scorer: TermFrequencyScorer,
			tokenizer: ApproxTokenCounter,
		}
	}
}

impl<S, T> ContextSelector<S, T>
where
	S: Scorer,
	T: TokenCounter,
{
	pub fn new(scorer: S, tokenizer: T) -> Self {
		Self { scorer, tokenizer }
	}

	pub fn select(
		&self,
		cache: &ContextCache,
		query: Query,
		budget: usize,
	) -> Result<SelectionResult, SelectionError> {
		// 0. Load documents strictly from manifest to ensure authoritativeness
		let loaded_docs = cache.load_documents().map_err(|_| SelectionError::CacheError)?;

		// 1. Scoring Phase
		let mut scored_docs: Vec<ScoredDocument> = loaded_docs
			.iter()
			.map(|doc| {
				let details = self.scorer.score(doc, &query);
				let score = self.scorer.score_value(&details);
				let token_count = self.tokenizer.count_tokens(&doc.content);
				ScoredDocument {
					document: doc,
					score,
					score_details: details,
					token_count,
				}
			})
			.collect();

		// 2. Ordering Phase
		// Sort globally by (score desc, id asc)
		scored_docs.sort_by(|a, b| {
			// Descending score
			let score_cmp = b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal);
			if score_cmp != Ordering::Equal {
				score_cmp
			} else {
				// Ascending ID
				a.document.id.cmp(&b.document.id)
			}
		});

		debug_assert!(
			scored_docs.windows(2).all(|w| {
				let a = &w[0];
				let b = &w[1];
				a.score > b.score || (a.score == b.score && a.document.id <= b.document.id)
			})
		);

		// 3. Budgeting Phase
		let BudgetResult {
			selected,
			tokens_used,
			documents_selected,
			documents_excluded_by_budget,
		} = apply_budget(scored_docs, budget);

		let metadata = SelectionMetadata {
			query: query.raw,
			budget,
			tokens_used,
			documents_considered: loaded_docs.len(),
			documents_selected,
			documents_excluded_by_budget,
		};

		Ok(SelectionResult {
			documents: selected,
			selection: metadata,
		})
	}
}
