use crate::types::context_bundle::{ScoredDocument, SelectedDocument, SelectionWhy};

pub struct BudgetResult {
    pub selected: Vec<SelectedDocument>,
    pub tokens_used: usize,
    pub documents_selected: usize,
    pub documents_excluded_by_budget: usize,
}

pub fn apply_budget(scored_docs: Vec<ScoredDocument>, budget: usize) -> BudgetResult {
    let mut selected = Vec::new();
    let mut tokens_used = 0;
    let mut documents_selected = 0;
    let mut documents_excluded_by_budget = 0;

    for sdoc in scored_docs {
        // Spec: "Documents with score 0.0 MAY be selected if budget allows."
        if tokens_used + sdoc.token_count <= budget {
            selected.push(SelectedDocument {
                id: sdoc.document.id.as_str().to_string(),
                version: sdoc.document.version.as_str().to_string(),
                content: sdoc.document.content.clone(),
                score: sdoc.score,
                tokens: sdoc.token_count,
                why: SelectionWhy {
                    query_terms: sdoc.score_details.query_terms,
                    term_matches: sdoc.score_details.term_matches,
                    total_words: sdoc.score_details.total_words,
                },
            });
            tokens_used += sdoc.token_count;
            documents_selected += 1;
        } else {
            documents_excluded_by_budget += 1;
        }
    }

    BudgetResult {
        selected,
        tokens_used,
        documents_selected,
        documents_excluded_by_budget,
    }
}
