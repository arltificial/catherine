// Adapted from the specodelic project (repository `charly/specodelic`,
// files `src/acset/instance.rs` — byte-stable canonical serialization of
// an interned instance, and `src/acset/schema.rs` — canonical sorted
// enumeration), licensed under Apache-2.0. The project ships no NOTICE
// file; this header records the attribution required by Apache-2.0 §4(c)
// and preserved by hard constraint 4 of openspec change
// `add-model-document-kernel`.
//
//! Purpose: byte-stable canonical serialization of a parsed model — the
//! artifact P6 ledger content-addressing hashes.
//!
//! Responsibilities: emit a parsed model as a valid `.ct.json` v1
//! document whose bytes depend only on the model's structure — input key
//! order, presentation row names, and equation order never reach them
//! (`parse(serialize(m)) == m` for every parsed model); and provide the
//! wild-normative canonical JSON writer over arbitrary JSON values.
//!
//! Rationale: acset-core's `build_deterministic` discipline — every
//! section is emitted in sorted order so input order cannot reach the
//! bytes — plus wild `formats.rs`'s canonical writer rules: recursively
//! sorted object keys by Unicode scalar value, no whitespace, base-ten
//! integers, lowercase `\u00xx` control escapes, unescaped solidus,
//! literal UTF-8 otherwise. The writer is byte-for-byte normative (its
//! own tests pin wild's exact examples); equality of canonical bytes is
//! the hash contract, asserted structurally everywhere else.

use crate::model::{Equation, Ident, Model, RowId};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Canonical bytes for hashing: the model re-labeled with canonical row
/// ids — dense per-object ids derived from the model's *structure*, never
/// from presentation names — and serialized. Two documents that express
/// the same model under different row names produce byte-identical output
/// here (design Decision 4's strong reading, pinned by the roundtrip
/// property).
pub fn canonical_bytes(model: &Model) -> String {
    serialize(&canonical_relabel(model))
}

/// One row's address during canonical labeling: object name + dense id.
/// Object and arrow names are declarations, not presentation — only rows
/// are renamed — so keys built from them are rename-free.
type RowKey = (String, usize);

/// Sort markers larger than any real class rank: a cell with no target
/// (dangling) and a row individualized during canonical labeling.
const DANGLING: usize = usize::MAX - 1;
const INDIVIDUALIZED: usize = usize::MAX;

/// Re-label the model's rows with canonical ids: iterative refinement
/// over the whole structure (class of a row = its previous class, the
/// classes of its outgoing cells, and the sorted classes of its incoming
/// preimages, per named arrow), then — while any class has more than one
/// row — individualize each member of the first tied class and keep the
/// branch whose serialization is lexicographically smallest. Every step
/// consults structure only, so the result is invariant under any
/// renaming of rows; the exhaustive tie-break keeps it exact.
fn canonical_relabel(model: &Model) -> Model {
    let mut cls: BTreeMap<RowKey, usize> = BTreeMap::new();
    for (object, rows) in &model.instance.rows {
        for r in 0..rows.len() {
            cls.insert((object.0.clone(), r), 0);
        }
    }
    solve(model, &mut cls)
}

/// Refine to fixpoint, then individualize tied classes until every row
/// has its own id. Branches whose post-seed refinement profile matches an
/// already-explored branch are treated as automorphism-equivalent and
/// pruned — this keeps heavily self-symmetric structures linear instead
/// of factorial (the assumption's bound is documented in the crate
/// README).
fn solve(model: &Model, cls: &mut BTreeMap<RowKey, usize>) -> Model {
    while refine(model, cls) {}
    let Some((object, members)) = first_tied_class(model, cls) else {
        return apply_labeling(model, cls);
    };
    let mut best: Option<(String, Model)> = None;
    let mut explored: Vec<Vec<(String, Vec<usize>)>> = Vec::new();
    for r in members {
        let mut seeded = cls.clone();
        seeded.insert((object.clone(), r), INDIVIDUALIZED);
        while refine(model, &mut seeded) {}
        let profile = partition_profile(model, &seeded);
        if explored.contains(&profile) {
            continue;
        }
        explored.push(profile);
        let candidate = solve(model, &mut seeded);
        let bytes = serialize(&candidate);
        if best.as_ref().is_none_or(|(b, _)| bytes < *b) {
            best = Some((bytes, candidate));
        }
    }
    best.expect("the first branch always runs").1
}

