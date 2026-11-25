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

use super::equations::check_equations;
use super::schema::{check_arrows, check_objects, RawSchema};
use super::spans::Spans;
use super::{check_structure, Decl};
use crate::error::ErrorCode;
use crate::model::{Coproduct, Ident, Key, Path, Pullback, Schema, SchemaMap};
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
    let (keys, coproducts, pullbacks) = structural_checks(raw, &objects, &arrows, &spans)?;
    Ok(Schema {
        objects,
        arrows,
        equations,
        keys,
        coproducts,
        pullbacks,
        // Presentation metadata (ticket `catherine-kw9.3`), carried so a
        // round-tripped target schema keeps its declared sources.
        stored: raw
            .stored
            .iter()
            .map(|(arrow, column)| (Ident(arrow.clone()), column.clone()))
            .collect(),
    })
}

/// The structural relations of a schema, checked in document order.
type Structural = (Vec<Key>, Vec<Coproduct>, Vec<Pullback>);

/// The keys, coproducts, and pullbacks over the target's checked objects
/// and arrows — the same triple behind every structural pass, so the
/// target reuses the document schema's `check_structure` verbatim.
fn structural_checks(
    raw: &RawSchema,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, crate::model::Arrow>,
    spans: &Spans,
) -> Result<Structural, crate::error::ParseError> {
    check_structure(
        raw,
        &Decl {
            objects,
            arrows,
            spans,
        },
    )
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

/// The schema map's source-side inputs: the document schema's declared
/// objects and arrows, and the span source the assignment checks report
/// through. Built once by the caller and shared by every assignment pass.
pub(super) struct SourceMap<'a> {
    pub(super) objects: &'a BTreeMap<Ident, ()>,
    pub(super) arrows: &'a BTreeMap<Ident, crate::model::Arrow>,
    pub(super) spans: &'a Spans,
}

/// The Δ target under assignment: the validated target schema and the
/// span source of its nodes.
struct TargetSchema<'a> {
    schema: &'a Schema,
    spans: &'a Spans,
}

/// The map's raw target schema: a migration has nowhere to go without
/// one, so its absence is the labeled `partial_schema_map` at the map.
fn required_target<'a>(
    raw: &'a RawSchemaMap,
    source: &SourceMap,
) -> Result<&'a RawSchema, crate::error::ParseError> {
    raw.target_schema.as_ref().ok_or_else(|| {
        source.spans.semantic(
            "/schema_map",
            ErrorCode::PartialSchemaMap,
            "a schema map needs a `target_schema` — the destination of the migration".to_string(),
        )
    })
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
    source: &SourceMap,
) -> Result<SchemaMap, crate::error::ParseError> {
    let target_raw = required_target(raw, source)?;
    let target = validated_target(target_raw, source.spans)?;
    let into_target = &TargetSchema {
        schema: &target,
        spans: source.spans,
    };
    let objects = mapped_objects(raw, source, into_target)?;
    let arrows = mapped_arrows(raw, source, into_target)?;
    check_arrow_typing(
        &arrows,
        &MapTyping {
            mapped_objects: &objects,
            arrows: source.arrows,
            target: &target,
            spans: source.spans,
        },
    )?;
    Ok(SchemaMap {
        target,
        objects,
        arrows,
    })
}

/// The typing context of one built schema map: the mapped objects, the
/// source arrows, the validated target, and the span source.
struct MapTyping<'a> {
    mapped_objects: &'a BTreeMap<Ident, Option<Ident>>,
    arrows: &'a BTreeMap<Ident, crate::model::Arrow>,
    target: &'a Schema,
    spans: &'a Spans,
}

/// The per-arrow check bundle: the arrow's pointer, name, source arrow,
/// mapped path, the object images, and the span source.
struct PathCheck<'a> {
    pointer: &'a str,
    name: &'a Ident,
    source: &'a crate::model::Arrow,
    path: &'a Path,
    typed: &'a [&'a crate::model::Arrow],
    mapped_objects: &'a BTreeMap<Ident, Option<Ident>>,
    spans: &'a Spans,
}

/// The composable-path typing check (catherine-kpn.7): every mapped
/// arrow's path must compose from the source object's image to the
/// target object's image in the target schema.
fn check_arrow_typing(
    mapped_arrows: &BTreeMap<Ident, Option<Path>>,
    typing: &MapTyping,
) -> Result<(), crate::error::ParseError> {
    for (name, path) in mapped_arrows {
        let Some(path) = path else {
            continue;
        };
        typing.check_arrow(name, path)?;
    }
    Ok(())
}

