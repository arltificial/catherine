//! Fixture builders for the analysis-fixture tests (moved from the
//! former single-file `analysis_fixtures.rs`: only that binary uses them).

use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};
use std::collections::BTreeMap;

use crate::common::ident;

// ── fixture builders (moved from common: only this binary uses them) ──

pub(crate) fn path(names: &[&str]) -> Path {
    Path(names.iter().map(|n| ident(n)).collect())
}

pub(crate) fn arrow(src: &str, tgt: &str) -> Arrow {
    Arrow {
        src: ident(src),
        tgt: ident(tgt),
    }
}

pub(crate) fn equation(
    lhs: &[&str],
    rhs: &[&str],
    rationale: &str,
    status: ConstraintStatus,
) -> Equation {
    Equation {
        lhs: path(lhs),
        rhs: path(rhs),
        rationale: rationale.to_string(),
        status,
    }
}

pub(crate) fn schema(
    objects: &[&str],
    arrows: &[(&str, &str, &str)],
    equations: Vec<Equation>,
) -> Schema {
    Schema {
        objects: objects
            .iter()
            .map(|o| (ident(o), ()))
            .collect::<BTreeMap<_, _>>(),
        arrows: arrows
            .iter()
            .map(|(name, src, tgt)| (ident(name), arrow(src, tgt)))
            .collect::<BTreeMap<_, _>>(),
        equations,
    }
}

/// Per-object row maxima, spelled in test order (stored in a `BTreeMap`).
pub(crate) fn rows(pairs: &[(&str, usize)]) -> BTreeMap<Ident, usize> {
    pairs
        .iter()
        .map(|(o, n)| (ident(o), *n))
        .collect::<BTreeMap<_, _>>()
}

/// A total arrow table: every source row resolves to `pick(row)`.
pub(crate) fn total_table(n_src: usize, pick: impl Fn(usize) -> usize) -> Vec<Option<RowId>> {
    (0..n_src).map(|i| Some(RowId(pick(i)))).collect()
}

/// RED task 1 + 2 fixture: the over-constrained shape. Arrow `f: a→b`
/// with the bounds leaving `b` empty: *no fully resolved instance exists*
/// (every candidate dangles on `f`), so exhaustion proves the accepted
/// equation set admits no instance. `e0` routes through `b`; `e1` is an
/// accepted self-loop equation — together the enforced scope.
pub(crate) fn contradiction_schema() -> Schema {
    schema(
        &["a", "b", "c"],
        &[
            ("f", "a", "b"),
            ("g", "b", "c"),
            ("h", "a", "c"),
            ("s", "a", "a"),
            ("t", "a", "a"),
        ],
        vec![
            equation(
                &["f", "g"],
                &["h"],
                "the composite lands via b",
                ConstraintStatus::Accepted,
            ),
            equation(
                &["s"],
                &["t"],
                "the two endomorphisms agree",
                ConstraintStatus::Accepted,
            ),
        ],
    )
}

/// RED task 3 fixture: `f∘g` and `h` share endpoints (a→c) and no
/// equation relates them — the discovery pattern.
pub(crate) fn parallel_schema() -> Schema {
    schema(
        &["a", "b", "c"],
        &[("f", "a", "b"), ("g", "b", "c"), ("h", "a", "c")],
        vec![],
    )
}

/// RED task 4 fixture: `e2` (the route `f,g,k` equals `n`) is derivable
/// from `e0` (`f,g = h`) and `e1` (`h,k = n`) by rewriting; `e0` and
/// `e1` are not derivable from the others. Paths are spelled in the
/// kernel's application order (the lead arrow first — the same
/// convention as the bowling fixture's `staff_unit∘assigned_staff`).
pub(crate) fn rewrite_schema() -> Schema {
    schema(
        &["a", "b", "c", "d"],
        &[
            ("f", "a", "b"),
            ("g", "b", "c"),
            ("h", "a", "c"),
            ("k", "c", "d"),
            ("n", "a", "d"),
        ],
        vec![
            equation(
                &["f", "g"],
                &["h"],
                "h is the composite",
                ConstraintStatus::Accepted,
            ),
            equation(
                &["h", "k"],
                &["n"],
                "n continues the composite",
                ConstraintStatus::Accepted,
            ),
            equation(
                &["f", "g", "k"],
                &["n"],
                "same route spelled through the generators",
                ConstraintStatus::Accepted,
            ),
        ],
    )
}

/// catherine-7e8 fixture: three parallel arrows `p,q,r: a→c`; the
/// accepted equations `p=q` and `q=r` relate the outer pair only
/// transitively — rewriting can derive `p=r`, yet the (p,r) question
/// stays raised (documented direct-only limit).
pub(crate) fn transitive_chain_schema() -> Schema {
    schema(
        &["a", "c"],
        &[("p", "a", "c"), ("q", "a", "c"), ("r", "a", "c")],
        vec![
            equation(
                &["p"],
                &["q"],
                "p and q declared equal",
                ConstraintStatus::Accepted,
            ),
            equation(
                &["q"],
                &["r"],
                "q and r declared equal",
                ConstraintStatus::Accepted,
            ),
        ],
    )
}

/// RED task 5 fixture: one schema, two arrow tables; the two instances
/// differ only in where `f` sends the single `a` row.
pub(crate) fn distinguish_schema() -> Schema {
    schema(&["a", "b"], &[("f", "a", "b"), ("g", "a", "b")], vec![])
}

pub(crate) fn distinguish_pair() -> (Schema, Instance, Instance) {
    let instance_a = Instance {
        rows: rows_of(&[("a", 1), ("b", 2)]),
        values: vec![("f", total_table(1, |_| 0)), ("g", total_table(1, |_| 0))]
            .into_iter()
            .map(|(name, cells)| (ident(name), cells))
            .collect::<BTreeMap<_, _>>(),
    };
    let instance_b = Instance {
        rows: rows_of(&[("a", 1), ("b", 2)]),
        values: vec![("f", total_table(1, |_| 1)), ("g", total_table(1, |_| 0))]
            .into_iter()
            .map(|(name, cells)| (ident(name), cells))
            .collect::<BTreeMap<_, _>>(),
    };
    (distinguish_schema(), instance_a, instance_b)
}

pub(crate) fn rows_of(objects: &[(&str, usize)]) -> BTreeMap<Ident, Vec<RowId>> {
    objects
        .iter()
        .map(|(o, n)| (ident(o), (0..*n).map(RowId).collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>()
}
