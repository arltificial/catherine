//! Purpose: the Δ migration engine (openspec change `add-delta-migration`
//! phase B, beads `catherine-alr`) — the pullback reindex along a total
//! schema map, the loud lost-data enumeration, and the round-trip
//! Δ∘Δ′ non-invertibility analysis.
//!
//! Responsibilities: [`reindex`] reshapes a parsed model's instance into
//! its map's target schema, inventing nothing (every target row and cell
//! value traces to a source row or cell through the returned
//! [`MigrationOutcome::row_map`]); [`lost_data`] enumerates, per the
//! kata-10 pattern, one finding per dropped row (dropped object) or per
//! valued dropped cell (dropped arrow on a surviving object) — findings
//! labeled [`EpistemicLabel::Executed`], never a parse error and never a
//! silent drop; [`round_trip`] composes a forward map with a candidate
//! inverse and, when an element's return differs, proves
//! non-invertibility by naming the offending `(object, row)` element no
//! candidate inverse returns to itself.
//!
//! Rationale: Spivak functorial migration, Σ/Π deliberately out of scope
//! until Δ is validated (proposal C3/E4). Row semantics: a target object
//! with several preimage objects is row-based on its first preimage
//! (sorted); every other preimage's row lands on the base row its
//! declared arrow into the base points at — the `collapse` fixture's
//! shape — and a row with no such pointer has nowhere to land and is
//! lost, loudly. Arrow cells re-index through the mapped target path,
//! arrow by arrow; a kept arrow whose cells cannot re-index (dropped
//! target object, non-composable path endpoint) loses each valued cell
//! as a finding. Determinism: every enumeration walks `BTreeMap` order,
//! so repeated migrations report identically. Lost data and the proof
//! live in this module (the taxonomy's P2 kinds are parse-time shapes;
//! refactor task 9 dedupes finding construction) but reuse the
//! taxonomy's label type — lost data is `executed`, the kernel walk's
//! weight, never a fabricated epistemic layer.

use crate::findings::EpistemicLabel;
use crate::model::{Ident, Instance, Model, RowId, SchemaMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod transport;
use transport::{build_instance, build_row_map, lost_cells_of, placed_row};

/// Why a finding was raised by the migration engine — the kind names the
/// shape, the label (`executed`) names the weight, mirroring the P2
/// taxonomy's contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "witness", rename_all = "snake_case")]
pub enum MigrationFindingKind {
    /// A whole row is dropped: the object was assigned `null`, or the
    /// row has no place in the target under a collapsing map.
    LostRow {
        /// The source object owning the row.
        object: Ident,
        /// The dropped row.
        row: RowId,
    },
    /// A valued arrow cell is dropped: the arrow was assigned `null`,
    /// or its cells cannot re-index into the target.
    LostCell {
        /// The dropped (or unplaceable) arrow.
        arrow: Ident,
        /// The source row whose cell held a value.
        row: RowId,
    },
}

/// A migration finding: the taxonomy shape (label + kind-with-witness),
/// constructed by this module only.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MigrationFinding {
    /// How the finding was established — always `executed` here: the
    /// engine computed the loss itself.
    pub label: EpistemicLabel,
    /// The finding kind with its mandatory witness.
    #[serde(flatten)]
    pub kind: MigrationFindingKind,
}

impl MigrationFinding {
    fn lost_row(object: &Ident, row: RowId) -> Self {
        Self {
            label: EpistemicLabel::Executed,
            kind: MigrationFindingKind::LostRow {
                object: object.clone(),
                row,
            },
        }
    }

    fn lost_cell(arrow: &Ident, row: RowId) -> Self {
        Self {
            label: EpistemicLabel::Executed,
            kind: MigrationFindingKind::LostCell {
                arrow: arrow.clone(),
                row,
            },
        }
    }
}

/// Why the candidate inverse fails — the proof names the offending
/// source element, an `(object, row)` pair, rather than failing at
/// runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonInvertibilityProof {
    /// The source object whose element does not return to itself.
    pub object: Ident,
    /// The offending row: not recoverable by the candidate inverse.
    pub row: RowId,
    /// The proof text, naming the forward image and the sibling row
    /// sharing it.
    pub detail: String,
}

