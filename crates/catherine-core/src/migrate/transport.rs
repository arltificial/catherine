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
use crate::model::{Arrow, Ident, Model, Path, RowId, Schema};
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

/// One reindex's read side: the model being migrated, its schema map,
/// and the kept objects' row correspondence.
struct Ctx<'a> {
    model: &'a Model,
    map: &'a SchemaMap,
    row_map: &'a BTreeMap<Ident, BTreeMap<RowId, RowId>>,
}

/// One reindex's write side: the tables being built and the lost-data
/// report being appended to.
struct Sink<'a> {
    tables: &'a mut BTreeMap<Ident, Vec<Option<RowId>>>,
    lost: &'a mut Vec<MigrationFinding>,
}

/// The mapped endpoints of one transportable arrow: the single-arrow
/// path's arrow name and the source/target object images.
struct Leg<'a> {
    path_arrow: &'a Ident,
    src_image: &'a Ident,
    tgt_image: &'a Ident,
}

impl Sink<'_> {
    /// Write one cell into a table, resizing as needed.
    fn write(&mut self, arrow: &Ident, row: RowId, value: Option<RowId>) {
        let entry = self.tables.entry(arrow.clone()).or_default();
        if entry.len() <= row.0 {
            entry.resize(row.0 + 1, None);
        }
        entry[row.0] = value;
    }
}

/// The target rows of the kept targets: dense per-target row spaces
/// sized by the first (sorted) preimage.
fn init_target_rows(map: &SchemaMap, model: &Model) -> BTreeMap<Ident, Vec<RowId>> {
    preimages_of(map)
        .into_iter()
        .map(|(target, preimages)| {
            let count = model.instance.rows[&preimages[0]].len();
            (target, (0..count).map(RowId).collect())
        })
        .collect()
}

/// Widen every declared target arrow's table to its domain's width; the
/// untransported cells stay `None` (never invented).
fn pad_tables(
    tables: &mut BTreeMap<Ident, Vec<Option<RowId>>>,
    instance: &crate::model::Instance,
    map: &SchemaMap,
) {
    for (name, decl) in &map.target.arrows {
        let width = instance
            .rows
            .get(&decl.src)
            .map(|rows| rows.len())
            .unwrap_or(0);
        tables.entry(name.clone()).or_default().resize(width, None);
    }
}

/// One reindex's correspondence and lost-data report: the kept rows'
/// target placement and the findings being appended to.
pub(super) struct Report<'a> {
    pub(super) row_map: &'a BTreeMap<Ident, BTreeMap<RowId, RowId>>,
    pub(super) lost: &'a mut Vec<MigrationFinding>,
}

/// One element's forward placement: the model and map whose object
/// correspondence and declared arrows place source rows on targets.
pub(super) struct Placement<'a> {
    pub(super) model: &'a Model,
    pub(super) map: &'a SchemaMap,
}

/// The kept arrows' tables, re-indexed through the mapped target paths,
/// over the target row sets established by the report's `row_map`.
pub(super) fn build_instance(
    model: &Model,
    map: &SchemaMap,
    report: Report<'_>,
) -> crate::model::Instance {
    let mut instance = crate::model::Instance {
        rows: init_target_rows(map, model),
        ..crate::model::Instance::default()
    };
    let ctx = Ctx {
        model,
        map,
        row_map: report.row_map,
    };
    let mut tables: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    let mut sink = Sink {
        tables: &mut tables,
        lost: report.lost,
    };
    for arrow in map.arrows.keys() {
        reindex_arrow(&ctx, arrow, &mut sink);
    }
    pad_tables(&mut tables, &instance, map);
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
fn reindex_arrow(ctx: &Ctx<'_>, arrow: &Ident, sink: &mut Sink<'_>) {
    let Some(path) = ctx.map.arrows.get(arrow).and_then(|p| p.as_ref()) else {
        return;
    };
    let Some(decl) = ctx.model.schema.arrows.get(arrow) else {
        return;
    };
    let Some((src_image, tgt_image)) = source_and_target_images(ctx.map, decl) else {
        // The source object is dropped (its rows are already findings)
        // or the target object is dropped: each valued cell is lost.
        dropped_target_cells(ctx, decl, arrow, sink.lost);
        return;
    };
    transport_or_record(
        ctx,
        sink,
        &Route {
            arrow,
            path,
            src_image,
            tgt_image,
        },
    );
}

/// One kept arrow's forwarding data: which arrow, the mapped path it
/// takes, and the source/target objects its endpoints map to.
struct Route<'a> {
    arrow: &'a Ident,
    path: &'a Path,
    src_image: &'a Ident,
    tgt_image: &'a Ident,
}

