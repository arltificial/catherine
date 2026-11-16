//! Purpose: the composition's instance merge — union the row space by
//! declared name and refuse the two claims an instance union meets.
//!
//! Responsibilities: the Decision-2 `row_conflict` checks (a shared
//! object's row set; a commonly-declared arrow's cell, with the parser's
//! `null` read as a dangling claim), the sorted merged row space, the
//! per-arrow value vectors over it, and the dense-id remapping through each
//! document's presentation names.
//!
//! Rationale: an instance union is not a union of cells but a meeting of
//! claims, so both are checked rather than merged: documents that declare
//! rows for one object must declare the same set (a partial instance is
//! never expanded into a fabricated dangling cell), and a shared cell must
//! carry one value (`null` against `null` agrees and stays dangling,
//! `null` against a value is a conflict). Row names, not dense ids, are the
//! merge key, so presentation order never decides which cells meet.

use super::{CompositionError, Document, SchemaParts};
use crate::model::{Arrow, Ident, Instance, RowId};
use std::collections::{BTreeMap, BTreeSet};

/// The instance union: row sets must agree, and every shared arrow's cells
/// are remapped into the agreed space.
pub(super) fn merge_instance(
    documents: &[Document],
    parts: &SchemaParts,
) -> Result<Instance, CompositionError> {
    check_row_sets(documents)?;
    let names = merged_row_names(documents, &parts.objects);
    let index = row_index(&names);
    let rows = index
        .iter()
        .map(|(object, ids)| (object.clone(), ids.values().copied().collect()))
        .collect();
    let values = parts
        .arrows
        .iter()
        .map(|(name, arrow)| {
            arrow_cells(documents, name, arrow, &index).map(|cells| (name.clone(), cells))
        })
        .collect::<Result<_, _>>()?;
    Ok(Instance { rows, values })
}

/// One document's declared row set on a shared object.
struct RowClaim {
    document: String,
    rows: Vec<Ident>,
}

/// Refuse a shared object declared with different row sets: the union of
/// two partial instances would fabricate cells no input carries.
fn check_row_sets(documents: &[Document]) -> Result<(), CompositionError> {
    let mut claims: BTreeMap<Ident, RowClaim> = BTreeMap::new();
    for doc in documents {
        for (object, rows) in &doc.rows {
            claim_rows(doc, object, rows, &mut claims)?;
        }
    }
    Ok(())
}

/// Record one document's row claim for one object, refusing a mismatch.
fn claim_rows(
    doc: &Document,
    object: &Ident,
    rows: &[Ident],
    claims: &mut BTreeMap<Ident, RowClaim>,
) -> Result<(), CompositionError> {
    if rows.is_empty() {
        return Ok(());
    }
    let Some(existing) = claims.get(object) else {
        claims.insert(
            object.clone(),
            RowClaim {
                document: doc.name.clone(),
                rows: rows.to_vec(),
            },
        );
        return Ok(());
    };
    if existing.rows == rows {
        return Ok(());
    }
    Err(row_set_conflict(doc, object, existing, rows))
}

/// The labeled row-set `row_conflict`: the object, both declared sets, and
/// the differing row names, attributed to both declaring documents.
fn row_set_conflict(
    doc: &Document,
    object: &Ident,
    existing: &RowClaim,
    rows: &[Ident],
) -> CompositionError {
    let differing = differing_rows(&existing.rows, rows);
    CompositionError::new(
        "row_conflict",
        format!(
            "object `{}` is declared with different row sets [{}] and [{}]; differing rows: [{}]",
            object.0,
            spelled(&existing.rows),
            spelled(rows),
            differing.join(", ")
        ),
        vec![existing.document.clone(), doc.name.clone()],
        None,
        Some(object.0.clone()),
    )
}

/// The row names only one of the two claims carries, sorted.
fn differing_rows(first: &[Ident], second: &[Ident]) -> Vec<String> {
    let first: BTreeSet<&Ident> = first.iter().collect();
    let second: BTreeSet<&Ident> = second.iter().collect();
    first
        .symmetric_difference(&second)
        .map(|name| name.0.clone())
        .collect()
}

/// A row-name list as the conflict message spells it.
fn spelled(rows: &[Ident]) -> String {
    rows.iter()
        .map(|name| name.0.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The per-object union of declared row names, sorted (the merged dense
/// order). Row-set agreement makes the union every claim's own set.
fn merged_row_names(
    documents: &[Document],
    objects: &BTreeMap<Ident, ()>,
) -> BTreeMap<Ident, Vec<Ident>> {
    objects
        .keys()
        .map(|object| {
            let names: BTreeSet<Ident> = documents
                .iter()
                .filter_map(|doc| doc.rows.get(object))
                .flatten()
                .cloned()
                .collect();
            (object.clone(), names.into_iter().collect())
        })
        .collect()
}

/// The merged name → dense id index per object.
fn row_index(names: &BTreeMap<Ident, Vec<Ident>>) -> BTreeMap<Ident, BTreeMap<Ident, RowId>> {
    names
        .iter()
        .map(|(object, rows)| {
            let ids = rows
                .iter()
                .enumerate()
                .map(|(i, name)| (name.clone(), RowId(i)))
                .collect();
            (object.clone(), ids)
        })
        .collect()
}

/// One document's claim about one merged cell: a declared target row, or
/// the parser's `null` dangling value — both are value claims.
#[derive(Clone)]
enum CellClaim {
    Dangling(String),
    Value(String, RowId),
}

impl CellClaim {
    /// The document that made the claim.
    fn document(&self) -> &str {
        match self {
            Self::Dangling(document) | Self::Value(document, _) => document,
        }
    }

    /// Two dangling claims agree; two values agree on one target row; a
    /// dangling claim against a value does not.
    fn agrees_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Dangling(_), Self::Dangling(_)) => true,
            (Self::Value(_, first), Self::Value(_, second)) => first == second,
            _ => false,
        }
    }
}