/// The pullback reindex result: the target instance, the row
/// correspondence (kept source rows → target rows), and the lost-data
/// enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationOutcome {
    /// The migrated instance over the map's target schema.
    pub instance: Instance,
    /// Per kept source object: each of its rows' target row. Dropped
    /// objects are absent — their loss is in `lost`.
    pub row_map: BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    /// The loud lost-data enumeration, in deterministic order.
    pub lost: Vec<MigrationFinding>,
}

/// Rows of objects assigned `null`: each is a lost-row finding with its
/// witness.
fn null_object_losses(map: &SchemaMap, model: &Model, lost: &mut Vec<MigrationFinding>) {
    for (object, assignment) in &map.objects {
        if assignment.is_none() {
            for row in &model.instance.rows[object] {
                lost.push(MigrationFinding::lost_row(object, *row));
            }
        }
    }
}

/// Valued cells of arrows assigned `null`: each is a lost-cell finding
/// with its row as witness.
fn null_arrow_losses(map: &SchemaMap, model: &Model, lost: &mut Vec<MigrationFinding>) {
    for (arrow, assignment) in &map.arrows {
        if assignment.is_none() {
            lost_cells_of(model, arrow, lost);
        }
    }
}

/// Objects and arrows assigned `null` are dropped explicitly — every row
/// of a dropped object and every valued cell of a dropped arrow is a
/// lost-data finding with its witness (kata-10 pattern: loud, row by
/// row, never silent).
fn enumerate_null_losses(map: &SchemaMap, model: &Model, lost: &mut Vec<MigrationFinding>) {
    null_object_losses(map, model, lost);
    null_arrow_losses(map, model, lost);
}

/// The Δ reindex: reshape the source instance along the map. Returns
/// `None` when the document carries no schema map — a migration without
/// a declared map is not a migration. The lost-data report includes the
/// null-assigned objects' rows and the dropped arrows' valued cells.
pub fn reindex(model: &Model) -> Option<MigrationOutcome> {
    let map = model.schema_map.as_ref()?;
    let mut lost = Vec::new();
    enumerate_null_losses(map, model, &mut lost);
    let row_map = build_row_map(model, map, &mut lost);
    let instance = build_instance(model, map, &row_map, &mut lost);
    lost.sort();
    Some(MigrationOutcome {
        instance,
        row_map,
        lost,
    })
}

/// The lost-data enumeration only (the `lost` field of [`reindex`]).
pub fn lost_data(model: &Model) -> Option<Vec<MigrationFinding>> {
    reindex(model).map(|outcome| outcome.lost)
}

/// The Δ∘Δ′ analysis: reindex the source along `forward`, then the
/// middle instance along `inverse`, and compare element-wise returns.
/// An analyzed element is a source `(object, row)` pair the forward map
/// keeps (a row Δ itself loses is already lost data, not a round-trip
/// element); its return is the source element the candidate inverse
/// yields for the pair's forward image. The smallest element whose
/// return is missing or differs — another object or another row — is
/// the offending element, and the proof names it with the sibling row
/// sharing its forward image. `None` = every kept element returns to
/// itself.
pub fn round_trip(
    source: &Model,
    forward: &SchemaMap,
    inverse: &SchemaMap,
) -> Option<NonInvertibilityProof> {
    let middle = reindex(source)?;
    let mut middle_model = Model::empty();
    middle_model.schema = forward.target.clone();
    middle_model.instance = middle.instance;
    middle_model.schema_map = Some(inverse.clone());
    let back = reindex(&middle_model)?;
    collapse_proof(source, forward, inverse, &back.row_map)
}

/// One forward-image group's proof context: the source object being
/// analyzed, its rows, and the middle object its rows are placed on.
struct Group<'a> {
    source: &'a Model,
    forward: &'a SchemaMap,
    object: &'a Ident,
    rows: &'a [RowId],
    target: &'a Ident,
}

