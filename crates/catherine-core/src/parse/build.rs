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

/// The normalized model plus the name side table: dense rows per declared
/// object, total value vectors, canonical equation, key, coproduct, and
/// pullback order. The model is exactly what [`parse`] returned before
/// this seam existed; the names travel beside it ([`RowNames`]) and never
/// enter [`Model`].
#[allow(clippy::too_many_arguments)]
pub(super) fn build_model(
    version: Version,
    objects: BTreeMap<Ident, ()>,
    arrows: BTreeMap<Ident, Arrow>,
    equations: Vec<Equation>,
    keys: Vec<Key>,
    coproducts: Vec<Coproduct>,
    pullbacks: Vec<Pullback>,
    row_index: BTreeMap<String, BTreeMap<String, RowId>>,
    values: BTreeMap<Ident, Vec<Option<RowId>>>,
    schema_map: Option<SchemaMap>,
    rule_presentation: Option<RulePresentation>,
    instance_map: Option<InstanceMap>,
) -> (Model, RowNames) {
    let names = rows::row_names(&objects, &row_index);
    let rows_out = objects
        .keys()
        .map(|o| {
            let n = row_index.get(&o.0).map_or(0, BTreeMap::len);
            (o.clone(), (0..n).map(RowId).collect())
        })
        .collect();
    let model = Model {
        format_version: version,
        schema: normalized_schema(objects, arrows, equations, keys, coproducts, pullbacks),
        instance: Instance {
            rows: rows_out,
            values,
        },
        schema_map,
        rule_presentation,
        instance_map,
    };
    (model, names)
}

/// The normalized schema: the merged name sets in canonical order and the
/// declaration lists already sorted by their canonical keys.
fn normalized_schema(
    objects: BTreeMap<Ident, ()>,
    arrows: BTreeMap<Ident, Arrow>,
    equations: Vec<Equation>,
    keys: Vec<Key>,
    coproducts: Vec<Coproduct>,
    pullbacks: Vec<Pullback>,
) -> Schema {
    Schema {
        objects,
        arrows,
        equations,
        keys,
        coproducts,
        pullbacks,
        // Attached by `validate_with` from the raw document's
        // `schema.stored` (ticket `catherine-kw9.3`); empty here.
        stored: BTreeMap::new(),
    }
}
