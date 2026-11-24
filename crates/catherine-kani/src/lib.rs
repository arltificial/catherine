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
    engine_build, engine_dangling_findings, engine_equation_findings, engine_families,
    CompactFinding, CompactViolation, EngineInstance, EngineSchema, EngineTyped, SortKey,
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

/// The build obligation over one family's minimal shape — the same
/// panic/overflow-freedom claim as the all-families harness, partitioned
/// per family (design Decision 4: each harness carries a minimal schema
/// and stated bounds). The all-families space stays declared with its
/// disposition; these carry the claim where the budget reaches it.
macro_rules! build_panic_freedom_over {
    ($name:ident, $shape:ident, $doc:literal) => {
        #[doc = $doc]
        #[kani::proof]
        fn $name() {
            let schema = shapes::$shape();
            let instance = shapes::instance_for(&schema);
            let typed = engine_build(&schema, &instance);
            let _ = typed.tables.len();
            // The claim is the walk; none of these types carries a
            // panicking drop, so the harness forgets them.
            std::mem::forget((schema, instance, typed));
        }
    };
}

build_panic_freedom_over!(
    build_panic_freedom_equation,
    schema_equation,
    "Panic/overflow-freedom of the engine build alone over the equation family's minimal space."
);
build_panic_freedom_over!(
    build_panic_freedom_key,
    schema_key,
    "Panic/overflow-freedom of the engine build alone over the key family's minimal space."
);
build_panic_freedom_over!(
    build_panic_freedom_coproduct,
    schema_coproduct,
    "Panic/overflow-freedom of the engine build alone over the coproduct family's minimal space."
);
build_panic_freedom_over!(
    build_panic_freedom_pullback,
    schema_pullback,
    "Panic/overflow-freedom of the engine build alone over the pullback family's minimal space."
);

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

