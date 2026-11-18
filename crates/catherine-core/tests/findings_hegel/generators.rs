//! The findings witness generators drawn by the sibling
//! `findings_hegel.rs` properties: labels, idents, paths, domain
//! questions and the six-way finding matrix.
//!
//! Responsibilities: hold every draw those properties use — a 3-way
//! label, `0..8` row/id indices, 1..=3-ident paths, at-most-40-char
//! question text and equal weight over the six finding kinds, each kind
//! in its own small helper so no single draw body carries the whole
//! matrix. The model generator itself is the shared
//! `common::generators::arb_case`, not a local copy.
//!
//! Rationale: the generators have exactly one consumer (the sibling
//! property file), so they sit next to the properties that pin their
//! shape rather than in the shared module. The draws mirror the
//! strategies they replaced, 1:1 — 3-way labels, `0..8` row/id indices,
//! 1..=3-ident paths, at-most-40-char question text and equal weight
//! over the six kinds — so the port explores the space the properties
//! were written against.

use catherine_core::findings::{
    ContradictionWitness, CoproductWitness, DanglingWitness, DomainQuestion, EpistemicLabel,
    EquationWitness, Finding, FindingKind, RedundantWitness, UnderconstraintWitness,
    ViolationWitness,
};
use catherine_core::model::{Ident, Path, RowId};

use hegel::generators as gs;
use hegel::TestCase;

/// One of the three epistemic labels, drawn uniformly.
#[hegel::composite]
pub fn arb_label(tc: &TestCase) -> EpistemicLabel {
    match tc.draw(gs::integers::<usize>().min_value(0).max_value(2)) {
        0 => EpistemicLabel::Executed,
        1 => EpistemicLabel::Hypothesized,
        _ => EpistemicLabel::Unknown,
    }
}

/// An ident: a `[a-z][a-z0-9_]{0,7}` stem followed by a row index in
/// `0..8`.
#[hegel::composite]
pub fn arb_ident(tc: &TestCase) -> Ident {
    let stem = tc.draw(gs::from_regex("[a-z][a-z0-9_]{0,7}"));
    let n = tc.draw(gs::integers::<usize>().min_value(0).max_value(7));
    Ident(format!("{stem}{n}"))
}

/// A path: one to three idents, in composition order.
#[hegel::composite]
pub fn arb_path(tc: &TestCase) -> Path {
    Path(tc.draw(gs::vecs(arb_ident()).min_size(1).max_size(3)))
}

/// A domain question: two parallel paths or an uncovered-input class,
/// equally weighted.
#[hegel::composite]
pub fn arb_domain_question(tc: &TestCase) -> DomainQuestion {
    match tc.draw(gs::integers::<usize>().min_value(0).max_value(1)) {
        0 => DomainQuestion::ParallelPaths {
            lhs: tc.draw(arb_path()),
            rhs: tc.draw(arb_path()),
        },
        _ => DomainQuestion::UncoveredInputs {
            class: tc.draw(gs::from_regex("[a-z ]{0,20}")),
            examples: drawn_rows(tc),
        },
    }
}

/// Row ids in `0..8`, at most four of them — the example and
/// contradiction family lists never carry more.
fn drawn_rows(tc: &TestCase) -> Vec<RowId> {
    tc.draw(gs::vecs(gs::integers::<usize>().min_value(0).max_value(7)).max_size(4))
        .into_iter()
        .map(RowId)
        .collect()
}

/// A single index in `0..8`.
fn drawn_index(tc: &TestCase) -> usize {
    tc.draw(gs::integers::<usize>().min_value(0).max_value(7))
}

/// Arbitrary findings across all five kinds and all three labels —
/// including the analysis kinds, which are representable values even
/// though this crate's kernel never produces them. The six kinds carry
/// equal weight.
#[hegel::composite]
pub fn arb_finding(tc: &TestCase) -> Finding {
    let kind = match tc.draw(gs::integers::<usize>().min_value(0).max_value(5)) {
        0 => equation_kind(tc),
        1 => dangling_kind(tc),
        2 => coproduct_kind(tc),
        3 => redundant_kind(tc),
        4 => contradiction_kind(tc),
        _ => underconstraint_kind(tc),
    };
    Finding {
        label: tc.draw(arb_label()),
        kind,
    }
}

/// The equation-violation kind: an equation index in `0..8`, a row of
/// the equation's domain, and both composed values, each a row index in
/// `0..8`.
fn equation_kind(tc: &TestCase) -> FindingKind {
    FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
        equation: drawn_index(tc),
        row: RowId(drawn_index(tc)),
        lhs: RowId(drawn_index(tc)),
        rhs: RowId(drawn_index(tc)),
    }))
}

/// The dangling-arrow kind: an ident and a source row index in `0..8`.
fn dangling_kind(tc: &TestCase) -> FindingKind {
    FindingKind::Dangling(DanglingWitness {
        arrow: tc.draw(arb_ident()),
        src: RowId(drawn_index(tc)),
    })
}

/// The coproduct-violation kind: a coproduct index and row index in
/// `0..8`, plus at most three injection idents.
fn coproduct_kind(tc: &TestCase) -> FindingKind {
    FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
        coproduct: drawn_index(tc),
        row: RowId(drawn_index(tc)),
        injections: tc.draw(gs::vecs(arb_ident()).max_size(3)),
    }))
}

/// The redundant-path kind: the derivable path and the path reducing it.
fn redundant_kind(tc: &TestCase) -> FindingKind {
    FindingKind::Redundant(RedundantWitness {
        derivable: tc.draw(arb_path()),
        reducing: tc.draw(arb_path()),
    })
}

/// A family list: at most four indices drawn from `0..8`.
fn drawn_family(tc: &TestCase) -> Vec<usize> {
    tc.draw(gs::vecs(gs::integers::<usize>().min_value(0).max_value(7)).max_size(4))
}

/// The contradiction kind: the four family lists — scope, keys,
/// coproducts, pullbacks — each at most four `0..8` indices.
fn contradiction_kind(tc: &TestCase) -> FindingKind {
    FindingKind::Contradiction(ContradictionWitness {
        scope: drawn_family(tc),
        keys: drawn_family(tc),
        coproducts: drawn_family(tc),
        pullbacks: drawn_family(tc),
    })
}

/// The underconstraint kind: uncovered-input question text of at most
/// 40 chars and one domain question.
fn underconstraint_kind(tc: &TestCase) -> FindingKind {
    FindingKind::Underconstraint(UnderconstraintWitness {
        question: tc.draw(gs::from_regex(".{0,40}")),
        case: tc.draw(arb_domain_question()),
    })
}
