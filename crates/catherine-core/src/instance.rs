//! Purpose: typed instance tables for executed checks — dense per-object
//! row ids and one partial morphism table per arrow, built from the parsed
//! presentation (openspec change `add-executed-checks`).
//!
//! Responsibilities: [`TypedInstance::build`] turns a parsed
//! [`Instance`](crate::model::Instance) into the tables [`checks::check`]
//! evaluates: every parsed arrow value lands somewhere — a morphism value,
//! a dangling record, or a typing violation — never silently dropped. The
//! builder is total and typing-enforcing at build
//! (acset-core `typing_enforced_at_build`): a value outside the arrow's
//! target object's dense id space is a named [`TypingViolation`] naming
//! the row and the arrow, and never becomes a morphism table entry.
//!
//! Rationale: adapted (not authored) from specodelic
//! (`charly/specodelic`, Apache-2.0, ships no NOTICE file)
//! `src/acset/instance.rs` — dense interning with sorted-order
//! assignment, partial morphism storage where an unresolved reference is
//! a representable value (`None`) that is never dropped or repaired, and
//! build-time reports that name their witnesses. Dropped as
//! specodelic-domain: the `graph::build` parity machinery, walk-order
//! edge recording, attribute cells, and markdown `Spec` parsing (P0's
//! parser owns the input). On the kernel's normalized presentation a
//! value *is* a row id in the target object's dense id space, so the
//! typing check is a space check: a value of a wrong object cannot be
//! spelled without leaving the target's space; name-level mistyped
//! values are gated earlier, at parse (structural `Syntax` — see the
//! crate README's error table).

use std::collections::BTreeMap;

use crate::model::{Ident, Instance, Path, RowId, Schema};

/// A build-time typing violation: an arrow value that resolved outside
/// the arrow's target object's dense id space — named with its witnesses
/// (the arrow, the source row it sat on, the offending value), and never
/// stored as a morphism value (acset-core `forbidden_edge_not_stored`,
/// adapted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingViolation {
    /// The arrow whose table the value was rejected from.
    pub arrow: Ident,
    /// The source row the value sat on.
    pub src: RowId,
    /// The rejected value — a row id outside the arrow's target space.
    pub value: RowId,
}

/// A dangling arrow value: the cell holds no row (`None`), stored as the
/// representable value it is — never dropped, repaired, or a hard failure
/// (acset-core `dangling_is_a_value`, adapted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanglingCell {
    /// The arrow whose cell is unresolved.
    pub arrow: Ident,
    /// The source row the dangling value sits on.
    pub src: RowId,
}

/// The typed instance over a parsed presentation: the per-object dense
/// row id spaces, one partial morphism table per declared arrow (parallel
/// to the arrow's source object's rows; `None` = dangling), plus the
/// build-time reports (typing violations, dangling cells).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedInstance {
    /// Dense per-object row id spaces (`0..n`), as parsed.
    rows: BTreeMap<Ident, Vec<RowId>>,
    /// One partial morphism table per declared arrow, keyed by arrow name
    /// in sorted order; `None` = dangling, a representable value.
    tables: BTreeMap<Ident, Vec<Option<RowId>>>,
    /// Typing violations, in build order (arrow name order, then source
    /// row order).
    typing_violations: Vec<TypingViolation>,
    /// Dangling values, in build order (arrow name order, then source row
    /// order).
    dangling: Vec<DanglingCell>,
}

impl TypedInstance {
    /// Build the typed instance for a parsed presentation. Infallible:
    /// every parsed arrow value lands somewhere — a morphism value, a
    /// dangling record, or a typing violation — never a hard failure.
    /// Tables are keyed by arrow name in sorted order, parallel to the
    /// arrow's source object's rows; a value is stored iff it lies inside
    /// the arrow's target object's dense id space. Values keyed by arrows
    /// outside the schema are not part of the presentation and are
    /// ignored.
    pub fn build(schema: &Schema, instance: &Instance) -> Self {
        let rows: BTreeMap<Ident, Vec<RowId>> = instance.rows.clone();
        let mut tables = BTreeMap::new();
        let mut typing_violations = Vec::new();
        let mut dangling = Vec::new();
        for (name, arrow) in &schema.arrows {
            let n_src = rows.get(&arrow.src).map_or(0, Vec::len);
            let n_tgt = rows.get(&arrow.tgt).map_or(0, Vec::len);
            let mut table = vec![None; n_src];
            // The parsed presentation carries one total vector per arrow;
            // cells beyond the source object's row space are outside the
            // presentation's shape (no source row to sit on) and are
            // ignored — a parsed value always has a source row.
            if let Some(cells) = instance.values.get(name) {
                for (idx, &cell) in cells.iter().enumerate().take(n_src) {
                    match cell {
                        None => {
                            dangling.push(DanglingCell {
                                arrow: name.clone(),
                                src: RowId(idx),
                            });
                        }
                        Some(value) if value.0 < n_tgt => table[idx] = Some(value),
                        Some(value) => typing_violations.push(TypingViolation {
                            arrow: name.clone(),
                            src: RowId(idx),
                            value,
                        }),
                    }
                }
            }
            tables.insert(name.clone(), table);
        }
        Self {
            rows,
            tables,
            typing_violations,
            dangling,
        }
    }

    /// The dense per-object row id spaces.
    pub fn rows(&self) -> &BTreeMap<Ident, Vec<RowId>> {
        &self.rows
    }

    /// The per-arrow partial morphism tables, keyed by arrow name in
    /// sorted order.
    pub fn tables(&self) -> &BTreeMap<Ident, Vec<Option<RowId>>> {
        &self.tables
    }

    /// The typing violations — named reports; their values are not
    /// morphism table entries.
    pub fn typing_violations(&self) -> &[TypingViolation] {
        &self.typing_violations
    }

    /// The dangling values — named reports; the cells stay `None` in the
    /// tables.
    pub fn dangling(&self) -> &[DanglingCell] {
        &self.dangling
    }

    /// Compose `path` starting at `start` through the morphism tables:
    /// per arrow, the row's cell — `None` short-circuits, because a path
    /// cannot compose through a dangling value. A cell index outside its
    /// table means the path leaves the presentation's shape; composition
    /// returns `None` rather than guessing (path well-typedness against
    /// the schema is parse's contract, `ill_typed_equation`). Shared by
    /// `check` and available to future consumers of the typed tables.
    pub fn compose(&self, path: &Path, start: RowId) -> Option<RowId> {
        let mut current = start;
        for name in &path.0 {
            let cell = self.tables.get(name)?.get(current.0)?;
            current = (*cell)?;
        }
        Some(current)
    }
}
