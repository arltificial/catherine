//! Purpose: Kani bounded-model-checking harnesses over the shipped
//! kernel (`catherine-core`) — panic/overflow-freedom over the
//! post-parse execution path (build, walk composition, four families)
//! and witness soundness on emitted findings (`soundness` submodule);
//! openspec `add-kani-verification-lane`, beads `catherine-6g2`.
//!
//! Responsibilities: harnesses are PARTITIONED PER CHECK FAMILY
//! (Decision 4), each with its own minimal schema and stated bounds;
//! synthesis is nondeterministic `kani::any()` scalars composed into
//! dense id tables by deterministic insertion loops (Decision 2: no
//! map-bearing domain types, no serde strings, no `BTreeMap` ops —
//! container modeling is the measured wall); bounds are non-vacuous,
//! and the provocation pass demonstrates each can fail.
//!
//! Rationale: the checker sets `cfg(kani)`, so this module compiles only
//! under verification; the sortedness obligation asserts the canonical
//! order over int sort keys — the comparison prefix of `Finding::cmp`
//! — with no `&str` charged to the solver (memcmp is measured fatal
//! inside sort comparators); UNDETERMINED and counterexamples follow
//! Decision 5: attribution recorded, obligations never weakened.

#![cfg(kani)]

use catherine_engine::engine::{
    engine_build, engine_coproduct_findings, engine_dangling_findings, engine_equation_findings,
    engine_families, engine_key_findings, engine_pullback_findings, CompactFinding,
    CompactPullback, CompactViolation, EngineInstance, EngineSchema, EngineTyped,
};

mod proofs;
mod shapes;
mod soundness;

/// Panic/overflow-freedom of the engine build alone (infallible by
/// contract: every value lands somewhere).
#[kani::proof]
fn build_panic_freedom() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let _ = typed.tables.len();
    // Drop glue dominates the solver (measured 154 s of 300 s on build).
    std::mem::forget((schema, instance, typed));
}

/// The build obligation over one family's minimal shape (Decision 4).
macro_rules! build_panic_freedom_over {
    ($name:ident, $shape:ident) => {
        #[doc = "Panic/overflow-freedom of the engine build alone over one family's minimal space (the all-families claim partitioned per family, Decision 4)."]
        #[kani::proof]
        fn $name() {
            let schema = shapes::$shape();
            let instance = shapes::instance_for(&schema);
            let typed = engine_build(&schema, &instance);
            let _ = typed.tables.len();
            std::mem::forget((schema, instance, typed));
        }
    };
}

build_panic_freedom_over!(build_panic_freedom_equation, schema_equation);
build_panic_freedom_over!(build_panic_freedom_key, schema_key);
build_panic_freedom_over!(build_panic_freedom_coproduct, schema_coproduct);
build_panic_freedom_over!(build_panic_freedom_pullback, schema_pullback);

/// The nondeterministic `check` + diff-stable-order obligation: all
/// four families on one minimal presentation, emissions totally ordered
/// in the int keys `Finding::cmp` compares (sort loops declare 48).
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

/// Panic/overflow-freedom of the equation family's walk — the check
/// side's per-family partition (Decision 4, ledger `0007`): this
/// formula carries only the equation walk; `check_dangling_panic_freedom`
/// carries the dangling partition; together the whole-walk claim.
/// Unwind 8.
#[kani::proof]
#[kani::unwind(8)]
fn check_equation_panic_freedom() {
    let schema = shapes::schema_equation();
    let (instance, typed) = typed_for(&schema);
    let mut findings = engine_equation_findings(&schema, &typed);
    // The family's sort obligation lives here (the entry emits in walk
    // order); the soundness harness's formula carries no sort.
    findings.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    let mut i = 0;
    while i < findings.len() {
        assert!(matches!(
            &findings[i],
            Some((
                _,
                _,
                CompactFinding::Violation(CompactViolation::Equation { .. })
            ))
        ));
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the dangling family's walk (the equation shape's dangling partition).
#[kani::proof]
fn check_dangling_panic_freedom() {
    let schema = shapes::schema_equation();
    let (instance, typed) = typed_for(&schema);
    let findings = engine_dangling_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        assert!(matches!(
            &findings[i],
            Some((_, _, CompactFinding::Dangling { .. }))
        ));
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the key family (unresolved cells exclude
/// their row from the collision scan — never double-reported).
/// Partitioned per family; sort obligation here; unwind 9.
#[kani::proof]
#[kani::unwind(9)]
fn check_key_panic_freedom() {
    let schema = shapes::schema_key();
    let (instance, typed) = typed_for(&schema);
    let mut findings = engine_key_findings(&schema, &typed);
    findings.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    let mut i = 0;
    while i < findings.len() {
        assert!(matches!(
            &findings[i],
            Some((
                _,
                _,
                CompactFinding::Violation(CompactViolation::Key { .. })
            ))
        ));
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the coproduct family (two injections into
/// a two-row target, decided only on resolvable cells). Unwind 8.
#[kani::proof]
#[kani::unwind(8)]
fn check_coproduct_panic_freedom() {
    let schema = shapes::schema_coproduct();
    let (instance, typed) = typed_for(&schema);
    let mut findings = engine_coproduct_findings(&schema, &typed);
    findings.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    let mut i = 0;
    while i < findings.len() {
        assert!(matches!(
            &findings[i],
            Some((
                _,
                _,
                CompactFinding::Violation(CompactViolation::Coproduct { .. })
            ))
        ));
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the pullback family (apex with two legs,
/// cospan over a two-row target so commutes can disagree; dangling legs
/// suspend a row's verdicts, dangling cospan cells shrink the decidable
/// fiber). Partitioned per family, unwind 8; the sort is load-bearing.
#[kani::proof]
#[kani::unwind(8)]
fn check_pullback_panic_freedom() {
    let schema = shapes::schema_pullback();
    let (instance, typed) = typed_for(&schema);
    let mut findings = engine_pullback_findings(&schema, &typed);
    findings.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    let mut i = 0;
    while i < findings.len() {
        assert!(matches!(
            &findings[i],
            Some((
                _,
                _,
                CompactFinding::Violation(CompactViolation::Pullback(_))
            ))
        ));
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Build the typed tables for a family's shape.
fn typed_for(schema: &EngineSchema) -> (EngineInstance, EngineTyped) {
    let instance = shapes::instance_for(schema);
    let typed = engine_build(schema, &instance);
    (instance, typed)
}

#[kani::proof]
fn probe_build_only() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    assert_eq!(typed.tables.len(), 7);
}
