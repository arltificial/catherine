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
    let objects: BTreeMap<Ident, ()> = documents
        .iter()
        .flat_map(|doc| doc.model.schema.objects.clone())
        .collect();
    let (arrows, arrow_sources) = documents.iter().try_fold(
        (
            BTreeMap::<Ident, Arrow>::new(),
            BTreeMap::<Ident, Vec<String>>::new(),
        ),
        |(mut arrows, mut sources), doc| {
            for (name, arrow) in &doc.model.schema.arrows {
                insert_arrow(
                    &DeclaredArrow { doc, name, arrow },
                    &mut arrows,
                    &mut sources,
                )?;
            }
            Ok((arrows, sources))
        },
    )?;
    Ok(SchemaParts {
        objects,
        arrows,
        arrow_sources,
    })
}

/// One document's declared arrow: the declaring document, its name, and
/// its endpoints.
struct DeclaredArrow<'a> {
    doc: &'a Document,
    name: &'a Ident,
    arrow: &'a Arrow,
}

/// Insert one declared arrow: an agreeing repeat records the second
/// declaring document, a disagreeing one is `schema_conflict`.
fn insert_arrow(
    declared: &DeclaredArrow<'_>,
    arrows: &mut BTreeMap<Ident, Arrow>,
    sources: &mut BTreeMap<Ident, Vec<String>>,
) -> Result<(), CompositionError> {
    let (doc, name, arrow) = (declared.doc, declared.name, declared.arrow);
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
    Err(schema_conflict(declared, (existing, arrow), sources))
}

/// The labeled `schema_conflict`: the arrow's two disagreeing endpoint
/// pairs, both declaring documents, and the offending node position.
fn schema_conflict(
    declared: &DeclaredArrow<'_>,
    endpoints: (&Arrow, &Arrow),
    sources: &BTreeMap<Ident, Vec<String>>,
) -> CompositionError {
    let (existing, arrow) = endpoints;
    let (doc, name) = (declared.doc, declared.name);
    let documents = sources
        .get(name)
        .cloned()
        .map(|mut declared_by| {
            declared_by.push(doc.name.clone());
            declared_by
        })
        .unwrap_or_else(|| vec![doc.name.clone()]);
    CompositionError::new(super::ErrorSpec {
        code: "schema_conflict".to_string(),
        message: format!(
            "arrow `{}` is declared with endpoints {} -> {} and {} -> {}",
            name.0, existing.src.0, existing.tgt.0, arrow.src.0, arrow.tgt.0
        ),
        documents,
        position: doc.arrow_positions.get(&name.0).copied(),
        node: Some(name.0.clone()),
    })
}
