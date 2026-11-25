//! Presentation-order rewriting of serialized documents: rotate row
//! lists and rename rows (with their table references) without touching
//! the model the document expresses. Pure JSON-level helpers for the
//! symmetric-ratchet round-trip properties.

use crate::common::Insist;
use serde_json::Value;

/// Per-arrow src/tgt object name maps, read from the schema's arrows.
type ObjectMap = std::collections::BTreeMap<String, String>;

/// Rotate every `rows` array of the document by `k` — a pure
/// presentation-order permutation (tables reference row *names*, so the
/// model is unchanged; only the serialized order moves).
pub(super) fn rotate_row_lists(value: &mut Value, k: usize) {
    let Some(rows) = value
        .get_mut("instance")
        .and_then(|instance| instance.get_mut("rows"))
    else {
        return;
    };
    let Some(rows) = rows.as_object_mut() else {
        return;
    };
    for list in rows.values_mut() {
        rotate_list(list, k);
    }
}

/// Rotate one serialized row list, keeping single-row lists untouched
/// (a rotation of one element is the identity).
fn rotate_list(list: &mut Value, k: usize) {
    let Some(array) = list.as_array_mut() else {
        return;
    };
    if array.len() > 1 {
        let shift = k % array.len();
        array.rotate_left(shift);
    }
}

/// Rewrite the document's presentation row names per object (reversed
/// dense order — a bijection), remapping table keys and values through
/// the same rename. The result expresses the same model under different
/// names.
pub(super) fn rename_presentation_rows(doc: &str) -> String {
    let mut value: Value = serde_json::from_str(doc).insist("canonical bytes are JSON");
    let (arrow_src, arrow_tgt) = arrow_endpoint_maps(&value);
    let renames = rename_rows(&mut value);
    let maps = RenameMaps {
        arrow_src: &arrow_src,
        arrow_tgt: &arrow_tgt,
        renames: &renames,
    };
    rename_tables(&mut value, &maps);
    serde_json::to_string(&value).insist("renamed document serializes")
}

fn arrow_endpoint_maps(value: &Value) -> (ObjectMap, ObjectMap) {
    let mut arrow_src: ObjectMap = Default::default();
    let mut arrow_tgt: ObjectMap = Default::default();
    for a in value["schema"]["arrows"].as_array().insist("arrows") {
        arrow_src.insert(
            a["name"].as_str().insist("arrow name").to_string(),
            a["src"].as_str().insist("arrow src").to_string(),
        );
        arrow_tgt.insert(
            a["name"].as_str().insist("arrow name").to_string(),
            a["tgt"].as_str().insist("arrow tgt").to_string(),
        );
    }
    (arrow_src, arrow_tgt)
}

/// Per-object row renames written into the document: each row name is
/// replaced by the reversed dense order (`zz<n - 1 - i>` — a bijection).
fn rename_rows(
    value: &mut Value,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> {
    let rows = value["instance"]["rows"].as_object_mut().insist("rows");
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
    let arr = list.as_array_mut().insist("row list");
    let n = arr.len();
    let mut map = std::collections::BTreeMap::new();
    for (i, row) in arr.iter_mut().enumerate() {
        let old = row.as_str().insist("row name").to_string();
        let new = format!("zz{}", n - 1 - i);
        map.insert(old, new.clone());
        *row = Value::String(new);
    }
    map
}

/// The per-arrow endpoint maps beside the per-object row renames, held
/// together so the table remap helpers stay within the two-parameter
/// test limit.
struct RenameMaps<'a> {
    arrow_src: &'a ObjectMap,
    arrow_tgt: &'a ObjectMap,
    renames: &'a std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
}

/// Remap every table's keys and values through the per-object renames.
fn rename_tables(value: &mut Value, maps: &RenameMaps<'_>) {
    let tables = value["instance"]["tables"].as_object_mut().insist("tables");
    for (arrow_name, table) in tables.iter_mut() {
        let key_map = maps
            .renames
            .get(&maps.arrow_src[arrow_name])
            .insist("rename map for src object");
        let value_map = maps
            .renames
            .get(&maps.arrow_tgt[arrow_name])
            .insist("rename map for tgt object");
        rename_table(table, (key_map, value_map));
    }
}

/// Remap one table's row keys and cell values through the two rename
/// maps (src names the keys, tgt names the values).
fn rename_table(
    table: &mut Value,
    maps: (
        &std::collections::BTreeMap<String, String>,
        &std::collections::BTreeMap<String, String>,
    ),
) {
    let (key_map, value_map) = maps;
    let entries: Vec<(String, Value)> = table
        .as_object()
        .insist("table")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    *table.as_object_mut().insist("table") = entries
        .into_iter()
        .map(|(k, v)| renamed_entry((k, v), (key_map, value_map)))
        .collect();
}

/// One renamed table entry: the key through the src map, a string cell
/// value through the tgt map.
fn renamed_entry(
    entry: (String, Value),
    maps: (
        &std::collections::BTreeMap<String, String>,
        &std::collections::BTreeMap<String, String>,
    ),
) -> (String, Value) {
    let (k, v) = entry;
    let (key_map, value_map) = maps;
    let new_key = key_map.get(&k).insist("renamed row key").clone();
    let new_value = match v {
        Value::Null => Value::Null,
        Value::String(s) => Value::String(value_map.get(&s).insist("renamed row value").clone()),
        other => other,
    };
    (new_key, new_value)
}
