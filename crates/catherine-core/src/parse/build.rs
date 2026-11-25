//! The normalized-model construction seam (dense rows, total value
//! vectors, canonical equation, key, coproduct, and pullback order) plus
//! the presentation-name side table. Split out of `mod.rs` so both files
//! stay under the size gate; no behavior change.

use super::rows::{self, RowNames};
use crate::model::{
    Arrow, Coproduct, Equation, Ident, Instance, InstanceMap, Key, Model, Pullback, RowId,
    RulePresentation, Schema, SchemaMap, Version,
};
use std::collections::BTreeMap;

/// Everything the normalized model is assembled from: the checked schema
/// and instance halves plus the validated additive sections. The schema
/// declarations travel as one [`SchemaParts`] bundle, so `build_model`
/// stays under the arity and size gates.
pub(super) struct ModelInput {
    pub(super) schema: SchemaParts,
    pub(super) row_index: BTreeMap<String, BTreeMap<String, RowId>>,
    pub(super) values: BTreeMap<Ident, Vec<Option<RowId>>>,
    pub(super) schema_map: Option<SchemaMap>,
    pub(super) rule_presentation: Option<RulePresentation>,
    pub(super) instance_map: Option<InstanceMap>,
}

/// The schema half of [`ModelInput`]: the declaration lists as
/// `normalized_schema` consumes them.
pub(super) struct SchemaParts {
    pub(super) objects: BTreeMap<Ident, ()>,
    pub(super) arrows: BTreeMap<Ident, Arrow>,
    pub(super) equations: Vec<Equation>,
    pub(super) keys: Vec<Key>,
    pub(super) coproducts: Vec<Coproduct>,
    pub(super) pullbacks: Vec<Pullback>,
}

/// The normalized model plus the name side table: dense rows per declared
/// object, total value vectors, canonical equation, key, coproduct, and
/// pullback order. The model is exactly what [`parse`] returned before
/// this seam existed; the names travel beside it ([`RowNames`]) and never
/// enter [`Model`].
pub(super) fn build_model(version: Version, input: ModelInput) -> (Model, RowNames) {
    let names = rows::row_names(&input.schema.objects, &input.row_index);
    let rows_out = dense_rows(&input.schema.objects, &input.row_index);
    let schema = normalized_schema(input.schema);
    let model = Model {
        format_version: version,
        schema,
        instance: Instance {
            rows: rows_out,
            values: input.values,
        },
        schema_map: input.schema_map,
        rule_presentation: input.rule_presentation,
        instance_map: input.instance_map,
    };
    (model, names)
}

/// Dense rows per declared object: `0..n` in id order, sized by the
/// object's declared row count.
fn dense_rows(
    objects: &BTreeMap<Ident, ()>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
) -> BTreeMap<Ident, Vec<RowId>> {
    objects
        .keys()
        .map(|o| {
            let n = row_index.get(&o.0).map_or(0, BTreeMap::len);
            (o.clone(), (0..n).map(RowId).collect())
        })
        .collect()
}

/// The normalized schema: the merged name sets in canonical order and the
/// declaration lists already sorted by their canonical keys.
fn normalized_schema(parts: SchemaParts) -> Schema {
    Schema {
        objects: parts.objects,
        arrows: parts.arrows,
        equations: parts.equations,
        keys: parts.keys,
        coproducts: parts.coproducts,
        pullbacks: parts.pullbacks,
        // Attached by `validate_with` from the raw document's
        // `schema.stored` (ticket `catherine-kw9.3`); empty here.
        stored: BTreeMap::new(),
    }
}