/// Single-arrow paths ending at the mapped target object transport their
/// valued cells; everything else is a recorded phase-B limit: each valued
/// source cell is enumerated as lost (loud) and the target cells stay
/// `None` — never invented.
fn transport_or_record(ctx: &Ctx<'_>, sink: &mut Sink<'_>, route: &Route<'_>) {
    if path_endpoint(&ctx.map.target, route.path) == Some(route.tgt_image)
        && route.path.0.len() == 1
    {
        transport_rows(
            ctx,
            sink,
            route.arrow,
            &Leg {
                path_arrow: &route.path.0[0],
                src_image: route.src_image,
                tgt_image: route.tgt_image,
            },
        );
        return;
    }
    lost_cells_of(ctx.model, route.arrow, sink.lost);
}

/// A dropped endpoint's valued cells: only a dropped *target* object has
/// cells to record (a dropped source's rows carry their own findings).
fn dropped_target_cells(
    ctx: &Ctx<'_>,
    decl: &Arrow,
    arrow: &Ident,
    lost: &mut Vec<MigrationFinding>,
) {
    if ctx.map.objects.get(&decl.tgt) == Some(&None) {
        lost_cells_of(ctx.model, arrow, lost);
    }
}

/// One transportable arrow's valued source cells, each re-indexed onto
/// the mapped target path and written where the source row was kept.
fn transport_rows(ctx: &Ctx<'_>, sink: &mut Sink<'_>, arrow: &Ident, leg: &Leg<'_>) {
    let decl = &ctx.model.schema.arrows[arrow];
    for (index, &row) in ctx.model.instance.rows[&decl.src].iter().enumerate() {
        let Some(&target_row) = ctx
            .row_map
            .get(leg.src_image)
            .and_then(|placed| placed.get(&row))
        else {
            continue;
        };
        let value = cell_value(ctx, arrow, index, leg.tgt_image);
        sink.write(leg.path_arrow, target_row, value);
    }
}

/// The mapped source/target object images of one declared arrow, when
/// both sides are kept.
fn source_and_target_images<'a>(
    map: &'a SchemaMap,
    decl: &crate::model::Arrow,
) -> Option<(&'a Ident, &'a Ident)> {
    let src = map.objects.get(&decl.src).and_then(|a| a.as_ref())?;
    let tgt = map.objects.get(&decl.tgt).and_then(|a| a.as_ref())?;
    Some((src, tgt))
}

/// One source cell's transported value: its declared target row mapped
/// through the target object's row correspondence.
fn cell_value(ctx: &Ctx<'_>, arrow: &Ident, index: usize, tgt_image: &Ident) -> Option<RowId> {
    ctx.model.instance.values[arrow]
        .get(index)
        .and_then(|cell| *cell)
        .and_then(|v| {
            ctx.row_map
                .get(tgt_image)
                .and_then(|placed| placed.get(&v))
                .copied()
        })
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

impl Placement<'_> {
    /// The target row the map places `object`'s row at `index` on — the
    /// element-wise forward image of ONE source element, read per object so
    /// sibling preimages' coincident row ids never leak across objects.
    /// Mirrors [`place_group`] for a single source object: the base preimage
    /// transfers by position, a collapsing preimage's row lands on the base
    /// row its declared arrow into the base points at. `None` when the map
    /// does not keep the object or the row has no place (that row's loss is
    /// already enumerated by the reindex).
    pub(super) fn of(&self, object: &Ident, index: usize) -> Option<RowId> {
        let target = self.map.objects.get(object)?.as_ref()?;
        let preimages = preimages_of_target(self.map, target);
        let base = preimages.first()?;
        if object == base {
            return Some(RowId(index));
        }
        let base_row = collapse_row(self.model, object, base, index)?;
        let position = self.model.instance.rows[base]
            .iter()
            .position(|row| *row == base_row)?;
        Some(RowId(position))
    }
}
