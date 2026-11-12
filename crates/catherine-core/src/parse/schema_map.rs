//! Purpose: the Δ schema-map parse pass (add-delta-migration phase A)
//! — the additive 1.1 section is validated on its own: the target
//! schema runs the same object/arrow/equation passes as the document
//! schema (own pointer root), and the assignment maps are total over
//! the declared source elements with targets declared in the target.
//!
//! Rationale: declared data only — no functoriality is enforced here;
//! execution (pullback reindex, lost-data enumeration) is phase B
//! (`catherine_core::migrate`). A partial map is the labeled
//! `partial_schema_map` build error naming the unmapped element.

use super::schema::{check_arrows, check_equations, check_objects, RawSchema};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Ident, Path, Schema, SchemaMap};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub(super) struct RawSchemaMap {
    #[serde(default)]
    pub(super) target_schema: Option<RawSchema>,
    #[serde(default)]
    pub(super) objects: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pub(super) arrows: BTreeMap<String, Option<Vec<String>>>,
}

/// The raw schema map's target schema, validated by the same passes as
/// the document schema under the `/schema_map/target_schema` pointer
/// root.
fn validated_target(raw: &RawSchema, spans: &Spans) -> Result<Schema, crate::error::ParseError> {
    let objects = check_objects(raw, spans)?;
    let arrows = check_arrows(raw, &objects, spans)?;
    let equations = check_equations(raw, &arrows, spans)?;
    Ok(Schema {
        objects,
        arrows,
        equations,
    })
}

/// Build the typed schema map from the raw 1.1 section: totality over
/// the declared source objects and arrows (a missing entry is the
/// labeled `partial_schema_map` error naming the unmapped element),
/// assignment targets declared in the target schema, and explicit-drop
/// `null` assignments. Declared data only — no functoriality check
/// (execution is phase B).
pub(super) fn build_schema_map(
    raw: &RawSchemaMap,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, crate::model::Arrow>,
    spans: &Spans,
) -> Result<SchemaMap, crate::error::ParseError> {
    let Some(target_raw) = &raw.target_schema else {
        return Err(spans.semantic(
            "/schema_map",
            ErrorCode::PartialSchemaMap,
            "a schema map needs a `target_schema` — the destination of the migration".to_string(),
        ));
    };
    let target = validated_target(target_raw, spans)?;
    let mapped_objects = mapped_objects(raw, objects, &target, spans)?;
    let mapped_arrows = mapped_arrows(raw, arrows, &target, spans)?;
    Ok(SchemaMap {
        target,
        objects: mapped_objects,
        arrows: mapped_arrows,
    })
}

/// The object assignments: every declared source object carries an
/// entry — `null` drops it explicitly; a target must be declared.
fn mapped_objects(
    raw: &RawSchemaMap,
    objects: &BTreeMap<Ident, ()>,
    target: &Schema,
    spans: &Spans,
) -> Result<BTreeMap<Ident, Option<Ident>>, crate::error::ParseError> {
    let mut mapped = BTreeMap::new();
    for name in objects.keys() {
        let Some(assignment) = raw.objects.get(&name.0) else {
            return Err(spans.semantic(
                &format!("/schema_map/objects/{}", name.0),
                ErrorCode::PartialSchemaMap,
                format!("schema map is partial: object `{}` has no assignment — map it, or map it to null to drop it explicitly", name.0),
            ));
        };
        match assignment {
            Some(target_name) => {
                let target_ident = Ident(target_name.clone());
                if !target.objects.contains_key(&target_ident) {
                    return Err(spans.semantic(
                        &format!("/schema_map/objects/{}", name.0),
                        ErrorCode::PartialSchemaMap,
                        format!(
                            "object `{}` maps to `{target_name}`, which the target schema does not declare",
                            name.0
                        ),
                    ));
                }
                mapped.insert(name.clone(), Some(target_ident));
            }
            None => {
                mapped.insert(name.clone(), None);
            }
        };
    }
    reject_foreign_objects(raw, objects, spans)?;
    Ok(mapped)
}

/// Refuse assignments for source objects the document does not declare
/// (the map covers the DECLARED source elements only).
fn reject_foreign_objects(
    raw: &RawSchemaMap,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    for key in raw.objects.keys() {
        if !objects.contains_key(&Ident(key.clone())) {
            return Err(spans.semantic(
                &format!("/schema_map/objects/{key}"),
                ErrorCode::PartialSchemaMap,
                format!("schema map assigns `{key}`, which the document schema does not declare"),
            ));
        }
    }
    Ok(())
}

/// The arrow assignments: every declared source arrow carries an entry —
/// `null` drops it explicitly; a target path is non-empty and every path
/// element is a declared target arrow.
fn mapped_arrows(
    raw: &RawSchemaMap,
    arrows: &BTreeMap<Ident, crate::model::Arrow>,
    target: &Schema,
    spans: &Spans,
) -> Result<BTreeMap<Ident, Option<Path>>, crate::error::ParseError> {
    let mut mapped = BTreeMap::new();
    for name in arrows.keys() {
        let Some(assignment) = raw.arrows.get(&name.0) else {
            return Err(spans.semantic(
                &format!("/schema_map/arrows/{}", name.0),
                ErrorCode::PartialSchemaMap,
                format!("schema map is partial: arrow `{}` has no assignment — map it, or map it to null to drop it explicitly", name.0),
            ));
        };
        match assignment {
            Some(path) => {
                mapped.insert(
                    name.clone(),
                    declared_path(
                        path,
                        target,
                        spans,
                        &format!("/schema_map/arrows/{}", name.0),
                    )?,
                );
            }
            None => {
                mapped.insert(name.clone(), None);
            }
        };
    }
    reject_foreign_arrows(raw, arrows, spans)?;
    Ok(mapped)
}

/// The declared target path for one source arrow: non-empty, and every
/// element a declared target arrow (target-declaredness only — the
/// composable-path typing check is phase B).
fn declared_path(
    path: &[String],
    target: &Schema,
    spans: &Spans,
    pointer: &str,
) -> Result<Option<Path>, crate::error::ParseError> {
    if path.is_empty() {
        return Err(spans.semantic(
            pointer,
            ErrorCode::PartialSchemaMap,
            "arrow maps to an empty path".to_string(),
        ));
    }
    for (index, element) in path.iter().enumerate() {
        let element_ident = Ident(element.clone());
        if !target.arrows.contains_key(&element_ident) {
            return Err(spans.semantic(
                &format!("{pointer}/{index}"),
                ErrorCode::PartialSchemaMap,
                format!(
                    "arrow maps to path element `{element}`, which the target schema does not declare"
                ),
            ));
        }
    }
    let path_idents = path.iter().map(|element| Ident(element.clone())).collect();
    Ok(Some(Path(path_idents)))
}

/// Refuse assignments for source arrows the document does not declare.
fn reject_foreign_arrows(
    raw: &RawSchemaMap,
    arrows: &BTreeMap<Ident, crate::model::Arrow>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    for key in raw.arrows.keys() {
        if !arrows.contains_key(&Ident(key.clone())) {
            return Err(spans.semantic(
                &format!("/schema_map/arrows/{key}"),
                ErrorCode::PartialSchemaMap,
                format!("schema map assigns `{key}`, which the document schema does not declare"),
            ));
        }
    }
    Ok(())
}
