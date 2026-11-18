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

/// Validate and normalize the declared instance map. `None` for an
/// absent or empty section (hash-neutral).
pub(super) fn build_instance_map(
    raw: &RawInstanceMap,
    objects: &BTreeMap<Ident, ()>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    spans: &Spans,
) -> Result<Option<InstanceMap>, ParseError> {
    let mut maps: BTreeMap<Ident, BTreeMap<RowId, Ident>> = BTreeMap::new();
    let mut seen: BTreeSet<Ident> = BTreeSet::new();
    for (index, entry) in raw.maps.iter().enumerate() {
        let object = check_entry_object(entry, index, objects, spans)?;
        if !seen.insert(object.clone()) {
            return Err(spans.semantic(
                &format!("/instance_map/maps/{index}/object"),
                ErrorCode::DuplicateIdent,
                format!("duplicate instance map entry for object `{}`", object.0),
            ));
        }
        let rows = check_entry_rows(entry, index, &object, row_index, spans)?;
        if !rows.is_empty() {
            maps.insert(object, rows);
        }
    }
    reject_uncovered_objects(objects, row_index, &seen, spans)?;
    if maps.is_empty() {
        return Ok(None);
    }
    Ok(Some(InstanceMap { maps }))
}

/// The entry's object: a valid ident naming a declared schema object.
fn check_entry_object(
    entry: &RawInstanceMapEntry,
    index: usize,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<Ident, ParseError> {
    let node = format!("/instance_map/maps/{index}/object");
    check_ident(spans, &node, &entry.object)?;
    let object = Ident(entry.object.clone());
    if !objects.contains_key(&object) {
        return Err(spans.semantic(
            &node,
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
    index: usize,
    object: &Ident,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    spans: &Spans,
) -> Result<BTreeMap<RowId, Ident>, ParseError> {
    let declared = row_index.get(&object.0);
    let mut mapped: BTreeMap<RowId, Ident> = BTreeMap::new();
    for (source, target) in &entry.rows {
        check_cell(declared, index, object, source, target, spans, &mut mapped)?;
    }
    check_coverage(declared, index, object, &mapped, spans)?;
    Ok(mapped)
}

/// One map cell: its source key must name a declared row of the entry's
/// object, and its target must be a well-formed ident (preserved, never
/// resolved here).
fn check_cell(
    declared: Option<&BTreeMap<String, RowId>>,
    index: usize,
    object: &Ident,
    source: &str,
    target: &str,
    spans: &Spans,
    mapped: &mut BTreeMap<RowId, Ident>,
) -> Result<(), ParseError> {
    let Some(row) = declared.and_then(|rows| rows.get(source)) else {
        return Err(spans.syntax_key(
            &format!("/instance_map/maps/{index}/rows"),
            source,
            format!(
                "instance map keys row `{source}`, which is not a row of `{}`",
                object.0
            ),
        ));
    };
    if !is_ident(target) {
        return Err(spans.semantic(
            &format!("/instance_map/maps/{index}/rows/{source}"),
            ErrorCode::InvalidIdent,
            format!("instance map target `{target}` is not a well-formed row ident"),
        ));
    }
    mapped.insert(*row, Ident(target.to_string()));
    Ok(())
}

/// Coverage: every declared row of the object carries a cell. The first
/// uncovered row in sorted-name order is named at the per-object `rows`
/// node — never an implicit identity on the rest.
fn check_coverage(
    declared: Option<&BTreeMap<String, RowId>>,
    index: usize,
    object: &Ident,
    mapped: &BTreeMap<RowId, Ident>,
    spans: &Spans,
) -> Result<(), ParseError> {
    let Some(rows) = declared else {
        return Ok(());
    };
    let uncovered = rows.iter().find(|(_, row)| !mapped.contains_key(row));
    let Some((name, _)) = uncovered else {
        return Ok(());
    };
    Err(spans.semantic(
        &format!("/instance_map/maps/{index}/rows"),
        ErrorCode::IncompleteInstanceMap,
        format!(
            "instance map for object `{}` omits row `{name}`; the map must be total",
            object.0
        ),
    ))
}

/// Every object with declared rows must carry an entry. A missing entry
/// is the same malformed claim as a missing row, reported at the section
/// node because no per-object map node exists to point at.
fn reject_uncovered_objects(
    objects: &BTreeMap<Ident, ()>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    seen: &BTreeSet<Ident>,
    spans: &Spans,
) -> Result<(), ParseError> {
    for object in objects.keys() {
        let Some(rows) = row_index.get(&object.0) else {
            continue;
        };
        let Some((name, _)) = rows.iter().next() else {
            continue;
        };
        if !seen.contains(object) {
            return Err(spans.semantic(
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
