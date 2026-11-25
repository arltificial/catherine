//! Canonical row re-labeling: structure-derived dense row ids. Split out
//! of `canonical.rs` verbatim (mechanical lint refactor); no behavior
//! change.

use crate::model::{Ident, Model, RowId};
use std::collections::BTreeMap;

use super::label_refine::{first_tied_class, refine};
use super::serialize;

/// One row's address during canonical labeling: object name + dense id.
/// Object and arrow names are declarations, not presentation — only rows
/// are renamed — so keys built from them are rename-free.
pub(super) type RowKey = (String, usize);

/// The exact refined partition: one sorted `RowKey` block per class.
pub(super) type PartitionKey = Vec<Vec<RowKey>>;

/// Sort marker larger than any real class rank: a row individualized
/// during canonical labeling.
const INDIVIDUALIZED: usize = usize::MAX;

/// Re-label the model's rows with canonical ids: iterative refinement
/// over the whole structure (class of a row = its previous class, the
/// classes of its outgoing cells, and the sorted classes of its incoming
/// preimages, per named arrow), then — while any class has more than one
/// row — individualize each member of the first tied class and keep the
/// branch whose serialization is lexicographically smallest. Every step
/// consults structure only, so the result is invariant under any
/// renaming of rows; the exhaustive tie-break keeps it exact.
pub(super) fn canonical_relabel(model: &Model) -> Model {
    let cls: BTreeMap<RowKey, usize> = model
        .instance
        .rows
        .iter()
        .flat_map(|(object, rows)| (0..rows.len()).map(move |r| ((object.0.clone(), r), 0usize)))
        .collect();
    solve(model, cls)
}

/// Refine to fixpoint, then individualize tied classes until every row
/// has its own id. Branches whose post-seed refined partition matches
/// an explored branch's exact refined partition are skipped: `solve` is
/// deterministic in the model and the partition, so an equal partition
/// has an identical continuation and cannot change the chosen minimum. (An
/// earlier version pruned on class-size multisets only; equal sizes do
/// not imply isomorphic partitions, and since the surviving branches
/// depend on the input's row order, that tie-break could pick different
/// labelings for renamed presentations. See README "canonical row
/// ids".)
fn solve(model: &Model, mut cls: BTreeMap<RowKey, usize>) -> Model {
    while refine(model, &mut cls) {}
    let Some((object, members)) = first_tied_class(model, &cls) else {
        return apply_labeling(model, &cls);
    };
    let mut best: Option<(String, Model)> = None;
    let mut explored: Vec<PartitionKey> = Vec::new();
    for r in interchangeable_representatives(model, &object, members) {
        let Some(candidate) = explore_branch(model, &cls, &(object.clone(), r), &mut explored)
        else {
            continue;
        };
        let bytes = serialize(&candidate);
        if best.as_ref().is_none_or(|(b, _)| bytes < *b) {
            best = Some((bytes, candidate));
        }
    }
    best.expect("the first branch always runs").1
}

/// One representative's branch: individualize its row, refine to fixpoint,
/// and skip (skipped branches carry no serialization) when an equal
/// refined partition was already explored — `solve` is deterministic in
/// the model and the partition, so an equal partition has an identical
/// continuation and cannot change the chosen minimum. The refined
/// partition ignores the instance map (target names are external data),
/// so two branches with equal partitions can still serialize differently
/// once map cells are relabeled: when a map is present, every branch is
/// explored and the serialization tie-break (which includes the map)
/// picks the canonical labeling, keeping the minimum rename-invariant.
fn explore_branch(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
    seed: &RowKey,
    explored: &mut Vec<PartitionKey>,
) -> Option<Model> {
    let mut seeded = cls.clone();
    seeded.insert(seed.clone(), INDIVIDUALIZED);
    while refine(model, &mut seeded) {}
    let key = partition_key(&seeded);
    if !has_instance_map(model) && explored.contains(&key) {
        return None;
    }
    explored.push(key);
    Some(solve(model, seeded))
}

/// One member per interchangeability group: a member that swaps with an
/// already-kept one branches to the same structure under another name, so
/// exploring it again only multiplies the search. This is what keeps a model
/// with several objects full of indistinguishable rows from hanging
/// (catherine-2f6).
fn interchangeable_representatives(model: &Model, object: &str, members: Vec<usize>) -> Vec<usize> {
    // The scan state is the kept representatives so far: a member that
    // swaps with an already-kept one is skipped (see the caller contract).
    members
        .into_iter()
        .scan(Vec::new(), |kept, r| {
            if kept
                .iter()
                .any(|&candidate| interchangeable(model, object, candidate, r))
            {
                None
            } else {
                kept.push(r);
                Some(r)
            }
        })
        .collect()
}

/// Whether the model declares an instance map whose source rows the canonical
/// labeling must relabel. An absent/empty map keeps the pre-1.4 path byte-identical.
fn has_instance_map(model: &Model) -> bool {
    model
        .instance_map
        .as_ref()
        .is_some_and(|map| !map.maps.is_empty())
}

/// The exact refined partition: one sorted `RowKey` block per class,
/// blocks in class-number order. Two branches with equal keys have the
/// identical refined partition; `solve` is deterministic in the model
/// and the partition (including the class numbers, which
/// `apply_labeling` uses as the new row ids and `first_tied_class`
/// orders by), so an equal key has an identical continuation and cannot
/// change the chosen minimum. Class-size multisets, the earlier key,
/// were NOT sound: equal sizes do not imply equal partitions, and the
/// surviving branch set then depended on the input's row order,
/// breaking rename invariance.
fn partition_key(cls: &BTreeMap<RowKey, usize>) -> PartitionKey {
    let mut blocks: BTreeMap<usize, Vec<RowKey>> = BTreeMap::new();
    for (key, &class) in cls {
        blocks.entry(class).or_default().push(key.clone());
    }
    blocks.into_values().collect()
}

