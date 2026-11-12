//! Purpose: the Δ transport mechanics (beads `catherine-alr` phase B)
//! — the private row-placement and arrow-transport helpers behind
//! `super::reindex`: per-target row placement (one-to-one transfer
//! plus collapsing preimages), lost-cell recording for dropped or
//! non-transportable arrows, target-path transport, and the preimage
//! grouping that drives it all.
//!
//! Rationale: kept apart from the public API file so the reindex
//! contract reads alone; every helper is deterministic (name-ordered
//! walks) so repeated migrations build identical tables.

use super::{MigrationFinding, SchemaMap};
use crate::model::{Ident, Model, Path, RowId, Schema};
use std::collections::BTreeMap;

/// The kept objects' row correspondence, keyed by source object. A
/// single-preimage target transfers rows one-to-one; a multi-preimage
/// target is row-based on its first (sorted) preimage, and each other
/// preimage's row lands on the base row its declared arrow into the base
/// points at. A collapsing row with no pointer (or a dangling one) has
/// no target place and is recorded as lost — never silently dropped and
/// never invented a row to sit on.
pub(super) fn build_row_map(
    model: &Model,
    map: &SchemaMap,
    lost: &mut Vec<MigrationFinding>,
) -> BTreeMap<Ident, BTreeMap<RowId, RowId>> {
    let mut row_map = BTreeMap::new();
    for (target, preimages) in preimages_of(map) {
        let placed = place_group(model, &preimages, lost);
        row_map.insert(target, placed);
    }
    row_map
}

/// One target's placed rows: the first (sorted) preimage transfers
/// one-to-one; each other preimage's row lands on the base row its
/// declared arrow into the base points at. A collapsing row with no
/// pointer (or a dangling one) has no target place and is recorded as
/// lost — never silently dropped and never invented a row to sit on.
fn place_group(
    model: &Model,
    preimages: &[Ident],
    lost: &mut Vec<MigrationFinding>,
) -> BTreeMap<RowId, RowId> {
    let base = &preimages[0];
    let mut placed: BTreeMap<RowId, RowId> = model.instance.rows[base]
        .iter()
        .enumerate()
        .map(|(index, &row)| (row, RowId(index)))
        .collect();
    for other in &preimages[1..] {
        place_collapsing(model, other, base, lost);
        for (index, &row) in model.instance.rows[other].iter().enumerate() {
            if let Some(base_row) = collapse_row(model, other, base, index) {
                placed.insert(row, placed[&base_row]);
            }
        }
    }
    placed
}

/// Where a collapsing preimage's rows land, and which are lost.
fn place_collapsing(model: &Model, other: &Ident, base: &Ident, lost: &mut Vec<MigrationFinding>) {
    for (index, &row) in model.instance.rows[other].iter().enumerate() {
        if collapse_row(model, other, base, index).is_none() {
            lost.push(MigrationFinding::lost_row(other, row));
        }
    }
}

/// The first declared arrow out of `from` into `base`'s object whose
/// cell at `index` holds a base row (walked in name order —
/// deterministic).
fn collapse_row(model: &Model, from: &Ident, base: &Ident, index: usize) -> Option<RowId> {
    model
        .schema
        .arrows
        .iter()
        .find(|(_, decl)| decl.src == *from && decl.tgt == *base)
        .and_then(|(name, _decl)| model.instance.values.get(name))
        .and_then(|table| table.get(index))
        .and_then(|cell| *cell)
        .filter(|v| model.instance.rows[base].contains(v))
}

/// The kept arrows' tables, re-indexed through the mapped target paths,
/// over the target row sets established by `row_map`.
pub(super) fn build_instance(
    model: &Model,
    map: &SchemaMap,
    row_map: &BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    lost: &mut Vec<MigrationFinding>,
) -> crate::model::Instance {
    let mut instance = crate::model::Instance::default();
    for (target, preimages) in preimages_of(map) {
        let count = model.instance.rows[&preimages[0]].len();
        instance
            .rows
            .insert(target.clone(), (0..count).map(RowId).collect());
    }
    let mut tables: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for arrow in map.arrows.keys() {
        reindex_arrow(model, map, row_map, arrow, &mut tables, lost);
    }
    for (name, decl) in &map.target.arrows {
        let width = instance
            .rows
            .get(&decl.src)
            .map(|rows| rows.len())
            .unwrap_or(0);
        let entry = tables.entry(name.clone()).or_default();
        entry.resize(width, None);
    }
    instance.values = tables;
    instance
}

/// Every valued cell of a dropped (or non-transportable) arrow is a
/// lost-cell finding with its row as witness.
pub(super) fn lost_cells_of(model: &Model, arrow: &Ident, lost: &mut Vec<MigrationFinding>) {
    let Some(cells) = model.instance.values.get(arrow) else {
        return;
    };
    for (index, cell) in cells.iter().enumerate() {
        if cell.is_some() {
            lost.push(MigrationFinding::lost_cell(arrow, RowId(index)));
        }
    }
}

