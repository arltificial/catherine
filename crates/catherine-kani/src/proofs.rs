//! The witness-soundness `#[kani::proof]` entry points (openspec change
//! `add-kani-verification-lane`, beads `catherine-6g2`): each harness
//! drives the shipped walk over one family's minimal shape and hands
//! every emitted finding to its independent obligation in
//! [`crate::soundness`]. Split out of `lib.rs` to keep both files inside
//! the file-size gate; the obligations and their rationale live in the
//! `soundness` module, the shapes in `shapes`.

use catherine_engine::engine::{
    engine_build, engine_coproduct_findings, engine_dangling_findings, engine_equation_findings,
    engine_key_findings, engine_pullback_findings, CompactFinding, CompactPullback,
    CompactViolation,
};

use crate::{shapes, soundness};

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
            soundness::assert_equation_lhs_sound(
                &soundness::Ctx::new(&schema, &typed, *index),
                *row,
                *lhs,
                *rhs,
            );
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
            soundness::assert_equation_rhs_sound(
                &soundness::Ctx::new(&schema, &typed, *index),
                *row,
                *lhs,
                *rhs,
            );
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
            soundness::assert_key_sound(&soundness::Ctx::new(&schema, &typed, *index), *lo, *hi);
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
                soundness::assert_coproduct_coverage_sound(
                    &soundness::Ctx::new(&schema, &typed, *index),
                    *row,
                );
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
                    &soundness::Ctx::new(&schema, &typed, *index),
                    *row,
                    injections,
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
            // The pullback verdicts carry their own schema index.
            let index = match pullback {
                CompactPullback::Commutes { index, .. }
                | CompactPullback::Covers { index, .. }
                | CompactPullback::Unique { index, .. } => *index,
            };
            soundness::assert_pullback_sound(
                &soundness::Ctx::new(&schema, &typed, index),
                pullback,
            );
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
