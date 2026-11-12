//! Purpose: the catherine-1ml ratchet — the WL-pruning assumption
//! behind the canonicalizer`s refined-partition branch skip
//! (`canonical/label.rs::solve`), now guarded by executable evidence:
//! an automorphism-rich model strategy (rotation cycles, identity-pair
//! involutions, twin rows, full collapse) and the property that
//! `model_hash` + `canonical_bytes` are invariant under row-presentation
//! permutation — the shapes where equal refined partitions recur and
//! the pruning is load-bearing.
//!
//! Budget: its own `proptest!` block with an explicit 32-case envelope
//! over sizes 2..=4 — the automorphism-rich shapes are the
//! canonicalizer`s combinatorial worst case, and the envelope is a
//! documented trade, not a silent shrink (larger symmetric instances
//! are deliberately NOT sampled; a CFI-family generator is a candidate
//! follow-up ticket).

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::hash::model_hash;
use catherine_core::model::{Arrow, Ident, Instance, Model, RowId, Schema, Version};
use catherine_core::parse::parse;
use proptest::prelude::*;
use serde_json::Value;

// catherine-1ml runs in its own block with an explicit budget: the
// automorphism-rich shapes are the canonicalizer's worst case (the
// pruned search space still grows combinatorially with row count), so
// the ratchet samples a near-exhaustive small envelope — 32 cases over
// all four shapes × sizes 2..=4 × rotations — which is what the
// assumption ratchet needs without dominating the suite's wall-clock
// budget. Larger symmetric instances are deliberately NOT sampled here;
// the budget is a documented trade, not a silent shrink.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn symmetric_ratchet_bounded_sizes(
        shape in 0u8..4,
        n in 2usize..=4,
        rotation in 1usize..=4usize,
    ) {
        symmetric_hash_ratchet(shape, n, rotation)?;
    }
}

/// The ratchet body shared by both budgets: serialize, rotate every row
/// list (pure presentation-order permutation), re-parse, and require
/// equal canonical bytes and equal hash.
fn symmetric_hash_ratchet(shape: u8, n: usize, rotation: usize) -> Result<(), TestCaseError> {
    let model = symmetric_model(shape, n);
    let doc = serialize(&model);
    let mut value: Value = serde_json::from_str(&doc).expect("serialized model is JSON");
    rotate_row_lists(&mut value, rotation);
    let rotated = serde_json::to_string(&value).expect("rotated document serializes");
    let re_presented = parse(&rotated)
        .map_err(|err| TestCaseError::fail(format!("rotated document re-parse failed: {err:?}")))?;
    prop_assert_eq!(
        canonical_bytes(&model),
        canonical_bytes(&re_presented),
        "row presentation order must not reach the canonical form"
    );
    prop_assert_eq!(
        model_hash(&model),
        model_hash(&re_presented),
        "symmetric structures hash equal under row permutation"
    );
    Ok(())
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
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", n, |i| Some(RowId((i + 1) % n))),
        },
        schema_map: None,
        rule_presentation: None,
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
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", n, |i| Some(RowId(i / 2))),
        },
        schema_map: None,
        rule_presentation: None,
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
        },
        instance: Instance {
            rows: dense_rows(&[("a", n)]),
            values: table("f", n, |_| Some(RowId(0))),
        },
        schema_map: None,
        rule_presentation: None,
    }
}

/// One declared object.
fn object(names: &[&str]) -> std::collections::BTreeMap<Ident, ()> {
    names
        .iter()
        .map(|n| (Ident((*n).to_string()), ()))
        .collect()
}

/// One self-loop arrow on `obj`.
fn self_loop(name: &str, obj: &str) -> std::collections::BTreeMap<Ident, Arrow> {
    let mut arrows = std::collections::BTreeMap::new();
    arrows.insert(
        Ident(name.to_string()),
        Arrow {
            src: Ident(obj.to_string()),
            tgt: Ident(obj.to_string()),
        },
    );
    arrows
}

/// A pair of opposite arrows `f: p↔q` (`f` and `g` both declared).
fn cross_maps(f: &str, p: &str, q: &str, g: &str) -> std::collections::BTreeMap<Ident, Arrow> {
    let mut arrows = std::collections::BTreeMap::new();
    arrows.insert(
        Ident(f.to_string()),
        Arrow {
            src: Ident(p.to_string()),
            tgt: Ident(q.to_string()),
        },
    );
    arrows.insert(
        Ident(g.to_string()),
        Arrow {
            src: Ident(q.to_string()),
            tgt: Ident(p.to_string()),
        },
    );
    arrows
}

/// Dense row-id spaces `0..count` per named object.
fn dense_rows(spec: &[(&str, usize)]) -> std::collections::BTreeMap<Ident, Vec<RowId>> {
    spec.iter()
        .map(|(name, count)| (Ident((*name).to_string()), (0..*count).map(RowId).collect()))
        .collect()
}

/// One total arrow table: `values[name] = vec![target(i); n]`, parallel
/// to the source object's rows.
fn table(
    name: &str,
    n: usize,
    target: impl Fn(usize) -> Option<RowId>,
) -> std::collections::BTreeMap<Ident, Vec<Option<RowId>>> {
    let mut values = std::collections::BTreeMap::new();
    values.insert(
        Ident(name.to_string()),
        (0..n).map(target).collect::<Vec<_>>(),
    );
    values
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
