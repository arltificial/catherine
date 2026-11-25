//! Purpose: the additive 1.4 `instance_map` section — a claimed, total
//! row mapping from this document's instance to another instance of the
//! same schema (openspec change `add-instance-morphism`, design
//! Decisions 1 and 2).
//!
//! Responsibilities: deserialize the raw section, validate each entry's
//! object and source rows against the declared schema and instance,
//! reject an uncovered row with `incomplete_instance_map` at the
//! per-object map node, and normalize the result to a [`InstanceMap`]
//! keyed by dense [`RowId`] in canonical object/row order. Target names
//! are preserved verbatim: they name rows of *another* document, so
//! resolution belongs to the operation's construction, never to parse.
//!
//! Rationale: a partial map has no decidable reading without an implicit
//! identity default, and an implicit default is exactly the
//! silent-anything failure mode the negative-input matrix exists to
//! prevent. The map is sealed model evidence, so its source side is
//! normalized structurally while its target side stays declared data.

use super::schema::{check_ident, is_ident};
use super::spans::Spans;
use crate::error::{ErrorCode, ParseError};
use crate::model::{Ident, InstanceMap, RowId};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// The raw additive section: `{"maps": [{"object": O, "rows": {...}}]}`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawInstanceMap {
    #[serde(default)]
    pub(super) maps: Vec<RawInstanceMapEntry>,
}

/// One declared per-object map. `rows` is keyed by the source document's
/// presentation row names; each value names a row of the target instance.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawInstanceMapEntry {
    pub(super) object: String,
    #[serde(default)]
    pub(super) rows: BTreeMap<String, String>,
}

/// One entry's row checks: the entry index (for node pointers), the
/// entry's object, the object's declared rows, the span index, and the
/// mapped rows accumulated so far.
struct EntryCheck<'a> {
    index: usize,
    object: &'a Ident,
    declared: Option<&'a BTreeMap<String, RowId>>,
    spans: &'a Spans,
    mapped: BTreeMap<RowId, Ident>,
}

/// What the instance-map section is checked against: the declared
/// objects, the declared rows per object, and the span index.
pub(super) struct MapCtx<'a> {
    pub(super) objects: &'a BTreeMap<Ident, ()>,
    pub(super) row_index: &'a BTreeMap<String, BTreeMap<String, RowId>>,
    pub(super) spans: &'a Spans,
}

/// The instance-map accumulator: the shared check context, the objects
/// already claimed by an entry, and the collected per-object row maps.
struct MapCheck<'a> {
    ctx: &'a MapCtx<'a>,
    seen: BTreeSet<Ident>,
    maps: BTreeMap<Ident, BTreeMap<RowId, Ident>>,
}

/// Validate and normalize the declared instance map. `None` for an
/// absent or empty section (hash-neutral).
pub(super) fn build_instance_map(
    raw: &RawInstanceMap,
    ctx: MapCtx,
) -> Result<Option<InstanceMap>, ParseError> {
    let mut check = MapCheck {
        ctx: &ctx,
        seen: BTreeSet::new(),
        maps: BTreeMap::new(),
    };
    for (index, entry) in raw.maps.iter().enumerate() {
        check.check_entry(entry, index)?;
    }
    reject_uncovered_objects(&ctx, &check.seen)?;
    if check.maps.is_empty() {
        return Ok(None);
    }
    Ok(Some(InstanceMap { maps: check.maps }))
}

impl MapCheck<'_> {
    /// One entry: its object (declared, not a repeated entry), then its
    /// rows; non-empty row maps are collected in object order.
    fn check_entry(&mut self, entry: &RawInstanceMapEntry, index: usize) -> Result<(), ParseError> {
        let node = format!("/instance_map/maps/{index}/object");
        let object = check_entry_object(entry, &node, self.ctx)?;
        if !self.seen.insert(object.clone()) {
            return Err(self.ctx.spans.semantic(
                &node,
                ErrorCode::DuplicateIdent,
                format!("duplicate instance map entry for object `{}`", object.0),
            ));
        }
        let entry_check = EntryCheck {
            index,
            object: &object,
            declared: self.ctx.row_index.get(&object.0),
            spans: self.ctx.spans,
            mapped: BTreeMap::new(),
        };
        let rows = check_entry_rows(entry, entry_check)?;
        if !rows.is_empty() {
            self.maps.insert(object, rows);
        }
        Ok(())
    }
}

