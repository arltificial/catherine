//! Purpose: the round-trip properties (proptest, never insta-snapshots):
//! `parse(serialize(arbitrary_model)) == model`, and the row-id
//! canonicalization property — the same structure presented under
//! different row names yields equal canonical bytes and equal hash
//! (design Decision 5; openspec scenarios "Round-trip preserves the
//! model" and "Byte-stable across input order").
//!
//! The arbitrary-model strategy and its well-typed construction live in
//! the sibling `model_fixture` module, split out of this file (with the
//! property assertion moved with them) to stay inside the file-size gate.

#[path = "roundtrip/model_fixture.rs"]
mod model_fixture;

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use model_fixture::arb_model;
use proptest::prelude::*;
use serde_json::Value;

/// Rewrite the document's presentation row names per object (reversed
/// dense order — a bijection), remapping table keys and values through
/// the same rename. The result expresses the same model under different
/// names.
fn rename_presentation_rows(doc: &str) -> String {
    let mut value: Value = serde_json::from_str(doc).expect("canonical bytes are JSON");
    let (arrow_src, arrow_tgt) = arrow_endpoint_maps(&value);
    let renames = rename_rows(&mut value);
    rename_tables(&mut value, &arrow_src, &arrow_tgt, &renames);
    serde_json::to_string(&value).expect("renamed document serializes")
}

/// Per-arrow src/tgt object name maps, read from the schema's arrows.
type ObjectMap = std::collections::BTreeMap<String, String>;

fn arrow_endpoint_maps(value: &Value) -> (ObjectMap, ObjectMap) {
    let mut arrow_src: ObjectMap = Default::default();
    let mut arrow_tgt: ObjectMap = Default::default();
    for a in value["schema"]["arrows"].as_array().expect("arrows") {
        arrow_src.insert(
            a["name"].as_str().expect("arrow name").to_string(),
            a["src"].as_str().expect("arrow src").to_string(),
        );
        arrow_tgt.insert(
            a["name"].as_str().expect("arrow name").to_string(),
            a["tgt"].as_str().expect("arrow tgt").to_string(),
        );
    }
    (arrow_src, arrow_tgt)
}

/// Per-object row renames written into the document: each row name is
/// replaced by the reversed dense order (`zz<n - 1 - i>` — a bijection).
fn rename_rows(
    value: &mut Value,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> {
    let rows = value["instance"]["rows"].as_object_mut().expect("rows");
    let mut renames: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, String>,
    > = Default::default();
    for (object, list) in rows.iter_mut() {
        renames.insert(object.clone(), rename_row_list(list));
    }
    renames
}

/// Rename one object's row list in place, returning its old → new map.
fn rename_row_list(list: &mut Value) -> std::collections::BTreeMap<String, String> {
    let arr = list.as_array_mut().expect("row list");
    let n = arr.len();
    let mut map = std::collections::BTreeMap::new();
    for (i, row) in arr.iter_mut().enumerate() {
        let old = row.as_str().expect("row name").to_string();
        let new = format!("zz{}", n - 1 - i);
        map.insert(old, new.clone());
        *row = Value::String(new);
    }
    map
}

/// Remap every table's keys and values through the per-object renames.
fn rename_tables(
    value: &mut Value,
    arrow_src: &ObjectMap,
    arrow_tgt: &ObjectMap,
    renames: &std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
) {
    let tables = value["instance"]["tables"].as_object_mut().expect("tables");
    for (arrow_name, table) in tables.iter_mut() {
        let key_map = renames
            .get(&arrow_src[arrow_name])
            .expect("rename map for src object");
        let value_map = renames
            .get(&arrow_tgt[arrow_name])
            .expect("rename map for tgt object");
        rename_table(table, key_map, value_map);
    }
}

/// Remap one table's row keys and cell values through the two rename
/// maps (src names the keys, tgt names the values).
fn rename_table(
    table: &mut Value,
    key_map: &std::collections::BTreeMap<String, String>,
    value_map: &std::collections::BTreeMap<String, String>,
) {
    let entries: Vec<(String, Value)> = table
        .as_object()
        .expect("table")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    *table.as_object_mut().expect("table") = entries
        .into_iter()
        .map(|(k, v)| renamed_entry(k, v, key_map, value_map))
        .collect();
}

/// One renamed table entry: the key through the src map, a string cell
/// value through the tgt map.
fn renamed_entry(
    k: String,
    v: Value,
    key_map: &std::collections::BTreeMap<String, String>,
    value_map: &std::collections::BTreeMap<String, String>,
) -> (String, Value) {
    let new_key = key_map.get(&k).expect("renamed row key").clone();
    let new_value = match v {
        Value::Null => Value::Null,
        Value::String(s) => Value::String(value_map.get(&s).expect("renamed row value").clone()),
        other => other,
    };
    (new_key, new_value)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `parse(serialize(arbitrary_model)) == model` — serialization is a
    /// parse fixed point for arbitrary valid models.
    #[test]
    fn roundtrip_preserves_the_model(model in arb_model()) {
        let bytes = serialize(&model);
        match parse(&bytes) {
            Ok(reparsed) => prop_assert_eq!(reparsed, model),
            Err(err) => prop_assert!(false, "re-parse failed: {err:?}; bytes were {bytes}"),
        }
    }

    /// Same structure under different presentation row names → equal
    /// canonical bytes and equal hash (the hash depends only on the
    /// model's structure, for *any* renaming — including order-reversing
    /// ones).
    #[test]
    fn presentation_row_names_never_reach_the_hash(model in arb_model()) {
        let doc = serialize(&model);
        let renamed = rename_presentation_rows(&doc);
        let original = parse(&doc).expect("canonical bytes parse");
        let re_presented = parse(&renamed).expect("renamed document parses");
        prop_assert_eq!(
            canonical_bytes(&original),
            canonical_bytes(&re_presented),
            "same structure, equal canonical bytes"
        );
        prop_assert_eq!(
            model_hash(&original),
            model_hash(&re_presented),
            "equal models hash equal under row renames"
        );
    }
}