/// The ρ-invariant shape of a refined partition: per object (sorted), the
/// sorted list of class sizes. Isomorphic seedings share a profile.
fn partition_profile(model: &Model, cls: &BTreeMap<RowKey, usize>) -> Vec<(String, Vec<usize>)> {
    let mut profile = Vec::new();
    for (object, rows) in &model.instance.rows {
        let mut sizes: BTreeMap<usize, usize> = BTreeMap::new();
        for r in 0..rows.len() {
            *sizes.entry(cls[&(object.0.clone(), r)]).or_default() += 1;
        }
        profile.push((object.0.clone(), sizes.into_values().collect()));
    }
    profile
}

/// The first tied class in sorted object order: `(object, members)` with
/// members in ascending row-id order.
fn first_tied_class(model: &Model, cls: &BTreeMap<RowKey, usize>) -> Option<(String, Vec<usize>)> {
    for (object, rows) in &model.instance.rows {
        let mut sizes: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for r in 0..rows.len() {
            let class = cls[&(object.0.clone(), r)];
            sizes.entry(class).or_default().push(r);
        }
        for members in sizes.values() {
            if members.len() > 1 {
                return Some((object.0.clone(), members.clone()));
            }
        }
    }
    None
}

/// One refinement round over all objects at once (cross-object cells
/// reference each other's classes). Returns whether any partition
/// changed; partitions only refine, so this terminates.
fn refine(model: &Model, cls: &mut BTreeMap<RowKey, usize>) -> bool {
    let mut next = cls.clone();
    for (object, rows) in &model.instance.rows {
        let n = rows.len();
        let mut sigs: Vec<Vec<usize>> = Vec::with_capacity(n);
        for r in 0..n {
            let mut sig = vec![cls[&(object.0.clone(), r)]];
            // Outgoing cells, per named arrow in sorted order.
            for (name, arrow) in &model.schema.arrows {
                if arrow.src.0 == object.0 {
                    let cell = model
                        .instance
                        .values
                        .get(name)
                        .and_then(|v| v.get(r))
                        .copied()
                        .flatten();
                    sig.push(match cell {
                        Some(RowId(t)) => cls[&(arrow.tgt.0.clone(), t)],
                        None => DANGLING,
                    });
                }
            }
            // Incoming preimages, per named arrow in sorted order: the
            // sorted multiset of source-row classes.
            for (name, arrow) in &model.schema.arrows {
                if arrow.tgt.0 == object.0 {
                    let mut preimages: Vec<usize> = Vec::new();
                    if let Some(cells) = model.instance.values.get(name) {
                        for (s, cell) in cells.iter().enumerate() {
                            if *cell == Some(RowId(r)) {
                                preimages.push(cls[&(arrow.src.0.clone(), s)]);
                            }
                        }
                    }
                    preimages.sort_unstable();
                    sig.push(preimages.len());
                    sig.extend(preimages);
                }
            }
            sigs.push(sig);
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|a, b| sigs[*a].cmp(&sigs[*b]));
        let mut ranks = vec![0usize; n];
        let mut rank = 0;
        for i in 1..n {
            if sigs[order[i]] != sigs[order[i - 1]] {
                rank += 1;
            }
            ranks[order[i]] = rank;
        }
        for (r, &rank) in ranks.iter().enumerate() {
            next.insert((object.0.clone(), r), rank);
        }
    }
    let changed = *cls != next;
    *cls = next;
    changed
}

/// Apply a fully-refined (all-singleton) labeling: rows become the dense
/// identity vector per object and every value cell is remapped through
/// both endpoints' classes.
fn apply_labeling(model: &Model, cls: &BTreeMap<RowKey, usize>) -> Model {
    let mut out = model.clone();
    for rows in out.instance.rows.values_mut() {
        let n = rows.len();
        *rows = (0..n).map(RowId).collect();
    }
    for (arrow, cells) in out.instance.values.iter_mut() {
        let spec = &model.schema.arrows[arrow];
        let (src, tgt) = (spec.src.0.clone(), spec.tgt.0.clone());
        let n = out.instance.rows[&Ident(src.clone())].len();
        let mut relabeled = vec![None; n];
        for (r, cell) in cells.iter().enumerate() {
            let new_r = cls[&(src.clone(), r)];
            relabeled[new_r] = cell.map(|RowId(t)| RowId(cls[&(tgt.clone(), t)]));
        }
        *cells = relabeled;
    }
    out
}

