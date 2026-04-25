use serde::Serialize;
use std::collections::BTreeSet;

use crate::scan::{Hit, HitClass};

#[derive(Serialize)]
pub struct ExclusionManifest {
    pub total_documents: usize,
    pub excluded_count: usize,
    pub doc_ids: Vec<usize>,
}

/// Build sorted list of document IDs to exclude from retraining.
///
/// Excludes documents classified as Primary (state media asserting
/// propaganda) or Organic (propaganda keywords asserted without
/// attribution). Keeps Cited (journalism reporting on the narrative)
/// and DomainOnly (state media without specific narrative keywords).
pub fn build_exclusion_list(hits: &[Hit], total_documents: usize) -> ExclusionManifest {
    let mut exclude = BTreeSet::new();

    for hit in hits {
        if let Some(doc_id) = hit.doc_id {
            if matches!(hit.class, HitClass::Primary | HitClass::Organic) {
                exclude.insert(doc_id);
            }
        }
    }

    let doc_ids: Vec<usize> = exclude.into_iter().collect();
    ExclusionManifest {
        total_documents,
        excluded_count: doc_ids.len(),
        doc_ids,
    }
}