/// One arrow's merged value vector: every declaring document claims the
/// cells it carries; a shared cell must agree or the run is refused.
fn arrow_cells(
    documents: &[Document],
    name: &Ident,
    arrow: &Arrow,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
) -> Result<Vec<Option<RowId>>, CompositionError> {
    let size = index.get(&arrow.src).map_or(0, BTreeMap::len);
    let mut cells: Vec<Option<CellClaim>> = vec![None; size];
    for doc in documents {
        if doc.model.schema.arrows.contains_key(name) {
            fill_doc_cells(doc, name, arrow, index, &mut cells)?;
        }
    }
    Ok(cells
        .into_iter()
        .map(|cell| cell.and_then(|claim| target_of(&claim)))
        .collect())
}

/// The target row of a value claim (`None` for a dangling claim).
fn target_of(claim: &CellClaim) -> Option<RowId> {
    match claim {
        CellClaim::Value(_, target) => Some(*target),
        CellClaim::Dangling(_) => None,
    }
}

/// Fill one document's claims for one arrow into the merged vector.
fn fill_doc_cells(
    doc: &Document,
    name: &Ident,
    arrow: &Arrow,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    cells: &mut [Option<CellClaim>],
) -> Result<(), CompositionError> {
    let Some(values) = doc.model.instance.values.get(name) else {
        return Ok(());
    };
    for (i, target) in values.iter().enumerate() {
        set_cell(doc, name, arrow, index, (RowId(i), *target), cells)?;
    }
    Ok(())
}

/// Set one remapped cell, refusing a disagreeing shared claim.
fn set_cell(
    doc: &Document,
    name: &Ident,
    arrow: &Arrow,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    cell: (RowId, Option<RowId>),
    cells: &mut [Option<CellClaim>],
) -> Result<(), CompositionError> {
    let Some(src) = remap_row(doc, &arrow.src, index, cell.0) else {
        return Ok(());
    };
    let Some(claim) = cell_claim(doc, arrow, index, cell.1) else {
        return Ok(());
    };
    match &cells[src.0] {
        None => {
            cells[src.0] = Some(claim);
            Ok(())
        }
        Some(existing) => set_or_conflict(doc, name, arrow, index, src, existing, claim),
    }
}

/// One document's claim about one cell: an unmappable target row (only
/// possible for a document that claims no rows for the target object)
/// yields no claim.
fn cell_claim(
    doc: &Document,
    arrow: &Arrow,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    target: Option<RowId>,
) -> Option<CellClaim> {
    let Some(target) = target else {
        return Some(CellClaim::Dangling(doc.name.clone()));
    };
    remap_row(doc, &arrow.tgt, index, target).map(|tgt| CellClaim::Value(doc.name.clone(), tgt))
}

/// Accept an agreeing claim or refuse the disagreeing pair.
fn set_or_conflict(
    doc: &Document,
    name: &Ident,
    arrow: &Arrow,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    src: RowId,
    existing: &CellClaim,
    claim: CellClaim,
) -> Result<(), CompositionError> {
    if existing.agrees_with(&claim) {
        return Ok(());
    }
    Err(row_conflict(doc, name, arrow, index, src, existing, &claim))
}

/// One dense id's merged id, through the document's presentation name.
fn remap_row(
    doc: &Document,
    object: &Ident,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    row: RowId,
) -> Option<RowId> {
    let name = doc.rows.get(object)?.get(row.0)?;
    index.get(object)?.get(name).copied()
}

/// The labeled cell `row_conflict`: names the arrow, the row, both claims
/// (a dangling claim spelled `<dangling>`), and both declaring documents.
fn row_conflict(
    doc: &Document,
    name: &Ident,
    arrow: &Arrow,
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    src: RowId,
    existing: &CellClaim,
    claim: &CellClaim,
) -> CompositionError {
    let row = name_of(index, &arrow.src, src);
    let first = spoken(index, &arrow.tgt, existing);
    let second = spoken(index, &arrow.tgt, claim);
    CompositionError::new(
        "row_conflict",
        format!(
            "arrow `{}` values row `{row}` as `{first}` and `{second}`",
            name.0
        ),
        vec![existing.document().to_string(), doc.name.clone()],
        doc.cell_positions
            .get(&(name.0.clone(), row.clone()))
            .copied(),
        Some(row),
    )
}

/// A claim as the conflict message spells it: the target row's presentation
/// name, or `<dangling>` for the parser's `null`.
fn spoken(
    index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>,
    target: &Ident,
    claim: &CellClaim,
) -> String {
    match claim {
        CellClaim::Dangling(_) => "<dangling>".to_string(),
        CellClaim::Value(_, row) => name_of(index, target, *row),
    }
}

/// A merged row's presentation name (the bare id when the lookup misses).
fn name_of(index: &BTreeMap<Ident, BTreeMap<Ident, RowId>>, object: &Ident, row: RowId) -> String {
    index
        .get(object)
        .and_then(|ids| ids.iter().find(|(_, id)| **id == row))
        .map(|(name, _)| name.0.clone())
        .unwrap_or_else(|| row.0.to_string())
}
