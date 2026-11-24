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
    CompactViolation, EngineInstance, EngineSchema, EngineTyped,
};

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

/// Witness soundness, equation family, lhs half: every emitted equation
/// violation's lhs side recomposed independently lands on the witness,
/// with a genuinely disagreeing pair. Split per composition side (Decision
/// 4, 2026-11-24); the rhs half is `soundness_equation_rhs`'s.
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

/// Witness soundness, equation family, rhs half (see `soundness_equation`).
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
/// re-evaluated independently finds equal, fully resolved tuples
/// (per-kind dispatch, unwind 9, order-free claims).
#[kani::proof]
#[kani::unwind(9)]
fn soundness_key() {
    let schema = shapes::schema_key();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_key_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((_, _, CompactFinding::Violation(CompactViolation::Key { index, lo, hi }))) =
            &findings[i]
        {
            soundness::assert_key_sound(&schema, &typed, *index, *lo, *hi);
        }
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, coproduct family, coverage half: a coverage
/// witness names a row in no complete injection image. Split per
/// witness shape (Decision 4, 2026-11-24); the collision half is
/// `soundness_coproduct_collision`'s. Unwind 4 — the coproduct shape's
/// loops are concrete (2 trips + exit); the mirror-era 8 unrolled past
/// need and peaked at 9.93 GB uncapped, over the 8 GB scope.
#[kani::proof]
#[kani::unwind(4)]
fn soundness_coproduct() {
    let schema = shapes::schema_coproduct();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_coproduct_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((
            _,
            _,
            CompactFinding::Violation(CompactViolation::Coproduct {
                index,
                row,
                injections,
            }),
        )) = &findings[i]
        {
            if injections.is_empty() {
                soundness::assert_coproduct_coverage_sound(&schema, &typed, *index, *row);
            }
        }
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, coproduct family, collision half (rationale in
/// `soundness_coproduct`'s doc). Unwind 4 — same concrete-bound sizing.
#[kani::proof]
#[kani::unwind(4)]
fn soundness_coproduct_collision() {
    let schema = shapes::schema_coproduct();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_coproduct_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((
            _,
            _,
            CompactFinding::Violation(CompactViolation::Coproduct {
                index,
                row,
                injections,
            }),
        )) = &findings[i]
        {
            if !injections.is_empty() {
                soundness::assert_coproduct_collision_sound(
                    &schema, &typed, *index, *row, injections,
                );
            }
        }
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, pullback family: each witness kind's claim
/// re-derived independently (per-kind dispatch, order-free). Unwind 8.
#[kani::proof]
#[kani::unwind(8)]
fn soundness_pullback() {
    let schema = shapes::schema_pullback();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    let findings = engine_pullback_findings(&schema, &typed);
    let mut i = 0;
    while i < findings.len() {
        if let Some((_, _, CompactFinding::Violation(CompactViolation::Pullback(pullback)))) =
            &findings[i]
        {
            soundness::assert_pullback_sound(&schema, &typed, pullback);
        }
        i += 1;
    }
    std::mem::forget((schema, instance, typed, findings));
}

/// Witness soundness, dangling family: every witness names a cell that really holds no row.
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

#[kani::proof]
fn probe_build_only() {
    let schema = shapes::schema_all_families();
    let instance = shapes::instance_for(&schema);
    let typed = engine_build(&schema, &instance);
    assert_eq!(typed.tables.len(), 7);
}
