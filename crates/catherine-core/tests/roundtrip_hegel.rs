//! Purpose: the round-trip properties of the canonical document, drawn
//! by the hegel engine: `parse(serialize(arbitrary_model)) == model`,
//! the row-id canonicalization property — the same structure presented
//! under different row names yields equal canonical bytes and equal
//! hash — and the catherine-1ml ratchet, which guards the WL-pruning
//! assumption behind the canonicalizer's refined-partition branch skip
//! (`canonical/label.rs::solve`) with automorphism-rich shapes (design
//! Decision 5; openspec scenarios "Round-trip preserves the model" and
//! "Byte-stable across input order").
//!
//! Responsibilities:
//!   - `roundtrip_preserves_the_model` and
//!     `presentation_row_names_never_reach_the_hash`, both over the
//!     arbitrary well-typed model from `model_fixture::arb_model`.
//!   - `symmetric_ratchet_bounded_sizes`: a 32-case envelope over
//!     rotation cycles, identity-pair involutions, twin rows and full
//!     collapse at sizes 2..=4, with its builders, and the presentation
//!     rotation and rename helpers both properties apply.
//!
//! Rationale: one binary keeps the properties, the ratchet's builders
//! and the presentation rewrites in one place, with the drawn strategy
//! and its construction beside them in `model_fixture`. The ratchet
//! keeps its own explicit 32-case envelope: the automorphism-rich shapes
//! are the canonicalizer's combinatorial worst case, so it samples a
//! near-exhaustive small envelope (four shapes × sizes 2..=4 ×
//! rotations) instead of dominating the suite's wall-clock budget.
//! Larger symmetric instances are deliberately NOT sampled; a
//! CFI-family generator is a candidate follow-up ticket.

#[path = "roundtrip/model_fixture.rs"]
mod model_fixture;

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::hash::model_hash;
use catherine_core::model::{Ident, Instance, Model, RowId, Schema, Version};
use catherine_core::parse::parse;
use hegel::generators as gs;
use hegel::TestCase;
use model_fixture::{arb_model, cross_maps, dense_rows, object, self_loop, table};
use serde_json::Value;

/// `parse(serialize(arbitrary_model)) == model` — serialization is a
/// parse fixed point for arbitrary valid models.
#[hegel::test]
fn roundtrip_preserves_the_model(tc: TestCase) {
    let model = tc.draw(arb_model());
    let bytes = serialize(&model);
    match parse(&bytes) {
        Ok(reparsed) => assert_eq!(reparsed, model, "re-parse must preserve the model"),
        Err(err) => panic!("re-parse failed: {err:?}; bytes were {bytes}"),
    }
}

/// Same structure under different presentation row names → equal
/// canonical bytes and equal hash (the hash depends only on the
/// model's structure, for *any* renaming — including order-reversing
/// ones).
#[hegel::test]
fn presentation_row_names_never_reach_the_hash(tc: TestCase) {
    let model = tc.draw(arb_model());
    let doc = serialize(&model);
    let renamed = rename_presentation_rows(&doc);
    let original = parse(&doc).expect("canonical bytes parse");
    let re_presented = parse(&renamed).expect("renamed document parses");
    assert_eq!(
        canonical_bytes(&original),
        canonical_bytes(&re_presented),
        "same structure, equal canonical bytes"
    );
    assert_eq!(
        model_hash(&original),
        model_hash(&re_presented),
        "equal models hash equal under row renames"
    );
}

/// The catherine-1ml ratchet over the automorphism-rich shapes. Its own
/// 32-case envelope keeps the near-exhaustive small sweep (four shapes ×
/// sizes 2..=4 × rotations) inside the suite's wall-clock budget without
/// shrinking what the assumption needs.
#[hegel::test(test_cases = 32)]
fn symmetric_ratchet_bounded_sizes(tc: TestCase) {
    let shape = tc.draw(gs::integers::<u8>().min_value(0).max_value(3));
    let n = tc.draw(gs::integers::<usize>().min_value(2).max_value(4));
    let rotation = tc.draw(gs::integers::<usize>().min_value(1).max_value(4));
    let (model, re_presented) = symmetric_hash_ratchet(shape, n, rotation);
    assert_eq!(
        canonical_bytes(&model),
        canonical_bytes(&re_presented),
        "row presentation order must not reach the canonical form"
    );
    assert_eq!(
        model_hash(&model),
        model_hash(&re_presented),
        "symmetric structures hash equal under row permutation"
    );
}

/// The ratchet body: serialize, rotate every row list (pure
/// presentation-order permutation), and re-parse, handing back the model
/// and its re-presentation for the invariant assertions.
fn symmetric_hash_ratchet(shape: u8, n: usize, rotation: usize) -> (Model, Model) {
    let model = symmetric_model(shape, n);
    let doc = serialize(&model);
    let mut value: Value = serde_json::from_str(&doc).expect("serialized model is JSON");
    rotate_row_lists(&mut value, rotation);
    let rotated = serde_json::to_string(&value).expect("rotated document serializes");
    let re_presented = parse(&rotated).expect("rotated document re-parses");
    (model, re_presented)
}

/// An automorphism-rich model of the drawn shape over `n` rows per
/// object — self-loop rotation cycles, identity-pair involutions,
/// twin-row collapse into two shared targets, and the one-target
/// collapse (the shapes where equal refined partitions are common and
/// the pruning assumption is load-bearing). Dispatch is a builder
/// table, not a match: straight-line builders keep every function under
/// the gate's caps.
fn symmetric_model(shape: u8, n: usize) -> Model {
    let builders: [fn(usize) -> Model; 4] =
        [cycle_model, involution_model, twins_model, collapse_model];
    builders[(shape % 4) as usize](n)
}

/// Rotation cycle: f(i) = (i+1) mod n — cyclic symmetry group.
fn cycle_model(n: usize) -> Model {
    Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["a"]),
            arrows: self_loop("f", "a"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", n, |i| Some(RowId((i + 1) % n))),
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    }
}

/// Identity involution across a two-object pair: f: p→q, g: q→p, both
/// identity maps — the mirror symmetry of a degenerate complete
/// bipartite pair.
fn involution_model(n: usize) -> Model {
    let identity = |n: usize| (0..n).map(|i| Some(RowId(i))).collect();
    Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["p", "q"]),
            arrows: cross_maps("f", "p", "q", "g"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("p", n), ("q", n)]),
            values: {
                let mut values = std::collections::BTreeMap::new();
                values.insert(Ident("f".to_string()), identity(n));
                values.insert(Ident("g".to_string()), identity(n));
                values
            },
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    }
}

/// Twin rows: f(i) = i/2 — every pair of adjacent rows shares a target,
/// the duplicated-component shape the pruning treats as one partition
/// class.
fn twins_model(n: usize) -> Model {
    Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["a"]),
            arrows: self_loop("f", "a"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", n, |i| Some(RowId(i / 2))),
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    }
}

/// Full collapse: every row maps to row 0 — the one-row
/// collapse-satisfiable shape.
fn collapse_model(n: usize) -> Model {
    Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["a"]),
            arrows: self_loop("f", "a"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", n, |_| Some(RowId(0))),
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    }
}

/// Rotate every `rows` array of the document by `k` — a pure
/// presentation-order permutation (tables reference row *names*, so the
/// model is unchanged; only the serialized order moves).
fn rotate_row_lists(value: &mut Value, k: usize) {
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