/// One kept arrow's table: every valued source cell re-indexed onto the
/// mapped target path. Single-arrow paths write directly; longer paths
/// are a recorded phase-B limit — their valued cells are enumerated as
/// lost (never silently dropped, never invented), and the cells stay
/// `None`. Arrows are visited in `BTreeMap` order, so repeated
/// migrations build identical tables.
fn reindex_arrow(
    model: &Model,
    map: &SchemaMap,
    row_map: &BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    arrow: &Ident,
    tables: &mut BTreeMap<Ident, Vec<Option<RowId>>>,
    lost: &mut Vec<MigrationFinding>,
) {
    let Some(path) = map.arrows.get(arrow).and_then(|p| p.as_ref()) else {
        return;
    };
    let Some(decl) = model.schema.arrows.get(arrow) else {
        return;
    };
    let src_image = map.objects.get(&decl.src).and_then(|a| a.as_ref());
    let tgt_image = map.objects.get(&decl.tgt).and_then(|a| a.as_ref());
    let (Some(src_image), Some(tgt_image)) = (src_image, tgt_image) else {
        // The source object is dropped (its rows are already findings)
        // or the target object is dropped: each valued cell is lost.
        if map.objects.get(&decl.tgt) == Some(&None) {
            lost_cells_of(model, arrow, lost);
        }
        return;
    };
    if path_endpoint(&map.target, path) != Some(tgt_image) {
        lost_cells_of(model, arrow, lost);
        return;
    }
    if path.0.len() > 1 {
        // Phase-B limit, stated where it bites: a multi-arrow target
        // path composes over tables the engine does not build yet, so
        // each valued source cell is recorded as lost and the target
        // cells stay `None` — loud, never invented.
        lost_cells_of(model, arrow, lost);
        return;
    }
    for (index, &row) in model.instance.rows[&decl.src].iter().enumerate() {
        let Some(&target_row) = row_map.get(src_image).and_then(|placed| placed.get(&row)) else {
            continue;
        };
        let value = model.instance.values[arrow]
            .get(index)
            .and_then(|cell| *cell)
            .and_then(|v| {
                row_map
                    .get(tgt_image)
                    .and_then(|placed| placed.get(&v))
                    .copied()
            });
        write_cell(tables, &path.0[0], target_row, value);
    }
}

/// Write one cell into a table, resizing as needed.
fn write_cell(
    tables: &mut BTreeMap<Ident, Vec<Option<RowId>>>,
    arrow: &Ident,
    row: RowId,
    value: Option<RowId>,
) {
    let entry = tables.entry(arrow.clone()).or_default();
    if entry.len() <= row.0 {
        entry.resize(row.0 + 1, None);
    }
    entry[row.0] = value;
}

/// The end object of a target path (`None` when an arrow is undeclared).
fn path_endpoint<'a>(target: &'a Schema, path: &Path) -> Option<&'a Ident> {
    path.0
        .last()
        .and_then(|name| target.arrows.get(name))
        .map(|decl| &decl.tgt)
}

/// Target objects with at least one kept preimage, each with its
/// preimages in sorted (deterministic) order.
fn preimages_of(map: &SchemaMap) -> Vec<(Ident, Vec<Ident>)> {
    let mut grouped: BTreeMap<Ident, Vec<Ident>> = BTreeMap::new();
    for (source, assignment) in &map.objects {
        if let Some(target) = assignment {
            grouped
                .entry(target.clone())
                .or_default()
                .push(source.clone());
        }
    }
    grouped.into_iter().collect()
}

/// The source objects placed into `target`, in sorted (deterministic)
/// order — the preimage group [`place_group`] lays out.
fn preimages_of_target(map: &SchemaMap, target: &Ident) -> Vec<Ident> {
    map.objects
        .iter()
        .filter(|(_, assignment)| assignment.as_ref() == Some(target))
        .map(|(source, _)| source.clone())
        .collect()
}

/// The target row the map places `object`'s row at `index` on — the
/// element-wise forward image of ONE source element, read per object so
/// sibling preimages' coincident row ids never leak across objects.
/// Mirrors [`place_group`] for a single source object: the base preimage
/// transfers by position, a collapsing preimage's row lands on the base
/// row its declared arrow into the base points at. `None` when the map
/// does not keep the object or the row has no place (that row's loss is
/// already enumerated by the reindex).
pub(super) fn placed_row(
    model: &Model,
    map: &SchemaMap,
    object: &Ident,
    index: usize,
) -> Option<RowId> {
    let target = map.objects.get(object)?.as_ref()?;
    let preimages = preimages_of_target(map, target);
    let base = preimages.first()?;
    if object == base {
        return Some(RowId(index));
    }
    let base_row = collapse_row(model, object, base, index)?;
    let position = model.instance.rows[base]
        .iter()
        .position(|row| *row == base_row)?;
    Some(RowId(position))
}
