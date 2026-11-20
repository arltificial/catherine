//! The executed check families over interned tables — the parsed walk's
//! five families and the fiber helper, emitting compact int sort keys.

use super::{
    dedup_sorted, engine_compose, insertion_sort_by, CompactFinding, CompactViolation, CompiledKey,
    EngineSchema, EngineTyped, Nid, SortKey,
};
use crate::{findings::EpistemicLabel, model::RowId};

mod pullback;

pub(crate) use pullback::pullback_findings;

pub(crate) use super::push_bounded;

/// One dangling finding per unresolved cell not accounted for by a
/// typing violation — the walk's dangling family over ids.
pub(crate) fn dangling_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
) {
    let mut violated: Vec<(Nid, usize)> = typed
        .typing_violations
        .iter()
        .map(|(nid, idx, _)| (*nid, *idx))
        .collect();
    insertion_sort_by(&mut violated, |a, b| a.cmp(b));
    for (nid, table) in typed.tables.iter().enumerate() {
        for (idx, cell) in table.iter().enumerate() {
            if cell.is_none() && !violated.iter().any(|&(seen, at)| seen == nid && at == idx) {
                push_bounded(
                    out,
                    (
                        (1, 0, 0, nid as u64, idx as u64, 0, 0, 0),
                        EpistemicLabel::Executed,
                        CompactFinding::Dangling {
                            arrow: nid,
                            src: idx,
                        },
                    ),
                );
            }
        }
    }
    let _ = schema;
}

/// Every declared equation on every row of its domain object; a path
/// through an unresolved cell cannot compose, and the affected row is
/// already recorded by its dangling finding.
pub(crate) fn equation_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
) {
    for (index, equation) in schema.equations.iter().enumerate() {
        let Some(domain) = equation.domain.and_then(|d| typed.row_counts[d]) else {
            continue;
        };
        for row in 0..domain {
            let Some(lhs) = engine_compose(typed, &equation.lhs, RowId(row)) else {
                continue;
            };
            let Some(rhs) = engine_compose(typed, &equation.rhs, RowId(row)) else {
                continue;
            };
            if lhs != rhs {
                push_equation_finding(out, index, row, lhs.0, rhs.0, equation.status);
            }
        }
    }
}

/// One equation-disagreement finding for a composed row pair.
fn push_equation_finding(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
    status: crate::model::ConstraintStatus,
) {
    push_bounded(
        out,
        (
            (0, 0, 0, index as u64, row as u64, lhs as u64, rhs as u64, 0),
            crate::findings::label_for(status),
            CompactFinding::Violation(CompactViolation::Equation {
                index,
                row,
                lhs,
                rhs,
            }),
        ),
    );
}

/// Every declared key as an injectivity claim over its object's rows. A
/// row with any unresolved key cell is excluded; one finding per
/// unordered pair of a collision group, ascending rows. The group map of
/// the walk is a stable tuple sort here: equal tuples keep ascending row
/// order, so groups and pairs come out exactly as the `BTreeMap`-based
/// walk emits them.
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

/// The rows of one key grouped by fully resolved key tuple, insertion
/// order — the walk's group map before its tuple sort.
fn key_groups(typed: &EngineTyped, key: &CompiledKey, rows: usize) -> Vec<(Vec<u64>, usize)> {
    let mut grouped: Vec<(Vec<u64>, usize)> = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut tuple: Vec<u64> = Vec::with_capacity(key.arrows.len());
        let mut resolved = true;
        for &arrow in &key.arrows {
            let table = &typed.tables[arrow];
            match table.get(row).copied().flatten() {
                Some(value) => push_bounded(&mut tuple, value.0 as u64),
                None => {
                    resolved = false;
                    break;
                }
            }
        }
        if resolved {
            push_bounded(&mut grouped, (tuple, row));
        }
    }
    insertion_sort_by(
        &mut grouped,
        |a: &(Vec<u64>, usize), b: &(Vec<u64>, usize)| a.0.cmp(&b.0),
    );
    grouped
}

/// One finding per unordered pair of each equal-tuple run, ascending
/// rows within a run and ascending tuples across runs.
fn emit_key_pairs(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    grouped: &[(Vec<u64>, usize)],
    index: usize,
    label: EpistemicLabel,
) {
    let mut start = 0;
    while start < grouped.len() {
        let tuple = grouped[start].0.clone();
        let mut end = start;
        while end < grouped.len() && grouped[end].0 == tuple {
            end += 1;
        }
        for lo in start..end {
            for hi in lo + 1..end {
                let (a, b) = (grouped[lo].1, grouped[hi].1);
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
            }
        }
        start = end;
    }
}

/// The coproduct half of the executed checks — coverage and disjointness
/// verdicts over a declared coproduct's injection images. Images are
/// sorted `Vec`s (same ascending distinct order as the walk's
/// `BTreeSet`, binary-search membership); the complete-image rule is the
/// walk's: any partial image makes coverage undecidable for every row,
/// while disjointness among complete images stays decidable.
pub(crate) fn coproduct_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
) {
    for (index, coproduct) in schema.coproducts.iter().enumerate() {
        let Some(coproduct) = coproduct else { continue };
        let Some(rows) = typed.row_counts[coproduct.object] else {
            continue;
        };
        let label = crate::findings::label_for(coproduct.status);
        let (images, complete) = coproduct_images(typed, &coproduct.arrows);
        coproduct_row_findings(out, &images, complete, index, rows, label);
    }
    let _ = schema;
}

/// The coproduct's complete injection images, ascending distinct target
/// values each — the walk's `BTreeSet` images; `partial` is true when
/// any injection image has an unresolved cell.
fn coproduct_images(typed: &EngineTyped, arrows: &[Nid]) -> (Vec<(Nid, Vec<u64>)>, bool) {
    let mut images: Vec<(Nid, Vec<u64>)> = Vec::with_capacity(arrows.len());
    let mut complete = true;
    for &arrow in arrows {
        let table = &typed.tables[arrow];
        match table
            .iter()
            .map(|cell| cell.as_ref().map(|c| c.0 as u64))
            .collect::<Option<Vec<u64>>>()
        {
            Some(values) => {
                let mut image = values;
                insertion_sort_by(&mut image, |a, b| a.cmp(b));
                dedup_sorted(&mut image);
                push_bounded(&mut images, (arrow, image));
            }
            None => complete = false,
        }
    }
    (images, complete)
}

/// Coverage and disjointness verdicts over one coproduct's rows: with a
/// complete image set, a row must be covered exactly once; with any
/// partial image, coverage is undecidable and only a collision of two or
/// more complete images is decidable.
fn coproduct_row_findings(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    images: &[(Nid, Vec<u64>)],
    complete: bool,
    index: usize,
    rows: usize,
    label: EpistemicLabel,
) {
    for row in 0..rows {
        let mut covering: Vec<Nid> = Vec::with_capacity(images.len());
        for (arrow, image) in images {
            if image.contains(&(row as u64)) {
                push_bounded(&mut covering, *arrow);
            }
        }
        let emit = if complete {
            covering.len() != 1
        } else {
            covering.len() >= 2
        };
        if emit {
            push_bounded(
                out,
                (
                    (0, 3, 0, index as u64, row as u64, 0, 0, 0),
                    label,
                    CompactFinding::Violation(CompactViolation::Coproduct {
                        index,
                        row,
                        injections: covering,
                    }),
                ),
            );
        }
    }
}
