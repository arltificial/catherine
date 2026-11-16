//! Purpose: the declared instance-map commutation check (openspec change
//! `add-instance-morphism`, task 6) — execute a source document's
//! `instance_map` against a target instance of the identical schema and
//! report each failing commutation square as a witness-bearing finding.
//!
//! Responsibilities: require the source map (an absent one is a caller
//! concern, reported as [`NaturalityError::MissingInstanceMap`]), resolve
//! every declared target name against the target instance at construction
//! (an unresolvable cell is [`NaturalityError::UnknownMapTarget`], never a
//! finding), then sweep every schema arrow and every source row comparing
//! `map ∘ arrow` with `arrow ∘ map`. A mismatch emits one
//! `violation`/`executed` finding whose witness names the arrow, the
//! source row, and both composed values. No new finding kinds are
//! introduced.
//!
//! Rationale: design Decisions 2-4 — malformed claims are labeled errors
//! before any check runs, breaks are violations with a witness (the
//! equation-witness shape discipline), and the operation is same-schema
//! only (the CLI reuses `distinguish`'s schema check). Resolution is
//! construction-time because map cells name rows of another document;
//! the parser deliberately preserves them as data.

use catherine_core::findings::{
    EpistemicLabel, Finding, FindingKind, NaturalityWitness, ViolationWitness,
};
use catherine_core::model::{Ident, InstanceMap, Model, RowId};
use catherine_core::parse::RowNames;
use std::collections::BTreeMap;

/// Why a naturality run could not construct its check. These are labeled
/// input defects, never findings: a malformed claim has no decidable
/// reading, so it is rejected before the sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NaturalityError {
    /// The source document declares no `instance_map`; there is no claim
    /// to execute (the CLI turns this into a warning, never a silent
    /// clean).
    MissingInstanceMap,
    /// A map cell names a target row the target instance does not declare.
    UnknownMapTarget {
        /// The source object the cell belongs to.
        object: Ident,
        /// The mapped source row's presentation name.
        source_row: String,
        /// The target row name the cell declares.
        target_row: String,
    },
    /// The map does not cover one of the source instance's rows. Parse
    /// rejects this shape, so the arm exists for programmatically
    /// assembled models; it is never a finding.
    IncompleteInstanceMap {
        /// The source object whose row is uncovered.
        object: Ident,
        /// The uncovered dense row id.
        row: RowId,
    },
}

/// Resolve and execute the source document's instance map against the
/// target instance. The source and target schemas are the caller's
/// precondition (the CLI reuses its `schema_mismatch` check); this
/// function assumes identical schemas and same-named objects.
pub fn naturality(
    source: &Model,
    target: &Model,
    source_names: &RowNames,
    target_names: &RowNames,
) -> Result<Vec<Finding>, NaturalityError> {
    let map = source
        .instance_map
        .as_ref()
        .ok_or(NaturalityError::MissingInstanceMap)?;
    let resolved = resolve_map(map, source, source_names, target_names)?;
    Ok(sweep(source, target, &resolved))
}

/// The construction-time target resolution: every source row of every
/// object gets the target row its map cell names. Target names are looked
/// up in the target instance's name side table, so an unknown cell is a
/// labeled error naming the object, the source row, and the missing
/// target row.
fn resolve_map(
    map: &InstanceMap,
    source: &Model,
    source_names: &RowNames,
    target_names: &RowNames,
) -> Result<BTreeMap<Ident, Vec<RowId>>, NaturalityError> {
    let mut resolved: BTreeMap<Ident, Vec<RowId>> = BTreeMap::new();
    for (object, rows) in &source.instance.rows {
        let mut targets = Vec::with_capacity(rows.len());
        if rows.is_empty() {
            resolved.insert(object.clone(), targets);
            continue;
        }
        let cells = map.maps.get(object);
        for row in rows {
            let Some(target_name) = cells.and_then(|cells| cells.get(row)) else {
                return Err(NaturalityError::IncompleteInstanceMap {
                    object: object.clone(),
                    row: *row,
                });
            };
            let target = target_names
                .get(object)
                .and_then(|names| names.iter().position(|name| name == target_name))
                .map(RowId)
                .ok_or_else(|| NaturalityError::UnknownMapTarget {
                    object: object.clone(),
                    source_row: source_row_name(source_names, object, *row),
                    target_row: target_name.0.clone(),
                })?;
            targets.push(target);
        }
        resolved.insert(object.clone(), targets);
    }
    Ok(resolved)
}

/// One source row's presentation name, for the construction-time error.
fn source_row_name(source_names: &RowNames, object: &Ident, row: RowId) -> String {
    source_names
        .get(object)
        .and_then(|names| names.get(row.0))
        .map(|name| name.0.clone())
        .unwrap_or_else(|| format!("#{}", row.0))
}

/// The commutation sweep: for every arrow and every source row, compare
/// `map ∘ arrow` (lhs) with `arrow ∘ map` (rhs). A composition through a
/// dangling cell is `None`; two unequal values — including one unresolved
/// side — are a witnessed break.
fn sweep(source: &Model, target: &Model, resolved: &BTreeMap<Ident, Vec<RowId>>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (arrow_name, arrow) in &source.schema.arrows {
        let Some(rows) = source.instance.rows.get(&arrow.src) else {
            continue;
        };
        for row in rows {
            if let Some(finding) = square(source, target, arrow_name, *row, resolved) {
                findings.push(finding);
            }
        }
    }
    findings.sort();
    findings
}

/// One commutation square: `map ∘ arrow` (lhs) against `arrow ∘ map`
/// (rhs). A composition through a dangling cell is `None`; two unequal
/// values — including one unresolved side — are a witnessed break.
fn square(
    source: &Model,
    target: &Model,
    arrow_name: &Ident,
    row: RowId,
    resolved: &BTreeMap<Ident, Vec<RowId>>,
) -> Option<Finding> {
    let arrow = source.schema.arrows.get(arrow_name)?;
    let src_map = resolved.get(&arrow.src)?;
    let tgt_map = resolved.get(&arrow.tgt);
    let mapped_row = src_map.get(row.0).copied();
    let lhs = source
        .instance
        .values
        .get(arrow_name)
        .and_then(|cells| cells.get(row.0))
        .copied()
        .flatten()
        .and_then(|image| tgt_map.and_then(|map| map.get(image.0)).copied());
    let rhs = arrow_image(target, arrow_name, mapped_row);
    if lhs == rhs {
        return None;
    }
    Some(Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Naturality(NaturalityWitness {
            arrow: arrow_name.clone(),
            source_row: row,
            lhs,
            rhs,
        })),
    })
}

/// One arrow's image at a row of another instance's object; `None` for a
/// dangling cell or an out-of-space row.
fn arrow_image(document: &Model, arrow_name: &Ident, row: Option<RowId>) -> Option<RowId> {
    row.and_then(|row| {
        document
            .instance
            .values
            .get(arrow_name)
            .and_then(|cells| cells.get(row.0))
            .copied()
            .flatten()
    })
}
