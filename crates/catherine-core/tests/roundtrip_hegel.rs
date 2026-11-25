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
#[path = "roundtrip/rename.rs"]
mod rename;

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::hash::model_hash;
use catherine_core::model::{Ident, Instance, Model, RowId, Schema, Version};
use catherine_core::parse::parse;
use hegel::generators as gs;
use hegel::TestCase;
use model_fixture::{arb_model, cross_maps, dense_rows, object, self_loop, table};
use rename::{rename_presentation_rows, rotate_row_lists};
use serde_json::Value;
use std::collections::BTreeMap;

mod common;

use common::Insist;

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
    let original = parse(&doc).insist("canonical bytes parse");
    let re_presented = parse(&renamed).insist("renamed document parses");
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
    let (model, re_presented) = symmetric_hash_ratchet(shape, (n, rotation));
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
fn symmetric_hash_ratchet(shape: u8, drawn: (usize, usize)) -> (Model, Model) {
    let (n, rotation) = drawn;
    let model = symmetric_model(shape, n);
    let doc = serialize(&model);
    let mut value: Value = serde_json::from_str(&doc).insist("serialized model is JSON");
    rotate_row_lists(&mut value, rotation);
    let rotated = serde_json::to_string(&value).insist("rotated document serializes");
    let re_presented = parse(&rotated).insist("rotated document re-parses");
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
    let model = Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["a"]),
            arrows: self_loop("f", "a"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
            stored: BTreeMap::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", (n, |i| Some(RowId((i + 1) % n)))),
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    };
    assert_eq!(
        model.instance.rows[&Ident("a".to_string())].len(),
        n,
        "the cycle shape spans all n rows"
    );
    model
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
            arrows: cross_maps(("f", "g"), ("p", "q")),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
            stored: BTreeMap::new(),
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
    let model = Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["a"]),
            arrows: self_loop("f", "a"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
            stored: BTreeMap::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", (n, |i| Some(RowId(i / 2)))),
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    };
    assert!(
        model.instance.rows[&Ident("a".to_string())].len() == n,
        "the twin shape spans all n rows"
    );
    model
}

/// Full collapse: every row maps to row 0 — the one-row
/// collapse-satisfiable shape.
fn collapse_model(n: usize) -> Model {
    let model = Model {
        format_version: Version { major: 1, minor: 0 },
        schema: Schema {
            objects: object(&["a"]),
            arrows: self_loop("f", "a"),
            equations: vec![],
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
            stored: BTreeMap::new(),
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", (n, |_| Some(RowId(0)))),
        },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    };
    assert_ne!(
        model.instance.rows[&Ident("a".to_string())].len(),
        0,
        "the collapse shape is never empty"
    );
    let flattened = model
        .instance
        .values
        .values()
        .flatten()
        .all(|c| *c == Some(RowId(0)));
    assert!(flattened, "every row collapses onto row 0");
    model
}
