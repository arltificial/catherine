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

mod label;
mod value;
mod writer;

pub use value::{canonical_value, serialize};
pub use writer::write_value;

use crate::model::Model;
use label::canonical_relabel;

/// Canonical bytes for hashing: the model re-labeled with canonical row
/// ids — dense per-object ids derived from the model's *structure*, never
/// from presentation names — and serialized. Two documents that express
/// the same model under different row names produce byte-identical output
/// here (design Decision 4's strong reading, pinned by the roundtrip
/// property).
pub fn canonical_bytes(model: &Model) -> String {
    serialize(&canonical_relabel(model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::model_hash;
    use crate::parse::parse;
    use serde_json::Value;

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
        let a = parse(first_ordering()).expect("first ordering parses");
        let b = parse(second_ordering()).expect("second ordering parses");
        prop_eq_model(&a, &b);
        let bytes = serialize(&a);
        assert_eq!(bytes, serialize(&b), "equal models, equal bytes");
        assert_eq!(
            parse(&bytes).expect("canonical bytes parse"),
            a,
            "serialization is a parse fixed point"
        );
    }

    /// The first input ordering: canonical declaration order.
    fn first_ordering() -> &'static str {
        r#"{
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
        }"#
    }

    /// The second input ordering: shuffled keys, reordered declarations,
    /// reordered equations — the same model under different presentation.
    fn second_ordering() -> &'static str {
        r#"{
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
        }"#
    }

    /// Canonical row ids are structure-derived: an order-reversing
    /// renaming (which sorted-name ids could never survive) yields equal
    /// canonical bytes and hash, while a structural difference (identity
    /// vs constant map) does not.
    #[test]
    fn canonical_bytes_are_rename_invariant() {
        let a = parse(constant_map()).expect("parses");
        let b = parse(reversed_map()).expect("parses");
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
        let c = parse(identity_map()).expect("parses");
        assert_ne!(
            model_hash(&a),
            model_hash(&c),
            "constant map vs identity map are different structures"
        );
    }

    /// One object `A`, one self-arrow `s`, identity mapping.
    fn identity_map() -> &'static str {
        r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A"],
            "arrows": [{"name": "s", "src": "A", "tgt": "A"}],
            "equations": []
          },
          "instance": {"rows": {"A": ["_0", "_1"]}, "tables": {"s": {"_0": "_0", "_1": "_1"}}}
        }"#
    }

    /// The same structure under an order-reversing renaming.
    fn reversed_map() -> &'static str {
        r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A"],
            "arrows": [{"name": "s", "src": "A", "tgt": "A"}],
            "equations": []
          },
          "instance": {"rows": {"A": ["zz1", "zz0"]}, "tables": {"s": {"zz1": "zz1", "zz0": "zz1"}}}
        }"#
    }

    /// A different structure: the constant map.
    fn constant_map() -> &'static str {
        r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A"],
            "arrows": [{"name": "s", "src": "A", "tgt": "A"}],
            "equations": []
          },
          "instance": {"rows": {"A": ["_0", "_1"]}, "tables": {"s": {"_0": "_0", "_1": "_0"}}}
        }"#
    }

    fn prop_eq_model(a: &Model, b: &Model) {
        assert_eq!(a, b, "models equal");
    }
}
