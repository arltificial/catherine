//! Purpose: the composition's schema merge pass — union objects and
//! arrows by declared name and refuse a disagreeing overlap.
//!
//! Responsibilities: the Decision-2 `schema_conflict` check (an arrow
//! declared twice with different endpoints), with both declaring documents
//! and the offending node position.
//!
//! Rationale: the merge key is the declared name, never presentation
//! position, so an agreeing repeat records the second declaring document
//! and nothing is silently overwritten. The instance half of the merge (row
//! sets, cells, remapping) lives in `instance` so each file stays inside the
//! file-size gate.

use super::{CompositionError, Document, SchemaParts};
use crate::model::{Arrow, Ident};
use std::collections::BTreeMap;

/// The schema union with the conflict-checked arrow insertions and the
/// per-arrow declaring documents.
pub(super) fn merge_schema(documents: &[Document]) -> Result<SchemaParts, CompositionError> {
    let mut objects = BTreeMap::new();
    let mut arrows = BTreeMap::new();
    let mut arrow_sources: BTreeMap<Ident, Vec<String>> = BTreeMap::new();
    for doc in documents {
        objects.extend(doc.model.schema.objects.clone());
        for (name, arrow) in &doc.model.schema.arrows {
            insert_arrow(doc, name, arrow, &mut arrows, &mut arrow_sources)?;
        }
    }
    Ok(SchemaParts {
        objects,
        arrows,
        arrow_sources,
    })
}

/// Insert one declared arrow: an agreeing repeat records the second
/// declaring document, a disagreeing one is `schema_conflict`.
fn insert_arrow(
    doc: &Document,
    name: &Ident,
    arrow: &Arrow,
    arrows: &mut BTreeMap<Ident, Arrow>,
    sources: &mut BTreeMap<Ident, Vec<String>>,
) -> Result<(), CompositionError> {
    let Some(existing) = arrows.get(name) else {
        arrows.insert(name.clone(), arrow.clone());
        sources.insert(name.clone(), vec![doc.name.clone()]);
        return Ok(());
    };
    if existing == arrow {
        sources
            .get_mut(name)
            .expect("every inserted arrow has a source entry")
            .push(doc.name.clone());
        return Ok(());
    }
    let mut documents = sources.get(name).cloned().unwrap_or_default();
    documents.push(doc.name.clone());
    Err(CompositionError::new(
        "schema_conflict",
        format!(
            "arrow `{}` is declared with endpoints {} -> {} and {} -> {}",
            name.0, existing.src.0, existing.tgt.0, arrow.src.0, arrow.tgt.0
        ),
        documents,
        doc.arrow_positions.get(&name.0).copied(),
        Some(name.0.clone()),
    ))
}
