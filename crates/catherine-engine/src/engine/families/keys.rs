//! The key family of the executed checks — injectivity of every
//! declared key over its object's rows, over the arena.

use super::super::{
    Bounded, CompactFinding, CompactViolation, CompiledKey, EngineSchema, EngineTyped, SortKey,
    ARROWS_CAP, ROWS_CAP,
};
use super::push_bounded;
use crate::findings::EpistemicLabel;
use std::cmp::Ordering;

pub(crate) fn key_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
) {
    for (index, key) in schema.keys.iter().enumerate() {
        let Some(key) = key else { continue };
        let Some(rows) = typed.row_counts[key.object] else {
            continue;
        };
        let label = crate::findings::label_for(key.status);
        let grouped = key_groups(typed, key, rows);
        emit_key_pairs(out, &grouped, index, label);
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
    let mut i = 0;
    while i < a.len() && i < b.len() {
        if a[i] < b[i] {
            return Ordering::Less;
        }
        if a[i] > b[i] {
            return Ordering::Greater;
        }
        i += 1;
    }
    a.len().cmp(&b.len())
}

/// Tuple equality — the run scan's comparison, slot by slot.
fn tuple_eq(a: &KeyTuple, b: &KeyTuple) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The rows of one key grouped by fully resolved key tuple, insertion
/// order — the walk's group map before its tuple sort. Arena-backed:
/// every temporary is a fixed-cap table, no allocation model.
fn key_groups(typed: &EngineTyped, key: &CompiledKey, rows: usize) -> KeyGroups {
    let mut grouped = KeyGroups::new();
    let mut row = 0;
    while row < rows {
        let mut tuple = KeyTuple::new();
        let mut resolved = true;
        let mut a = 0;
        while a < key.arrows.len() {
            let table = &typed.tables[key.arrows[a]];
            match table.get(row).copied().flatten() {
                Some(value) => tuple.push(value.0 as u64),
                None => {
                    resolved = false;
                    break;
                }
            }
            a += 1;
        }
        if resolved {
            grouped.push((tuple, row));
        }
        row += 1;
    }
    grouped.sort_pushed_by(|a: &(KeyTuple, usize), b: &(KeyTuple, usize)| tuple_cmp(&a.0, &b.0));
    grouped
}

/// One finding per unordered pair of each equal-tuple run, ascending
/// rows within a run and ascending tuples across runs.
fn emit_key_pairs(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    grouped: &KeyGroups,
    index: usize,
    label: EpistemicLabel,
) {
    let mut start = 0;
    while start < grouped.len() {
        let end = key_run_end(grouped, start);
        emit_run_pairs(out, grouped, start, end, index, label);
        start = end;
    }
}

/// One equal-tuple run's exclusive end — the walk's run scan, ascending
/// rows within.
fn key_run_end(grouped: &KeyGroups, start: usize) -> usize {
    let Some((start_tuple, _)) = grouped[start].as_ref() else {
        return start;
    };
    let mut end = start;
    while end < grouped.len() {
        let Some((tuple, _)) = grouped[end].as_ref() else {
            break;
        };
        if !tuple_eq(tuple, start_tuple) {
            break;
        }
        end += 1;
    }
    end
}

/// One finding per unordered pair within one equal-tuple run, ascending
/// rows.
fn emit_run_pairs(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    grouped: &KeyGroups,
    start: usize,
    end: usize,
    index: usize,
    label: EpistemicLabel,
) {
    let mut lo = start;
    while lo < end {
        let mut hi = lo + 1;
        while hi < end {
            let (a, b) = (
                grouped[lo].as_ref().map(|g| g.1).unwrap_or(0),
                grouped[hi].as_ref().map(|g| g.1).unwrap_or(0),
            );
            push_bounded(
                out,
                (
                    (0, 1, 0, index as u64, a as u64, b as u64, 0, 0),
                    label,
                    CompactFinding::Violation(CompactViolation::Key {
                        index,
                        lo: a,
                        hi: b,
                    }),
                ),
            );
            hi += 1;
        }
        lo += 1;
    }
}