/// Canonical serialization: a valid `.ct.json` v1 document (dense
/// per-object row names `_<id>` in id order) that re-parses to the same
/// model. This is the byte sequence the model hash is taken over.
pub fn serialize(model: &Model) -> String {
    write_value(&canonical_value(model))
}

/// The model as a JSON value in canonical shape: sorted enumerations,
/// dense per-object row names, total per-arrow tables.
pub fn canonical_value(model: &Model) -> Value {
    // Defensive canonical order: the parser already stores equations
    // sorted, but constructed values must serialize the same way
    // (specodelic's `schema_export` discipline).
    let mut equations: Vec<&Equation> = model.schema.equations.iter().collect();
    equations.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));

    let mut rows = Map::new();
    for (object, ids) in &model.instance.rows {
        let names: Vec<Value> = (0..ids.len()).map(|i| string(&row_name(i))).collect();
        rows.insert(object.0.clone(), Value::Array(names));
    }
    let mut tables = Map::new();
    for (arrow, cells) in &model.instance.values {
        let mut emitted = Map::new();
        for (i, cell) in cells.iter().enumerate() {
            let value = match cell {
                Some(RowId(j)) => string(&row_name(*j)),
                None => Value::Null,
            };
            emitted.insert(row_name(i), value);
        }
        tables.insert(arrow.0.clone(), Value::Object(emitted));
    }

    let arrows: Vec<Value> = model
        .schema
        .arrows
        .iter()
        .map(|(name, arrow)| {
            obj([
                ("name", string(&name.0)),
                ("src", string(&arrow.src.0)),
                ("tgt", string(&arrow.tgt.0)),
            ])
        })
        .collect();
    let equation_values: Vec<Value> = equations
        .iter()
        .map(|e| {
            obj([
                ("lhs", path(&e.lhs)),
                ("rationale", string(&e.rationale)),
                ("rhs", path(&e.rhs)),
                ("status", string(e.status.as_str())),
            ])
        })
        .collect();
    obj([
        (
            "format_version",
            string(&format!(
                "{}.{}",
                model.format_version.major, model.format_version.minor
            )),
        ),
        (
            "instance",
            obj([
                ("rows", Value::Object(rows)),
                ("tables", Value::Object(tables)),
            ]),
        ),
        (
            "schema",
            obj([
                (
                    "objects",
                    Value::Array(model.schema.objects.keys().map(|o| string(&o.0)).collect()),
                ),
                ("arrows", Value::Array(arrows)),
                ("equations", Value::Array(equation_values)),
            ]),
        ),
    ])
}

/// Canonical bytes of an arbitrary JSON value per wild's normative rules.
/// serde_json's `Map` is a `BTreeMap`, so object iteration *is* the
/// recursively-sorted key order wild's `keys.sort()` produces.
pub fn write_value(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => write_escaped(s, out),
        Value::Array(items) => {
            out.push('[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(entries) => {
            out.push('{');
            for (idx, (key, value)) in entries.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_escaped(key, out);
                out.push(':');
                write_canonical(value, out);
            }
            out.push('}');
        }
    }
}