/// The proof scan: every source element the forward map keeps is carried
/// to its forward image and back through the candidate inverse; the
/// return is compared as an `(object, row)` pair, never as a bare row
/// id, so per-object dense row spaces cannot be confused. Objects are
/// walked in name order and each object's rows by position, so the
/// smallest offender is deterministic.
fn collapse_proof(
    source: &Model,
    forward: &SchemaMap,
    inverse: &SchemaMap,
    back_row_map: &BTreeMap<Ident, BTreeMap<RowId, RowId>>,
) -> Option<NonInvertibilityProof> {
    for (object, rows) in &source.instance.rows {
        if let Some(proof) = object_proof(source, forward, inverse, back_row_map, object, rows) {
            return Some(proof);
        }
    }
    None
}

/// One source object's element scan: the first row whose return is
/// missing or differs is that group's offender. A missing return place,
/// a return to another object, and a return to another row are all
/// non-identities.
fn object_proof(
    source: &Model,
    forward: &SchemaMap,
    inverse: &SchemaMap,
    back_row_map: &BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    object: &Ident,
    rows: &[RowId],
) -> Option<NonInvertibilityProof> {
    let target = forward.objects.get(object).and_then(|a| a.as_ref())?;
    let group = Group {
        source,
        forward,
        object,
        rows,
        target,
    };
    for (index, &row) in rows.iter().enumerate() {
        let Some(image) = placed_row(source, forward, object, index) else {
            continue;
        };
        let returned = element_return(inverse, back_row_map, target, image);
        if returned != Some((object, row)) {
            return Some(offending(&group, row, image, returned));
        }
    }
    None
}

/// The element the candidate inverse returns for a middle element: the
/// object it maps `target` onto and the back row it places `image` at.
/// `None` = no return place (the inverse does not map `target` back, or
/// the middle row has no place in the returned object).
fn element_return<'a>(
    inverse: &'a SchemaMap,
    back_row_map: &'a BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    target: &Ident,
    image: RowId,
) -> Option<(&'a Ident, RowId)> {
    let back = inverse.objects.get(target)?.as_ref()?;
    let row = *back_row_map.get(back)?.get(&image)?;
    Some((back, row))
}

/// The proof for one offending element, with the sibling source row
/// sharing its forward image (when the group has one) named in the
/// detail.
fn offending(
    group: &Group<'_>,
    row: RowId,
    image: RowId,
    returned: Option<(&Ident, RowId)>,
) -> NonInvertibilityProof {
    let sibling = group
        .rows
        .iter()
        .enumerate()
        .find(|(index, other)| {
            **other != row
                && placed_row(group.source, group.forward, group.object, *index) == Some(image)
        })
        .map(|(_, other)| *other);
    NonInvertibilityProof {
        object: group.object.clone(),
        row,
        detail: proof_detail(group.object, row, image, group.target, sibling, returned),
    }
}

/// The proof text: the element's forward image (and the sibling sharing
/// it), what the candidate inverse actually returns, and the
/// non-recovery verdict.
fn proof_detail(
    object: &Ident,
    row: RowId,
    image: RowId,
    target: &Ident,
    sibling: Option<RowId>,
    returned: Option<(&Ident, RowId)>,
) -> String {
    let pair = sibling.map(|sibling| {
        if sibling.0 <= row.0 {
            (sibling, row)
        } else {
            (row, sibling)
        }
    });
    let element = match pair {
        Some((low, high)) => format!(
            "rows {} and {} of {} share the forward image {} row {}",
            low.0, high.0, object.0, target.0, image.0
        ),
        None => format!(
            "row {} of {} has the forward image {} row {}",
            row.0, object.0, target.0, image.0
        ),
    };
    let outcome = match returned {
        Some((back, back_row)) => format!(
            "the candidate inverse returns {} row {}",
            back.0, back_row.0
        ),
        None => "the candidate inverse returns no place".to_string(),
    };
    format!(
        "{element}; {outcome}, so row {} is not recoverable under the candidate inverse",
        row.0
    )
}
