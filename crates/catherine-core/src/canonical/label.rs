//! Canonical row re-labeling: structure-derived dense row ids. Split out
//! of `canonical.rs` verbatim (mechanical lint refactor); no behavior
//! change.

use crate::model::{Ident, Model, RowId};
use std::collections::BTreeMap;

use super::serialize;

/// One row's address during canonical labeling: object name + dense id.
/// Object and arrow names are declarations, not presentation — only rows
/// are renamed — so keys built from them are rename-free.
pub(super) type RowKey = (String, usize);

/// The exact refined partition: one sorted `RowKey` block per class.
pub(super) type PartitionKey = Vec<Vec<RowKey>>;

/// Sort markers larger than any real class rank: a cell with no target
/// (dangling) and a row individualized during canonical labeling.
const DANGLING: usize = usize::MAX - 1;
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
    let mut cls: BTreeMap<RowKey, usize> = BTreeMap::new();
    for (object, rows) in &model.instance.rows {
        for r in 0..rows.len() {
            cls.insert((object.0.clone(), r), 0);
        }
    }
    solve(model, &mut cls)
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
fn solve(model: &Model, cls: &mut BTreeMap<RowKey, usize>) -> Model {
    while refine(model, cls) {}
    let Some((object, members)) = first_tied_class(model, cls) else {
        return apply_labeling(model, cls);
    };
    let mut best: Option<(String, Model)> = None;
    let mut explored: Vec<PartitionKey> = Vec::new();
    for r in members {
        let mut seeded = cls.clone();
        seeded.insert((object.clone(), r), INDIVIDUALIZED);
        while refine(model, &mut seeded) {}
        let key = partition_key(&seeded);
        if explored.contains(&key) {
            continue;
        }
        explored.push(key.clone());
        let candidate = solve(model, &mut seeded);
        let bytes = serialize(&candidate);
        if best.as_ref().is_none_or(|(b, _)| bytes < *b) {
            best = Some((bytes, candidate));
        }
    }
    best.expect("the first branch always runs").1
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

/// The first tied class in sorted object order: `(object, members)` with
/// members in ascending row-id order.
fn first_tied_class(model: &Model, cls: &BTreeMap<RowKey, usize>) -> Option<(String, Vec<usize>)> {
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
fn refine(model: &Model, cls: &mut BTreeMap<RowKey, usize>) -> bool {
    let mut next = cls.clone();
    for (object, rows) in &model.instance.rows {
        let n = rows.len();
        let sigs: Vec<Vec<usize>> = (0..n)
            .map(|r| row_signature(model, cls, &object.0, r))
            .collect();
        apply_ranks(object, &sigs, &mut next);
    }
    let changed = *cls != next;
    *cls = next;
    changed
}

/// One row's refinement signature: its previous class, the classes of its
/// outgoing cells (dangling cells marked [`DANGLING`]), and — per incoming
/// arrow — the size then the sorted multiset of source-row classes.
fn row_signature(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
    object: &str,
    r: usize,
) -> Vec<usize> {
    let mut sig = vec![cls[&(object.to_string(), r)]];
    push_outgoing(model, cls, object, r, &mut sig);
    push_incoming(model, cls, object, r, &mut sig);
    sig
}

/// Outgoing cells, per named arrow in sorted order.
fn push_outgoing(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
    object: &str,
    r: usize,
    sig: &mut Vec<usize>,
) {
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
}

/// Incoming preimages, per named arrow in sorted order: the size of the
/// sorted multiset of source-row classes, then its elements.
fn push_incoming(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
    object: &str,
    r: usize,
    sig: &mut Vec<usize>,
) {
    for (name, arrow) in &model.schema.arrows {
        if arrow.tgt.0 != object {
            continue;
        }
        let mut preimages = preimage_classes(model, cls, name, arrow, r);
        preimages.sort_unstable();
        sig.push(preimages.len());
        sig.extend(preimages);
    }
}

/// The classes of the rows that `arrow` maps to row `r` of its target.
fn preimage_classes(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
    name: &Ident,
    arrow: &crate::model::Arrow,
    r: usize,
) -> Vec<usize> {
    let mut preimages: Vec<usize> = Vec::new();
    if let Some(cells) = model.instance.values.get(name) {
        for (s, cell) in cells.iter().enumerate() {
            if *cell == Some(RowId(r)) {
                preimages.push(cls[&(arrow.src.0.clone(), s)]);
            }
        }
    }
    preimages
}

/// Re-rank one object's rows by signature order and write the new classes.
fn apply_ranks(object: &Ident, sigs: &[Vec<usize>], next: &mut BTreeMap<RowKey, usize>) {
    let n = sigs.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|a, b| sigs[*a].cmp(&sigs[*b]));
    let mut ranks = vec![0usize; n];
    let mut rank = 0;
    for i in 1..n {
        if sigs[order[i]] != sigs[order[i - 1]] {
            rank += 1;
        }
        ranks[order[i]] = rank;
    }
    for (r, &rank) in ranks.iter().enumerate() {
        next.insert((object.0.clone(), r), rank);
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
        relabel_cells(model, cls, arrow, cells);
    }
    out
}

/// Remap one arrow's cells through both endpoints' classes.
fn relabel_cells(
    model: &Model,
    cls: &BTreeMap<RowKey, usize>,
    arrow: &Ident,
    cells: &mut Vec<Option<RowId>>,
) {
    let spec = &model.schema.arrows[arrow];
    let (src, tgt) = (spec.src.0.clone(), spec.tgt.0.clone());
    let n = model.instance.rows[&Ident(src.clone())].len();
    let mut relabeled = vec![None; n];
    for (r, cell) in cells.iter().enumerate() {
        let new_r = cls[&(src.clone(), r)];
        relabeled[new_r] = cell.map(|RowId(t)| RowId(cls[&(tgt.clone(), t)]));
    }
    *cells = relabeled;
}
