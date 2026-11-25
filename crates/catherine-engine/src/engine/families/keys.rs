//! The key family of the executed checks — injectivity of every
//! declared key over its object's rows, over the arena.

use super::super::{
    Bounded, CompactFinding, CompactViolation, CompiledKey, EngineSchema, EngineTyped, ARROWS_CAP,
    ROWS_CAP,
};
use super::FindingSink;
use crate::findings::EpistemicLabel;
use std::cmp::Ordering;

pub(crate) fn key_findings(schema: &EngineSchema, typed: &EngineTyped, out: &mut impl FindingSink) {
    for (index, key) in schema.keys.iter().enumerate() {
        let Some(key) = key else { continue };
        let Some(rows) = typed.row_counts[key.object] else {
            continue;
        };
        let label = crate::findings::label_for(key.status);
        let grouped = key_groups(typed, key, rows);
        emit_key_pairs(out, &grouped, KeyEmission { index, label });
    }
    let _ = schema;
}

/// One row's key tuple over the arena — the walk's resolved key cells,
/// ascending arrow order.
type KeyTuple = Bounded<u64, ARROWS_CAP>;

/// The rows of one key grouped by fully resolved key tuple, insertion
/// order — the walk's group map before its tuple sort, over the arena.
type KeyGroups = Bounded<(KeyTuple, usize), ROWS_CAP>;

/// Lexicographic tuple order — the walk's `Vec<u64>` comparison, slot by
/// slot (the tuple lengths are equal within one key's group; the length
/// arm keeps the general `Vec` order for shorter prefixes).
fn tuple_cmp(a: &KeyTuple, b: &KeyTuple) -> Ordering {
    for i in 0..a.len().min(b.len()) {
        if a[i] < b[i] {
            return Ordering::Less;
        }
        if a[i] > b[i] {
            return Ordering::Greater;
        }
    }
    a.len().cmp(&b.len())
}

/// Tuple equality — the run scan's comparison, slot by slot.
fn tuple_eq(a: &KeyTuple, b: &KeyTuple) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}

/// One row's fully resolved key tuple, ascending arrow order — `None`
/// when any of the key's cells is unresolved (the walk's skip).
fn key_tuple(typed: &EngineTyped, key: &CompiledKey, row: usize) -> Option<KeyTuple> {
    let mut tuple = KeyTuple::new();
    for arrow in &key.arrows {
        let table = &typed.tables[*arrow];
        let value = table.get(row).copied().flatten()?;
        tuple.push(value.0 as u64);
    }
    Some(tuple)
}

/// The rows of one key grouped by fully resolved key tuple, insertion
/// order — the walk's group map before its tuple sort. Arena-backed:
/// every temporary is a fixed-cap table, no allocation model.
fn key_groups(typed: &EngineTyped, key: &CompiledKey, rows: usize) -> KeyGroups {
    let mut grouped = KeyGroups::new();
    for row in 0..rows {
        if let Some(tuple) = key_tuple(typed, key, row) {
            grouped.push((tuple, row));
        }
    }
    grouped.sort_pushed_by(|a: &(KeyTuple, usize), b: &(KeyTuple, usize)| tuple_cmp(&a.0, &b.0));
    grouped
}

/// One key declaration's emission context: the declaration index and
/// its constraint label — the pair every collision finding carries.
#[derive(Clone, Copy)]
struct KeyEmission {
    index: usize,
    label: EpistemicLabel,
}

/// One finding per unordered pair of each equal-tuple run, ascending
/// rows within a run and ascending tuples across runs.
///
/// Flat concrete-index slot scan (the dangling exclusion's pattern,
/// ledger `0006`): the table is sorted, so equal tuples are adjacent
/// and every equal pair `(i, j)`, `i < j`, is exactly the run's
/// unordered pair set — the old run scan (`key_run_end` + symbolic
/// `end` bounds) emitted the same pairs in the same order while
/// charging symex symbolic-bound loops and per-slot `Option` forks
/// (measured 2026-11-24: both key harnesses budget-killed in symex at
/// the standard, at unwind 8 and 16 alike; the flat scan bounds every
/// loop concretely by the cap).
fn emit_key_pairs(out: &mut impl FindingSink, grouped: &KeyGroups, ctx: KeyEmission) {
    for i in 0..ROWS_CAP {
        let a = key_slot(grouped, i);
        for j in i + 1..ROWS_CAP {
            emit_key_collision(out, &a, &key_slot(grouped, j), &ctx);
        }
    }
}

/// One grouped entry by concrete slot index — `None` past the length or
/// on an empty slot (the flat scan's guard, no symbolic bounds).
fn key_slot(grouped: &KeyGroups, i: usize) -> Option<(&KeyTuple, usize)> {
    let entry = grouped.get(i)?.as_ref()?;
    Some((&entry.0, entry.1))
}

/// One collision finding when two ascending slots carry equal tuples —
/// the run scan's unordered pair, once per pair.
fn emit_key_collision(
    out: &mut impl FindingSink,
    a: &Option<(&KeyTuple, usize)>,
    b: &Option<(&KeyTuple, usize)>,
    ctx: &KeyEmission,
) {
    if let (Some((ta, ra)), Some((tb, rb))) = (a, b) {
        if tuple_eq(ta, tb) {
            out.emit((
                (0, 1, 0, ctx.index as u64, *ra as u64, *rb as u64, 0, 0),
                ctx.label,
                CompactFinding::Violation(CompactViolation::Key {
                    index: ctx.index,
                    lo: *ra,
                    hi: *rb,
                }),
            ));
        }
    }
}
