//! Purpose: the Δ schema-map parse pass (add-delta-migration phase A)
//! — the additive 1.1 section is validated on its own: the target
//! schema runs the same object/arrow/equation/key/coproduct/pullback
//! passes as the document schema (own pointer root), and the assignment
//! maps are total over the declared source elements with targets declared
//! in the target.
//!
//! Rationale: declared data only, but declared *typed* data — every
//! mapped arrow path is checked composable and endpoint-typed against
//! the mapped objects (catherine-kpn.7); execution (pullback reindex,
//! lost-data enumeration) remains phase B
//! (`catherine_core::migrate`). A partial map — or a map whose arrow
//! paths do not compose between the mapped objects' images — is the
//! labeled `partial_schema_map` build error naming the offending
//! element.

use super::coproducts::check_coproducts;
use super::keys::check_keys;
use super::pullbacks::check_pullbacks;
use super::schema::{check_arrows, check_equations, check_objects, RawSchema};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Ident, Path, Schema, SchemaMap};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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
    let spans = target_spans(spans);
    let objects = check_objects(raw, &spans)?;
    let arrows = check_arrows(raw, &objects, &spans)?;
    let equations = check_equations(&raw.equations, &arrows, &spans)?;
    let keys = check_keys(raw, &objects, &arrows, &spans)?;
    let coproducts = check_coproducts(raw, &objects, &arrows, &spans)?;
    let pullbacks = check_pullbacks(raw, &objects, &arrows, &spans)?;
    Ok(Schema {
        objects,
        arrows,
        equations,
        keys,
        coproducts,
        pullbacks,
    })
}

/// The target schema's span view, re-rooted at `/schema`: the target runs
/// the document schema's passes verbatim, whose pointers are `/schema/...`.
/// Without the re-root those pointers would miss the target's nodes and
/// fall back to (1,1) — or silently land on a same-indexed document node.
fn target_spans(spans: &Spans) -> Spans {
    const PREFIX: &str = "/schema_map/target_schema";
    let mut target = Spans::default();
    for (pointer, pos) in &spans.nodes {
        if let Some(rest) = pointer.strip_prefix(PREFIX) {
            target.nodes.insert(format!("/schema{rest}"), *pos);
        }
    }
    for ((object, key), pos) in &spans.keys {
        if let Some(rest) = object.strip_prefix(PREFIX) {
            target
                .keys
                .insert((format!("/schema{rest}"), key.clone()), *pos);
        }
    }
    target
}

/// Build the typed schema map from the raw 1.1 section: totality over
/// the declared source objects and arrows (a missing entry is the
/// labeled `partial_schema_map` error naming the unmapped element),
/// assignment targets declared in the target schema, explicit-drop
/// `null` assignments, and composable-path endpoint typing (a
/// non-functorial map is a labeled refusal, never deferred to
/// execution).
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
    check_arrow_typing(&mapped_objects, arrows, &mapped_arrows, &target, spans)?;
    Ok(SchemaMap {
        target,
        objects: mapped_objects,
        arrows: mapped_arrows,
    })
}

/// The composable-path typing check (catherine-kpn.7): every mapped
/// arrow's path must compose from the source object's image to the
/// target object's image in the target schema. An endpoint or
/// composition mismatch is the labeled `partial_schema_map` error
/// naming the arrow and the objects involved.
fn check_arrow_typing(
    mapped_objects: &BTreeMap<Ident, Option<Ident>>,
    arrows: &BTreeMap<Ident, crate::model::Arrow>,
    mapped_arrows: &BTreeMap<Ident, Option<Path>>,
    target: &Schema,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    for (name, path) in mapped_arrows {
        let Some(path) = path else {
            continue;
        };
        let typed: Vec<&crate::model::Arrow> = path
            .0
            .iter()
            .map(|element| &target.arrows[element])
            .collect();
        let pointer = format!("/schema_map/arrows/{}", name.0);
        check_path_source(
            &pointer,
            name,
            &arrows[name],
            path,
            &typed,
            mapped_objects,
            spans,
        )?;
        check_path_composes(&pointer, name, path, &typed, spans)?;
        check_path_target(
            &pointer,
            name,
            &arrows[name],
            path,
            &typed,
            mapped_objects,
            spans,
        )?;
    }
    Ok(())
}

/// The path's source endpoint: the first path arrow must be sourced at
/// the source object's image — skipped when the source object is
/// dropped explicitly (`null`), whose image does not exist.
fn check_path_source(
    pointer: &str,
    name: &Ident,
    source: &crate::model::Arrow,
    path: &Path,
    typed: &[&crate::model::Arrow],
    mapped_objects: &BTreeMap<Ident, Option<Ident>>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    let Some(image) = image_of(mapped_objects, &source.src) else {
        return Ok(());
    };
    let first = typed[0].src.0.as_str();
    let path_head = path.0[0].0.as_str();
    if first == image.0 {
        return Ok(());
    }
    Err(spans.semantic(
        pointer,
        ErrorCode::PartialSchemaMap,
        format!(
            "arrow `{}` maps `{}` -> `{}` onto [`{}`], whose source endpoint `{first}` is not `{}`, the image of `{}` — the schema map is not functorial",
            name.0, source.src.0, source.tgt.0, path_head, image.0, source.src.0
        ),
    ))
}

/// The path's target endpoint: the last path arrow must land at the
/// target object's image — skipped when the target object is dropped
/// explicitly (`null`).
fn check_path_target(
    pointer: &str,
    name: &Ident,
    source: &crate::model::Arrow,
    path: &Path,
    typed: &[&crate::model::Arrow],
    mapped_objects: &BTreeMap<Ident, Option<Ident>>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    let Some(image) = image_of(mapped_objects, &source.tgt) else {
        return Ok(());
    };
    let last = typed[typed.len() - 1].tgt.0.as_str();
    let path_tail = path.0[path.0.len() - 1].0.as_str();
    if last == image.0 {
        return Ok(());
    }
    Err(spans.semantic(
        pointer,
        ErrorCode::PartialSchemaMap,
        format!(
            "arrow `{}` maps `{}` -> `{}` onto [`{}`], whose target endpoint `{last}` is not `{}`, the image of `{}` — the schema map is not functorial",
            name.0, source.src.0, source.tgt.0, path_tail, image.0, source.tgt.0
        ),
    ))
}

/// The path composes element-wise: each path arrow's target is the next
/// one's source.
fn check_path_composes(
    pointer: &str,
    name: &Ident,
    path: &Path,
    typed: &[&crate::model::Arrow],
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    for (index, window) in typed.windows(2).enumerate() {
        let (here, next) = (&window[0].tgt.0, &window[1].src.0);
        if here == next {
            continue;
        }
        return Err(spans.semantic(
            pointer,
            ErrorCode::PartialSchemaMap,
            format!(
                "arrow `{}` maps onto a path that does not compose: element `{}`'s target `{here}` is not element `{}`'s source `{next}`",
                name.0, path.0[index].0, path.0[index + 1].0
            ),
        ));
    }
    Ok(())
}

/// A source object's image in the target — `None` when the object is
/// dropped explicitly (`null` assignment).
fn image_of(mapped_objects: &BTreeMap<Ident, Option<Ident>>, object: &Ident) -> Option<Ident> {
    mapped_objects.get(object).cloned().flatten()
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
/// element a declared target arrow (endpoint typing runs in
/// `check_arrow_typing` once both assignment maps exist).
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