fn write_escaped(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn obj<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn string(s: &str) -> Value {
    Value::String(s.to_string())
}

fn path(p: &crate::model::Path) -> Value {
    Value::Array(p.0.iter().map(|i| string(&i.0)).collect())
}

/// The presentation spelling of a dense row id inside serialized bytes:
/// `_`-prefixed so it is a valid ident (leading digits are
/// `invalid_ident`).
fn row_name(i: usize) -> String {
    format!("_{i}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::model_hash;
    use crate::parse::parse;

    /// Ported byte-for-byte from wild `src/formats.rs`'s normative test —
    /// these bytes ARE the contract for the writer.
    #[test]
    fn canonical_rules_match_the_wild_format_examples() {
        let cases: Vec<(&str, &str)> = vec![
            (r#"{"b":2,"a":1}"#, r#"{"a":1,"b":2}"#),
            (r#"{ "text": "\u0061" }"#, r#"{"text":"a"}"#),
            (r#"{"text":"\n"}"#, r#"{"text":"\u000a"}"#),
        ];
        for (input, expected) in cases {
            let value: Value = serde_json::from_str(input).expect("parses");
            assert_eq!(write_value(&value), expected);
        }
    }

    /// Two input orderings of the same model — shuffled keys, reordered
    /// declarations, reordered equations — serialize byte-identically
    /// (`build_deterministic`), and serialization is a parse fixed point.
    #[test]
    fn serialization_is_byte_stable_across_input_order() {
        let first = r#"{
          "format_version": "1.0",
          "schema": {
            "equations": [
              {"lhs": ["h"], "rhs": ["h", "h"], "rationale": "idempotent", "status": "accepted"},
              {"lhs": ["f"], "rhs": ["f"], "rationale": "loop", "status": "proposed"}
            ],
            "arrows": [
              {"name": "h", "src": "B", "tgt": "B"},
              {"name": "f", "src": "A", "tgt": "B"}
            ],
            "objects": ["B", "A"]
          },
          "instance": {
            "rows": {"B": ["_0"], "A": ["_0"]},
            "tables": {"h": {"_0": null}, "f": {"_0": "_0"}}
          }
        }"#;
        let second = r#"{
          "instance": {"tables": {"f": {"_0": "_0"}, "h": {}}, "rows": {"A": ["_0"], "B": ["_0"]}},
          "schema": {
            "objects": ["A", "B"],
            "arrows": [
              {"tgt": "B", "src": "A", "name": "f"},
              {"tgt": "B", "src": "B", "name": "h"}
            ],
            "equations": [
              {"status": "proposed", "rationale": "loop", "rhs": ["f"], "lhs": ["f"]},
              {"status": "accepted", "rationale": "idempotent", "rhs": ["h", "h"], "lhs": ["h"]}
            ]
          },
          "format_version": "1.0"
        }"#;
        let a = parse(first).expect("first ordering parses");
        let b = parse(second).expect("second ordering parses");
        prop_eq_model(&a, &b);
        let bytes = serialize(&a);
        assert_eq!(bytes, serialize(&b), "equal models, equal bytes");
        assert_eq!(
            parse(&bytes).expect("canonical bytes parse"),
            a,
            "serialization is a parse fixed point"
        );
    }

    /// Canonical row ids are structure-derived: an order-reversing
    /// renaming (which sorted-name ids could never survive) yields equal
    /// canonical bytes and hash, while a structural difference (identity
    /// vs constant map) does not.
    #[test]
    fn canonical_bytes_are_rename_invariant() {
        let original = r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A"],
            "arrows": [{"name": "s", "src": "A", "tgt": "A"}],
            "equations": []
          },
          "instance": {"rows": {"A": ["_0", "_1"]}, "tables": {"s": {"_0": "_0", "_1": "_0"}}}
        }"#;
        let renamed = r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A"],
            "arrows": [{"name": "s", "src": "A", "tgt": "A"}],
            "equations": []
          },
          "instance": {"rows": {"A": ["zz1", "zz0"]}, "tables": {"s": {"zz1": "zz1", "zz0": "zz1"}}}
        }"#;
        let a = parse(original).expect("parses");
        let b = parse(renamed).expect("parses");
        assert_eq!(
            canonical_bytes(&a),
            canonical_bytes(&b),
            "renaming is invisible"
        );
        assert_eq!(
            model_hash(&a),
            model_hash(&b),
            "renaming is invisible to the hash"
        );
        let identity = r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A"],
            "arrows": [{"name": "s", "src": "A", "tgt": "A"}],
            "equations": []
          },
          "instance": {"rows": {"A": ["_0", "_1"]}, "tables": {"s": {"_0": "_0", "_1": "_1"}}}
        }"#;
        let c = parse(identity).expect("parses");
        assert_ne!(
            model_hash(&a),
            model_hash(&c),
            "constant map vs identity map are different structures"
        );
    }

    fn prop_eq_model(a: &Model, b: &Model) {
        assert_eq!(a, b, "models equal");
    }
}
