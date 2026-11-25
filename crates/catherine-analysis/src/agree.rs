//! Purpose: the cross-instance agreement check (beads epic
//! `catherine-kw9`, ticket 7) — given two instances over schemas sharing
//! a sub-schema, compare rows keyed by shared-entity identity and report
//! every cell-level disagreement as a witness-bearing entry.
//!
//! Responsibilities: match rows of every object the two instances share
//! by presentation name (a name declared on both sides is the same
//! entity), then sweep every schema arrow present on *both* sides — same
//! name, same source, same target — comparing the projected values at
//! each matched row. An equal pair (both sides the same row, or both
//! sides undefined) is silent agreement; a mismatch — including one side
//! dangling — emits one [`Disagreement`] naming the entity's object, the
//! shared key, the arrow, and both projected values in each side's own
//! presentation names. Partial overlap is the point: rows or keys
//! present on only one side are agreement, never completeness failures,
//! and are not findings.
//!
//! Anti-goal: agreement *reports* disagreements; it never auto-merges,
//! rewrites, or mutates either instance.
//!
//! Set-level only — no sheaf/site machinery, no gluing: the check is the
//! finite comparison of declared rows and arrow tables over the shared
//! sub-schema, nothing transfinite.
//!
//! Rationale: mirrors [`crate::naturality`]'s shape discipline —
//! construction over the parsed documents and their `RowNames` side
//! tables, one witnessed entry per breaking cell, sorted output — while
//! staying name-based where naturality is map-based: the shared
//! sub-schema's identity is the presentation name, the very data the
//! parser validates and normalizes away.

use catherine_core::model::{Ident, Model, RowId};
use catherine_core::parse::RowNames;

/// One cell-level disagreement: the arrow of the shared sub-schema
/// projects the shared entity's key to different rows on the two sides.
/// A `None` value is a dangling cell on that side — a disagreement
/// against a defined counterpart, never silently skipped.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Disagreement {
    /// The object whose shared rows disagree (the arrow's source).
    pub entity: Ident,
    /// The presentation key both sides declare for the row.
    pub key: String,
    /// The shared arrow whose projections differ.
    pub arrow: Ident,
    /// Side A's projected value, in A's own presentation names.
    pub value_a: Option<Ident>,
    /// Side B's projected value, in B's own presentation names.
    pub value_b: Option<Ident>,
}

/// Compare the two instances over their shared sub-schema. Objects and
/// arrows are matched by name (an arrow must name the same source and
/// target on both sides to be shared); rows are matched by presentation
/// name. Sorted, one [`Disagreement`] per breaking cell; one-sided rows
/// and keys are silent.
pub fn agree(
    a: &Model,
    b: &Model,
    names_a: &RowNames,
    names_b: &RowNames,
) -> Vec<Disagreement> {
    let mut disagreements = Vec::new();
    for (arrow_name, arrow) in &a.schema.arrows {
        let Some(shared) = b.schema.arrows.get(arrow_name) else {
            continue;
        };
        if shared.src != arrow.src || shared.tgt != arrow.tgt {
            continue;
        }
        for (row_a, row_b, key) in matched_rows(&arrow.src, names_a, names_b) {
            let value_a = cell(a, arrow_name, row_a);
            let value_b = cell(b, arrow_name, row_b);
            if value_a == value_b {
                continue;
            }
            disagreements.push(Disagreement {
                entity: arrow.src.clone(),
                key,
                arrow: arrow_name.clone(),
                value_a: value_name(names_a, &arrow.tgt, value_a),
                value_b: value_name(names_b, &arrow.tgt, value_b),
            });
        }
    }
    disagreements.sort();
    disagreements
}

/// The rows an object declares on both sides, keyed by presentation
/// name: side A's declared order, each name paired with side B's dense
/// row of the same name. Names present on only one side are absent —
/// agreement over the shared sub-schema, never a completeness claim.
fn matched_rows(
    object: &Ident,
    names_a: &RowNames,
    names_b: &RowNames,
) -> Vec<(RowId, RowId, String)> {
    let (Some(names_a), Some(names_b)) = (names_a.get(object), names_b.get(object)) else {
        return Vec::new();
    };
    names_a
        .iter()
        .enumerate()
        .filter_map(|(row_a, name)| {
            names_b
                .iter()
                .position(|other| other == name)
                .map(|row_b| (RowId(row_a), RowId(row_b), name.0.clone()))
        })
        .collect()
}

/// One arrow cell of one instance; `None` for a dangling cell or an
/// out-of-space row.
fn cell(model: &Model, arrow: &Ident, row: RowId) -> Option<RowId> {
    model
        .instance
        .values
        .get(arrow)
        .and_then(|cells| cells.get(row.0))
        .copied()
        .flatten()
}

/// A projected row's presentation name on one side, falling back to the
/// dense id when the side table has no spelled name.
fn value_name(names: &RowNames, object: &Ident, value: Option<RowId>) -> Option<Ident> {
    value.map(|row| {
        names
            .get(object)
            .and_then(|names| names.get(row.0))
            .cloned()
            .unwrap_or_else(|| Ident(format!("#{}", row.0)))
    })
}