impl MapTyping<'_> {
    /// One mapped arrow: endpoint and composition typing over its path.
    fn check_arrow(&self, name: &Ident, path: &Path) -> Result<(), crate::error::ParseError> {
        let typed: Vec<&crate::model::Arrow> = path
            .0
            .iter()
            .map(|element| &self.target.arrows[element])
            .collect();
        let pointer = format!("/schema_map/arrows/{}", name.0);
        let check = PathCheck {
            pointer: &pointer,
            name,
            source: &self.arrows[name],
            path,
            typed: &typed,
            mapped_objects: self.mapped_objects,
            spans: self.spans,
        };
        check.check_source()?;
        check.check_composes()?;
        check.check_target()
    }
}

impl PathCheck<'_> {
    /// The path's source endpoint: the first path arrow must be sourced
    /// at the source object's image — skipped when the source object is
    /// dropped explicitly (`null`), whose image does not exist.
    fn check_source(&self) -> Result<(), crate::error::ParseError> {
        let Some(image) = image_of(self.mapped_objects, &self.source.src) else {
            return Ok(());
        };
        let head = self.path.0[0].0.as_str();
        let endpoint = self.typed[0].src.0.as_str();
        if endpoint == image.0 {
            return Ok(());
        }
        Err(self.spans.semantic(
            self.pointer,
            ErrorCode::PartialSchemaMap,
            format!(
                "arrow `{}` maps `{}` -> `{}` onto [`{head}`], whose source endpoint `{endpoint}` is not `{}`, the image of `{}` — the schema map is not functorial",
                self.name.0, self.source.src.0, self.source.tgt.0, image.0, self.source.src.0
            ),
        ))
    }

    /// The path's target endpoint: the last path arrow must land at the
    /// target object's image — skipped when the target object is dropped
    /// explicitly (`null`).
    fn check_target(&self) -> Result<(), crate::error::ParseError> {
        let Some(image) = image_of(self.mapped_objects, &self.source.tgt) else {
            return Ok(());
        };
        let tail = self.path.0[self.path.0.len() - 1].0.as_str();
        let endpoint = self.typed[self.typed.len() - 1].tgt.0.as_str();
        if endpoint == image.0 {
            return Ok(());
        }
        Err(self.spans.semantic(
            self.pointer,
            ErrorCode::PartialSchemaMap,
            format!(
                "arrow `{}` maps `{}` -> `{}` onto [`{tail}`], whose target endpoint `{endpoint}` is not `{}`, the image of `{}` — the schema map is not functorial",
                self.name.0, self.source.src.0, self.source.tgt.0, image.0, self.source.tgt.0
            ),
        ))
    }

    /// The path composes element-wise: each path arrow's target is the
    /// next one's source.
    fn check_composes(&self) -> Result<(), crate::error::ParseError> {
        for (index, window) in self.typed.windows(2).enumerate() {
            let (here, next) = (&window[0].tgt.0, &window[1].src.0);
            if here == next {
                continue;
            }
            return Err(self.spans.semantic(
                self.pointer,
                ErrorCode::PartialSchemaMap,
                format!(
                    "arrow `{}` maps onto a path that does not compose: element `{}`'s target `{here}` is not element `{}`'s source `{next}`",
                    self.name.0, self.path.0[index].0, self.path.0[index + 1].0
                ),
            ));
        }
        Ok(())
    }
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
    source: &SourceMap,
    target: &TargetSchema,
) -> Result<BTreeMap<Ident, Option<Ident>>, crate::error::ParseError> {
    let mut mapped = BTreeMap::new();
    for name in source.objects.keys() {
        mapped.insert(name.clone(), object_image(raw, name, target)?);
    }
    reject_foreign_objects(raw, source.objects, source.spans)?;
    Ok(mapped)
}

/// One source object's assignment: required, and `null` drops it
/// explicitly; a target must be declared in the target schema.
fn object_image(
    raw: &RawSchemaMap,
    name: &Ident,
    target: &TargetSchema,
) -> Result<Option<Ident>, crate::error::ParseError> {
    let pointer = format!("/schema_map/objects/{}", name.0);
    let Some(assignment) = raw.objects.get(&name.0) else {
        return Err(target.spans.semantic(
            &pointer,
            ErrorCode::PartialSchemaMap,
            format!("schema map is partial: object `{}` has no assignment — map it, or map it to null to drop it explicitly", name.0),
        ));
    };
    let Some(target_name) = assignment else {
        return Ok(None);
    };
    let target_ident = Ident(target_name.clone());
    if !target.schema.objects.contains_key(&target_ident) {
        return Err(target.spans.semantic(
            &pointer,
            ErrorCode::PartialSchemaMap,
            format!(
                "object `{}` maps to `{target_name}`, which the target schema does not declare",
                name.0
            ),
        ));
    }
    Ok(Some(target_ident))
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

mod arrows;

use arrows::mapped_arrows;
