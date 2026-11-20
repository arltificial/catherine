//! Purpose: the Kani bounded-model-checking harnesses over the shipped
//! kernel (`catherine-core`) — panic/overflow-freedom over the
//! post-parse execution path (engine build, path composition inside the
//! `check` walk, equation/key/coprod/pullback families) and witness
//! soundness on emitted findings (re-evaluation half in the `soundness`
//! submodule) — openspec change `add-kani-verification-lane`, beads
//! `catherine-6g2` (the interning that made the lane runnable).
//!
//! Responsibilities: harnesses are PARTITIONED PER CHECK FAMILY, each
//! with its own minimal schema and stated bounds (design Decision 4).
//! Synthesis is nondeterministic `kani::any()` scalars composed into
//! dense id tables by deterministic insertion loops (Decision 2: never
//! the map-bearing domain types themselves, never serde strings — and,
//! since catherine-amh, never any `BTreeMap` op: container modeling is
//! the measured wall). Bounds are non-vacuous per family: each harness's
//! space admits a document exercising its family's failure shapes; the
//! provocation pass (task 3) demonstrates each harness can fail.
//!
//! Rationale: the checker sets `cfg(kani)`; this module compiles only
//! under verification. The sortedness obligation asserts the canonical
//! taxonomy order over the engine's int sort keys — the exact comparison
//! prefix of `Finding::cmp` (id order is name order by construction),
//! without charging a `&str` comparison to the solver (the memcmp model
//! is measured fatal inside sort comparators). Kani's UNDETERMINED and
//! any counterexample follow the counterexample protocol (design
//! Decision 5): attribution recorded, obligations never weakened.

#![cfg(kani)]

use catherine_engine::engine::{
    engine_build, engine_families, CompactFinding, EngineSchema, EngineTyped, SortKey,
};

mod shapes;
mod soundness;

/// Panic/overflow-freedom of the engine build alone over the full arrow
/// space, including dangling and typing-violation branches (build is
/// infallible by contract: every value lands somewhere, never a hard
/// failure).
#[kani::proof]
fn build_panic_freedom() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let _ = typed.tables.len();
    // Drop glue and deallocation paths dominate the solver's symbolic
    // execution (catherine-6g2, measured: 154 s symex, 1.17 M steps,
    // nearly all drop/dealloc); the claim is the walk, and none of
    // these types carries a panicking drop, so the harness forgets them.
    std::mem::forget((schema, instance, typed));
}

/// The nondeterministic `check` + diff-stable-order obligation over the
/// whole walk: all four families declared on one minimal presentation,
/// the emitted findings asserted totally ordered — in the int keys that
/// `Finding::cmp` compares (kind, variant, witness ids, label). The
/// all-families space emits up to 45 findings (concrete bound:
/// `arrow_src.len() * max_rows + max_rows^2 * (eqs + keys + cops + 3 *
/// pbs) + ...`), so the sort loops need their own unwind — the declared
/// per-harness bound (design Decision 4); every other loop here stays
/// within the lane's default 16.
#[kani::proof]
#[kani::unwind(48)]
fn check_sortedness_panic_freedom() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_families(&schema, &typed);
    for window in findings.windows(2) {
        assert!(window[0].0 <= window[1].0);
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the equation family over its minimal space
/// (parallel pair `[e1] = [e2]`; both sides must resolve or the row
/// emits nothing — recorded by dangling, never crashed).
#[kani::proof]
fn check_equation_panic_freedom() {
    let schema = shapes::schema_equation();
    let typed = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, typed, findings));
}

/// Panic/overflow-freedom of the key family over its minimal space (one
/// two-arrow key over a two-row object; unresolved cells exclude their
/// row from the collision scan — never double-reported).
#[kani::proof]
fn check_key_panic_freedom() {
    let schema = shapes::schema_key();
    let typed = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, typed, findings));
}

/// Panic/overflow-freedom of the coproduct family over its minimal space
/// (two injections into a two-row target; coverage and disjointness
/// decided only on resolvable cells).
#[kani::proof]
fn check_coproduct_panic_freedom() {
    let schema = shapes::schema_coproduct();
    let typed = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, typed, findings));
}

/// Panic/overflow-freedom of the pullback family over its minimal space
/// (apex with two legs, cospan over a two-row target so commutes can
/// disagree; dangling legs suspend a row's verdicts, dangling cospan
/// cells shrink the decidable fiber).
#[kani::proof]
fn check_pullback_panic_freedom() {
    let schema = shapes::schema_pullback();
    let typed = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, typed, findings));
}

/// Witness soundness, equation family: every emitted violation's witness
/// re-evaluated independently (soundness submodule) disagrees exactly as
/// claimed.
#[kani::proof]
fn soundness_equation() {
    let (schema, typed, findings) = family_under_test(shapes::schema_equation());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, typed, findings));
}

/// Witness soundness, key family: every emitted collision's witness
/// re-evaluated independently finds equal, fully resolved tuples on
/// ascending rows.
#[kani::proof]
fn soundness_key() {
    let (schema, typed, findings) = family_under_test(shapes::schema_key());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, typed, findings));
}

/// Witness soundness, coproduct family: coverage witnesses name rows in
/// no injection image; disjointness witnesses name colliding injections
/// that really cover their row.
#[kani::proof]
fn soundness_coproduct() {
    let (schema, typed, findings) = family_under_test(shapes::schema_coproduct());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, typed, findings));
}

/// Witness soundness, pullback family: commutes witnesses re-derive both
/// compositions and the disagreement; covers witnesses re-derive the
/// uncovered fiber pair; unique witnesses re-derive the shared
/// projections.
#[kani::proof]
fn soundness_pullback() {
    let (schema, typed, findings) = family_under_test(shapes::schema_pullback());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, typed, findings));
}

/// Witness soundness, dangling family: every dangling witness names a
/// cell that really holds no row.
#[kani::proof]
fn soundness_dangling() {
    let (schema, typed, findings) = family_under_test(shapes::schema_equation());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, typed, findings));
}

/// Build the typed tables for a family's shape.
fn typed_for(schema: &EngineSchema) -> EngineTyped {
    let instance = shapes::instance_for(schema);
    engine_build(schema, &instance)
}

/// The schema, its tables, and the keyed findings — one soundness
/// sample space per family.
fn family_under_test(
    schema: EngineSchema,
) -> (
    EngineSchema,
    EngineTyped,
    Vec<(
        SortKey,
        catherine_engine::findings::EpistemicLabel,
        CompactFinding,
    )>,
) {
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_families(&schema, &typed);
    (schema, typed, findings)
}

#[kani::proof]
fn probe_build_only() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    assert_eq!(typed.tables.len(), 7);
}
