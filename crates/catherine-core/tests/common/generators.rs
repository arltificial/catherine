//! Purpose: the canonical model generator shared by the property suites —
//! draws a `FixtureCase` (arrow shapes, per-object row counts, per-arrow
//! cell tables, equation seeds, equation statuses) and builds it through
//! `common::fixture::build_case`, which keeps the tables well-typed.
//!
//! Responsibilities: host exactly one generator, [`arb_case`], and nothing
//! suite-specific. The executed-checks properties draw it today; the
//! findings-taxonomy properties reuse it unchanged. The equations are
//! sorted into the parser's canonical storage order before the case is
//! returned, so every suite draws the same well-ordered models.
//!
//! Rationale: a single canonical draw keeps the suites exploring one
//! well-typed model space instead of drifting apart per file. The draw
//! order is the dependency order — shapes → row counts → cells → equation
//! seeds → statuses — because each later draw is sized (or bounded) by the
//! values drawn before it; a sequential composite expresses that directly,
//! where a flat strategy would have to be flattened and rebuilt by hand.

use hegel::generators as gs;
use hegel::TestCase;

use super::fixture::{self, FixtureCase};

/// Draw a well-typed model case: 1–3 objects, 1–4 arrows (self-loops
/// included), 0–3 rows per object, a cell pool sized by the drawn shapes,
/// up to three equation seeds and up to three statuses. The equations are
/// sorted into the parser's canonical order.
#[hegel::composite]
pub fn arb_case(tc: &TestCase) -> FixtureCase {
    let n_objects = tc.draw(gs::integers::<usize>().min_value(1).max_value(3));
    let n_arrows = tc.draw(gs::integers::<usize>().min_value(1).max_value(4));

    let shapes = draw_shapes(tc, (n_objects, n_arrows));
    let row_counts = draw_row_counts(tc, n_objects);
    let cell_pool = draw_cell_pool(tc, (&shapes, &row_counts));
    let (eq_seeds, statuses) = draw_equations(tc);

    let mut case = fixture::build_case(fixture::DrawnParts {
        shapes,
        row_counts,
        cell_pool,
        eq_seeds,
        statuses,
    });
    sort_equations(&mut case);
    case
}

/// Arrow shapes: `n_arrows` `(src, tgt)` pairs over `n_objects` objects,
/// self-loops included.
fn draw_shapes(tc: &TestCase, layout: (usize, usize)) -> Vec<(usize, usize)> {
    let (n_objects, n_arrows) = layout;
    tc.draw(
        gs::vecs(hegel::tuples!(
            gs::integers::<usize>()
                .min_value(0)
                .max_value(n_objects - 1),
            gs::integers::<usize>()
                .min_value(0)
                .max_value(n_objects - 1),
        ))
        .min_size(n_arrows)
        .max_size(n_arrows),
    )
}

/// Per-object row counts: 0–3 rows for each of the `n_objects` objects.
fn draw_row_counts(tc: &TestCase, n_objects: usize) -> Vec<usize> {
    tc.draw(
        gs::vecs(gs::integers::<usize>().min_value(0).max_value(3))
            .min_size(n_objects)
            .max_size(n_objects),
    )
}

/// The cell pool must cover every source row of every arrow; the
/// builder consumes it in arrow order and ignores any surplus.
fn draw_cell_pool(tc: &TestCase, drawn: (&[(usize, usize)], &[usize])) -> Vec<usize> {
    let (shapes, row_counts) = drawn;
    let n_cells: usize = shapes
        .iter()
        .enumerate()
        .map(|(i, &(s, _))| row_counts[s] * (i + 1))
        .sum::<usize>()
        .max(1);
    tc.draw(
        gs::vecs(gs::integers::<usize>().min_value(0).max_value(8))
            .min_size(n_cells)
            .max_size(n_cells),
    )
}

/// Up to three equation seeds and up to three statuses.
fn draw_equations(tc: &TestCase) -> (Vec<(usize, usize)>, Vec<bool>) {
    let eq_seeds = tc.draw(
        gs::vecs(hegel::tuples!(
            gs::integers::<usize>().min_value(0).max_value(7),
            gs::integers::<usize>().min_value(0).max_value(7),
        ))
        .min_size(0)
        .max_size(3),
    );
    let statuses = tc.draw(gs::vecs(gs::booleans()).min_size(0).max_size(3));
    (eq_seeds, statuses)
}

/// Sort the built equations into the parser's canonical storage order.
fn sort_equations(case: &mut FixtureCase) {
    case.equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });
}