/// The entry's object: a valid ident naming a declared schema object.
fn check_entry_object(
    entry: &RawInstanceMapEntry,
    node: &str,
    ctx: &MapCtx,
) -> Result<Ident, ParseError> {
    check_ident(ctx.spans, node, &entry.object)?;
    let object = Ident(entry.object.clone());
    if !ctx.objects.contains_key(&object) {
        return Err(ctx.spans.semantic(
            node,
            ErrorCode::InvalidIdent,
            format!("instance map names undeclared object `{}`", entry.object),
        ));
    }
    Ok(object)
}

/// One entry's rows: every source key names a declared row of the entry's
/// object, every target is a well-formed ident (preserved, never
/// resolved here), and every declared row of the object is covered.
fn check_entry_rows(
    entry: &RawInstanceMapEntry,
    mut check: EntryCheck,
) -> Result<BTreeMap<RowId, Ident>, ParseError> {
    for (source, target) in &entry.rows {
        check_cell(source, target, &mut check)?;
    }
    check_coverage(&check)?;
    Ok(check.mapped)
}

/// One map cell: its source key must name a declared row of the entry's
/// object, and its target must be a well-formed ident (preserved, never
/// resolved here).
fn check_cell(source: &str, target: &str, check: &mut EntryCheck) -> Result<(), ParseError> {
    let Some(row) = check.declared.and_then(|rows| rows.get(source)) else {
        return Err(check.spans.syntax_key(
            &format!("/instance_map/maps/{}/rows", check.index),
            source,
            format!(
                "instance map keys row `{source}`, which is not a row of `{}`",
                check.object.0
            ),
        ));
    };
    if !is_ident(target) {
        return Err(check.spans.semantic(
            &format!("/instance_map/maps/{}/rows/{source}", check.index),
            ErrorCode::InvalidIdent,
            format!("instance map target `{target}` is not a well-formed row ident"),
        ));
    }
    check.mapped.insert(*row, Ident(target.to_string()));
    Ok(())
}

/// Coverage: every declared row of the object carries a cell. The first
/// uncovered row in sorted-name order is named at the per-object `rows`
/// node — never an implicit identity on the rest.
fn check_coverage(check: &EntryCheck) -> Result<(), ParseError> {
    let Some(rows) = check.declared else {
        return Ok(());
    };
    let uncovered = rows.iter().find(|(_, row)| !check.mapped.contains_key(row));
    let Some((name, _)) = uncovered else {
        return Ok(());
    };
    Err(check.spans.semantic(
        &format!("/instance_map/maps/{}/rows", check.index),
        ErrorCode::IncompleteInstanceMap,
        format!(
            "instance map for object `{}` omits row `{name}`; the map must be total",
            check.object.0
        ),
    ))
}

/// Every object with declared rows must carry an entry. A missing entry
/// is the same malformed claim as a missing row, reported at the section
/// node because no per-object map node exists to point at.
fn reject_uncovered_objects(ctx: &MapCtx, seen: &BTreeSet<Ident>) -> Result<(), ParseError> {
    for object in ctx.objects.keys() {
        let Some(rows) = ctx.row_index.get(&object.0) else {
            continue;
        };
        let Some((name, _)) = rows.iter().next() else {
            continue;
        };
        if !seen.contains(object) {
            return Err(ctx.spans.semantic(
                "/instance_map/maps",
                ErrorCode::IncompleteInstanceMap,
                format!(
                    "instance map has no entry for object `{}`; its row `{name}` is uncovered",
                    object.0
                ),
            ));
        }
    }
    Ok(())
}