/// Panic/overflow-freedom of the equation family's walk over its minimal
/// space — the check side's per-family partition (Decision 4 at walker
/// granularity, ledger `0007`'s recorded trigger): this harness's
/// formula carries only the equation walk; the dangling partition is
/// `check_dangling_panic_freedom`'s, and the two together carry the
/// whole-walk claim over the equation shape (the sort machinery and the
/// per-family walkers are each exercised in exactly one partition).
///
/// The declared unwind is 8, not the lane default 16: every loop this
/// harness executes is concretely bounded under the shape by 8 — the
/// synthesis scans (≤ 7 arrows, ≤ 4 objects, ≤ 2 rows per arrow, the
/// 8-byte `ident_key` pad), the build walk (7 arrows × 2 cells), the
/// compose path (1 step), the equation rows (2), the sort (≤ 2
/// emissions), and the harness loop — so 8 discharges every unwinding
/// assertion and halves the unrolled formula mass against the 8 GB
/// reduction (the equation partition's wall; check_dangling cleared the
/// same wall at 7 M clauses and 16 unwind). A loop that outgrows 8
/// fails loudly as an unwinding assertion, never silently.
#[kani::proof]
#[kani::unwind(8)]
fn check_equation_panic_freedom() {
    let schema = shapes::schema_equation();
    let (instance, typed) = typed_for(&schema);
    let mut findings = engine_equation_findings(&schema, &typed);
    // The family's sort obligation lives here (the entry emits in walk
    // order): the sort's panic-freedom over the equation partition's
    // symbolic keys stays a proven claim of this harness, while the
    // soundness harness — whose claims are order-free — consumes the
    // emissions without it.
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

/// Panic/overflow-freedom of the dangling family's walk over its minimal
/// space (the equation shape's dangling partition — the probe that
/// verified 47.7 s at the 2026-11-23 standard, now declared). The
/// equation partition is `check_equation_panic_freedom`'s.
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

/// Panic/overflow-freedom of the key family over its minimal space (one
/// two-arrow key over a two-row object; unresolved cells exclude their
/// row from the collision scan — never double-reported).
#[kani::proof]
fn check_key_panic_freedom() {
    let schema = shapes::schema_key();
    let (instance, typed) = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the coproduct family over its minimal space
/// (two injections into a two-row target; coverage and disjointness
/// decided only on resolvable cells).
#[kani::proof]
fn check_coproduct_panic_freedom() {
    let schema = shapes::schema_coproduct();
    let (instance, typed) = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Panic/overflow-freedom of the pullback family over its minimal space
/// (apex with two legs, cospan over a two-row target so commutes can
/// disagree; dangling legs suspend a row's verdicts, dangling cospan
/// cells shrink the decidable fiber).
#[kani::proof]
fn check_pullback_panic_freedom() {
    let schema = shapes::schema_pullback();
    let (instance, typed) = typed_for(&schema);
    let findings = engine_families(&schema, &typed);
    for (key, _, finding) in &findings {
        assert!(matches!(
            finding,
            CompactFinding::Violation(_) | CompactFinding::Dangling { .. }
        ));
        let _ = key;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, equation family, lhs half: every emitted equation
/// violation's lhs side recomposed independently (soundness submodule)
/// lands on the witness, with a genuinely disagreeing pair. The claim is
/// split per composition side (Decision 4 at obligation granularity,
/// 2026-11-24): the full both-sides formula exceeded the 8 GB reduction
/// even at unwind 8 with the sort dropped, so the rhs half is
/// `soundness_equation_rhs`'s and the two harnesses' union carries the
/// family's claim; the dispatch is partitioned per witness kind, and the
/// declared unwind is 8 (see `check_equation_panic_freedom` — every
/// executed loop is concretely bounded by 8 under the shape).
#[kani::proof]
#[kani::unwind(8)]
fn soundness_equation() {
    let schema = shapes::schema_equation();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_equation_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((
            _,
            _,
            CompactFinding::Violation(CompactViolation::Equation {
                index,
                row,
                lhs,
                rhs,
            }),
        )) = &findings[i]
        {
            soundness::assert_equation_lhs_sound(&schema, &typed, *index, *row, *lhs, *rhs);
        }
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, equation family, rhs half (see
/// `soundness_equation` for the split rationale and the unwind bound).
#[kani::proof]
#[kani::unwind(8)]
fn soundness_equation_rhs() {
    let schema = shapes::schema_equation();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_equation_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((
            _,
            _,
            CompactFinding::Violation(CompactViolation::Equation {
                index,
                row,
                lhs,
                rhs,
            }),
        )) = &findings[i]
        {
            soundness::assert_equation_rhs_sound(&schema, &typed, *index, *row, *lhs, *rhs);
        }
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, key family: every emitted collision's witness
/// re-evaluated independently finds equal, fully resolved tuples on
/// ascending rows.
#[kani::proof]
fn soundness_key() {
    let (schema, instance, typed, findings) = family_under_test(shapes::schema_key());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, coproduct family: coverage witnesses name rows in
/// no injection image; disjointness witnesses name colliding injections
/// that really cover their row.
#[kani::proof]
fn soundness_coproduct() {
    let (schema, instance, typed, findings) = family_under_test(shapes::schema_coproduct());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, pullback family: commutes witnesses re-derive both
/// compositions and the disagreement; covers witnesses re-derive the
/// uncovered fiber pair; unique witnesses re-derive the shared
/// projections.
#[kani::proof]
fn soundness_pullback() {
    let (schema, instance, typed, findings) = family_under_test(shapes::schema_pullback());
    let mut i = 0;
    while i < findings.len() {
        soundness::assert_finding_sound(&schema, &typed, &findings[i].2);
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, dangling family: every dangling witness names a
/// cell that really holds no row (its own dispatch arm — see
/// `soundness_equation` for the partition rationale).
#[kani::proof]
fn soundness_dangling() {
    let schema = shapes::schema_equation();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_dangling_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((_, _, CompactFinding::Dangling { arrow, src })) = &findings[i] {
            soundness::assert_dangling_sound(&typed, *arrow, *src);
        }
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

/// The schema, its tables, and the keyed findings — one soundness
/// sample space per family.
fn family_under_test(
    schema: EngineSchema,
) -> (
    EngineSchema,
    EngineInstance,
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
    (schema, instance, typed, findings)
}

#[kani::proof]
fn probe_build_only() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    assert_eq!(typed.tables.len(), 7);
}
