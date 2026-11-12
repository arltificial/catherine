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
//! inverse and, when rows collapse, proves non-invertibility by naming
//! the offending source row no candidate inverse recovers.
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
use transport::{build_instance, build_row_map, lost_cells_of};

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
/// element (the collapsed source row) rather than failing at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonInvertibilityProof {
    /// The source object that loses rows under Δ∘Δ′.
    pub object: Ident,
    /// The offending row: not recoverable by the candidate inverse.
    pub row: RowId,
    /// The proof text, naming the sibling row it collapses onto.
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
/// middle instance along `inverse`. When a source object maps through
/// the forward map and the candidate inverse back onto itself with fewer
/// rows than it started with, the smallest non-recovered row is the
/// offending element and the proof names the sibling row it collapses
/// onto. `None` = no collapse observed.
pub fn round_trip(
    source: &Model,
    forward: &SchemaMap,
    inverse: &SchemaMap,
) -> Option<NonInvertibilityProof> {
    let middle = reindex(source)?;
    let mut middle_model = Model::empty();
    middle_model.schema = forward.target.clone();
    middle_model.instance = middle.instance.clone();
    middle_model.schema_map = Some(inverse.clone());
    let back = reindex(&middle_model)?;
    collapse_proof(source, &middle.row_map, &back.row_map, inverse)
}

/// The proof scan: for each source object that the candidate inverse
/// maps back onto itself with FEWER rows than it started with, the
/// smallest non-recovered row is the offending element and the proof
/// names the sibling row it collapses onto.
fn collapse_proof(
    source: &Model,
    middle_row_map: &BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    back_row_map: &BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    inverse: &SchemaMap,
) -> Option<NonInvertibilityProof> {
    for (object, images) in middle_row_map {
        if inverse.objects.get(object) != Some(&Some((*object).clone())) {
            continue;
        }
        let recovered: Vec<RowId> = back_row_map
            .get(object)
            .into_iter()
            .flat_map(|placed| placed.values().copied())
            .collect();
        let source_rows = &source.instance.rows[object];
        if recovered.len() >= source_rows.len() {
            continue;
        }
        let offending = source_rows
            .iter()
            .copied()
            .find(|row| !recovered.contains(row))?;
        let target = images[&offending];
        let sibling = *images
            .iter()
            .find(|(row, place)| **row != offending && **place == target)
            .map(|(row, _)| row)?;
        return Some(NonInvertibilityProof {
            object: (*object).clone(),
            row: offending,
            detail: format!(
                "rows {} and {} of {} collapse onto one target row; \
                 {} is not recoverable under the candidate inverse",
                sibling.0, offending.0, object.0, offending.0
            ),
        });
    }
    None
}