/// Whether swapping rows `a` and `b` of `object` — and nothing else — is an
/// automorphism of the model, instance map included. Such rows are
/// structurally interchangeable: they carry identical incident data, so the
/// two individualization branches differ only in which of the two dense ids
/// each receives and they serialize identically. Exploring one of them is
/// exact, not a heuristic.
fn interchangeable(model: &Model, object: &str, a: usize, b: usize) -> bool {
    if a == b {
        return true;
    }
    if !map_targets_agree(model, object, a, b) {
        return false;
    }
    model.schema.arrows.iter().all(|(name, arrow)| {
        let cells = model.instance.values.get(name).map(Vec::as_slice);
        if arrow.src.0 == object {
            outgoing_commutes(cells, arrow.tgt.0 == object, Swap { a, b })
        } else if arrow.tgt.0 == object {
            incoming_untouched(cells, a, b)
        } else {
            true
        }
    })
}

/// The instance map is external data keyed by source row: rows with different
/// targets are not interchangeable even when their structure matches.
fn map_targets_agree(model: &Model, object: &str, a: usize, b: usize) -> bool {
    let Some(map) = &model.instance_map else {
        return true;
    };
    match map.maps.get(&Ident(object.to_string())) {
        Some(rows) => rows.get(&RowId(a)) == rows.get(&RowId(b)),
        None => true,
    }
}

/// The transposition must commute with the cell function on every row. When
/// the arrow is a self-arrow the target side is swapped too; otherwise the
/// targets are fixed and no cell may name either member.
fn outgoing_commutes(cells: Option<&[Option<RowId>]>, self_targeted: bool, swap: Swap) -> bool {
    let Some(cells) = cells else {
        return true;
    };
    cells.iter().enumerate().all(|(r, cell)| {
        let actual = if self_targeted {
            cells.get(swap.transposed(r)).copied().flatten()
        } else {
            *cell
        };
        swap_row(*cell, swap.a, swap.b) == actual
    })
}

/// The cell with the transposed pair's rows swapped.
fn swap_row(cell: Option<RowId>, a: usize, b: usize) -> Option<RowId> {
    match cell {
        Some(RowId(r)) if r == a => Some(RowId(b)),
        Some(RowId(r)) if r == b => Some(RowId(a)),
        other => other,
    }
}

/// Sources are untouched by the transposition, so no row may point at
/// exactly one of the pair (a functional cell cannot point at both).
fn incoming_untouched(cells: Option<&[Option<RowId>]>, a: usize, b: usize) -> bool {
    cells.is_none_or(|cells| {
        cells
            .iter()
            .all(|cell| (*cell == Some(RowId(a))) == (*cell == Some(RowId(b))))
    })
}

/// The two rows being transposed.
#[derive(Clone, Copy)]
struct Swap {
    a: usize,
    b: usize,
}

impl Swap {
    /// `r` under the transposition.
    fn transposed(&self, r: usize) -> usize {
        if r == self.a {
            self.b
        } else if r == self.b {
            self.a
        } else {
            r
        }
    }
}

/// Apply a fully-refined (all-singleton) labeling: rows become the dense
/// identity vector per object and every value cell is remapped through
/// both endpoints' classes.
fn apply_labeling(model: &Model, cls: &BTreeMap<RowKey, usize>) -> Model {
    let mut out = model.clone();
    for rows in out.instance.rows.values_mut() {
        let n = rows.len();
        *rows = (0..n).map(RowId).collect();
    }
    for (arrow, cells) in out.instance.values.iter_mut() {
        relabel_cells(&Relabel { model, cls }, arrow, cells);
    }
    relabel_map(cls, &mut out);
    out
}

/// Remap the instance map's source ids through the chosen classes;
/// target names are external declarations and stay verbatim.
fn relabel_map(cls: &BTreeMap<RowKey, usize>, out: &mut Model) {
    let Some(out_map) = &mut out.instance_map else {
        return;
    };
    for (object, rows) in &mut out_map.maps {
        let relabeled: BTreeMap<RowId, crate::model::Ident> = rows
            .iter()
            .map(|(row, target)| {
                let new = cls[&(object.0.clone(), row.0)];
                (RowId(new), target.clone())
            })
            .collect();
        *rows = relabeled;
    }
}

/// The relabeling context: the original model and the chosen classes.
struct Relabel<'a> {
    model: &'a Model,
    cls: &'a BTreeMap<RowKey, usize>,
}

/// Remap one arrow's cells through both endpoints' classes.
fn relabel_cells(ctx: &Relabel<'_>, arrow: &Ident, cells: &mut Vec<Option<RowId>>) {
    let spec = &ctx.model.schema.arrows[arrow];
    let (src, tgt) = (spec.src.0.clone(), spec.tgt.0.clone());
    let n = ctx.model.instance.rows[&Ident(src.clone())].len();
    let mut relabeled = vec![None; n];
    for (r, cell) in cells.iter().enumerate() {
        let new_r = ctx.cls[&(src.clone(), r)];
        relabeled[new_r] = cell.map(|RowId(t)| RowId(ctx.cls[&(tgt.clone(), t)]));
    }
    *cells = relabeled;
}
