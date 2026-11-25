//! Purpose: the refinement half of canonical relabeling — the first tied
//! class, one refinement round, and the row-signature machinery behind
//! it. Split out of `canonical/label.rs` so both files stay under the
//! size gate; bodies moved verbatim, no behavior change.
//!
//! Responsibilities: tied-class detection, the cross-object refinement
//! fixpoint round, and per-row signatures (previous class, outgoing cell
//! classes, incoming preimage multisets).
//!
//! Rationale: partitions only refine, so the round loop terminates; the
//! signature order (class, outgoing, incoming sizes then elements) is
//! the canonical class identity.

use super::label::RowKey;
use crate::model::{Ident, Model, RowId};
use std::collections::{BTreeMap, BTreeSet};

/// Sort marker larger than any real class rank: a cell with no target
/// (dangling).
const DANGLING: usize = usize::MAX - 1;

/// The first class with more than one row, per object in declared order:
/// the branch point of the exhaustive tie-break.
pub(super) fn first_tied_class(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
) -> Option<(String, Vec<usize>)> {
    for (object, rows) in &model.instance.rows {
        let mut sizes: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for r in 0..rows.len() {
            let class = cls[&(object.0.clone(), r)];
            sizes.entry(class).or_default().push(r);
        }
        for members in sizes.values() {
            if members.len() > 1 {
                return Some((object.0.clone(), members.clone()));
            }
        }
    }
    None
}

/// One refinement round over all objects at once (cross-object cells
/// reference each other's classes). Returns whether any partition
/// changed; partitions only refine, so this terminates.
pub(super) fn refine(model: &Model, cls: &mut BTreeMap<RowKey, usize>) -> bool {
    let mut next = cls.clone();
    for (object, rows) in &model.instance.rows {
        let n = rows.len();
        let sigs: Vec<Vec<usize>> = (0..n)
            .map(|r| row_signature(&SigCtx { model, cls, r }, &object.0))
            .collect();
        apply_ranks(object, &sigs, &mut next);
    }
    let changed = *cls != next;
    *cls = next;
    changed
}

/// Shared lookup context for signature computation: the model, the current
/// class map, and the row being signed.
struct SigCtx<'a> {
    model: &'a Model,
    cls: &'a BTreeMap<RowKey, usize>,
    r: usize,
}

/// One row's refinement signature: its previous class, the classes of its
/// outgoing cells (dangling cells marked [`DANGLING`]), and — per incoming
/// arrow — the size then the sorted multiset of source-row classes.
fn row_signature(ctx: &SigCtx<'_>, object: &str) -> Vec<usize> {
    let sig = vec![ctx.cls[&(object.to_string(), ctx.r)]];
    let sig = push_outgoing(ctx, object, sig);
    push_incoming(ctx, object, sig)
}

/// Outgoing cells, per named arrow in sorted order.
fn push_outgoing(ctx: &SigCtx<'_>, object: &str, mut sig: Vec<usize>) -> Vec<usize> {
    let (model, cls, r) = (ctx.model, ctx.cls, ctx.r);
    for (name, arrow) in &model.schema.arrows {
        if arrow.src.0 != object {
            continue;
        }
        let cell = model
            .instance
            .values
            .get(name)
            .and_then(|v| v.get(r))
            .copied()
            .flatten();
        sig.push(match cell {
            Some(RowId(t)) => cls[&(arrow.tgt.0.clone(), t)],
            None => DANGLING,
        });
    }
    sig
}

/// Incoming preimages, per named arrow in sorted order: the size of the
/// sorted multiset of source-row classes, then its elements.
fn push_incoming(ctx: &SigCtx<'_>, object: &str, mut sig: Vec<usize>) -> Vec<usize> {
    for (name, arrow) in &ctx.model.schema.arrows {
        if arrow.tgt.0 != object {
            continue;
        }
        let preimages = preimage_classes(ctx, name, arrow);
        sig.push(preimages.len());
        sig.extend(preimages);
    }
    sig
}

/// The sorted classes of the rows that `arrow` maps to row `r` of its
/// target.
fn preimage_classes(ctx: &SigCtx<'_>, name: &Ident, arrow: &crate::model::Arrow) -> Vec<usize> {
    let mut preimages: Vec<usize> = ctx
        .model
        .instance
        .values
        .get(name)
        .map(|cells| {
            cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| **cell == Some(RowId(ctx.r)))
                .map(|(s, _)| ctx.cls[&(arrow.src.0.clone(), s)])
                .collect()
        })
        .unwrap_or_default();
    preimages.sort_unstable();
    preimages
}

/// Re-rank one object's rows by signature order and write the new classes.
fn apply_ranks(object: &Ident, sigs: &[Vec<usize>], next: &mut BTreeMap<RowKey, usize>) {
    let ranks = ranks_by_signature(sigs);
    for (r, &rank) in ranks.iter().enumerate() {
        next.insert((object.0.clone(), r), rank);
    }
}

/// Per-row rank: the position of the row's signature among the object's
/// distinct signatures in sorted order — equal signatures share a rank.
fn ranks_by_signature(sigs: &[Vec<usize>]) -> Vec<usize> {
    let distinct: BTreeSet<&[usize]> = sigs.iter().map(|sig| sig.as_slice()).collect();
    sigs.iter()
        .map(|sig| distinct.range(..sig.as_slice()).count())
        .collect()
}
