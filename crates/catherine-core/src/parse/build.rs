//! The normalized-model construction seam (dense rows, total value
//! vectors, canonical equation and key order) plus the presentation-name
//! side table. Split out of `mod.rs` so both files stay under the size
//! gate; no behavior change.

use super::rows::{self, RowNames};
use crate::model::{
    Arrow, Equation, Ident, Instance, Key, Model, RowId, RulePresentation, Schema, SchemaMap,
    Version,
};
use std::collections::BTreeMap;

/// The normalized model plus the name side table: dense rows per declared
/// object, total value vectors, canonical equation and key order. The
/// model is exactly what [`parse`] returned before this seam existed; the
/// names travel beside it ([`RowNames`]) and never enter [`Model`].
#[allow(clippy::too_many_arguments)]
pub(super) fn build_model(
    version: Version,
    objects: BTreeMap<Ident, ()>,
    arrows: BTreeMap<Ident, Arrow>,
    equations: Vec<Equation>,
    keys: Vec<Key>,
    row_index: BTreeMap<String, BTreeMap<String, RowId>>,
    values: BTreeMap<Ident, Vec<Option<RowId>>>,
    schema_map: Option<SchemaMap>,
    rule_presentation: Option<RulePresentation>,
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
        schema: Schema {
            objects,
            arrows,
            equations,
            keys,
        },
        instance: Instance {
            rows: rows_out,
            values,
        },
        schema_map,
        rule_presentation,
    };
    (model, names)
}
