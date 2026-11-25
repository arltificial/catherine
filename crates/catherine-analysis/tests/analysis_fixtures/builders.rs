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

/// The parts of one equation fixture, grouped so builders stay within the
/// two-parameter test limit.
pub(crate) struct EquationInput<'a> {
    pub lhs: &'a [&'a str],
    pub rhs: &'a [&'a str],
    pub rationale: &'a str,
    pub status: ConstraintStatus,
}

pub(crate) fn equation(input: EquationInput) -> Equation {
    Equation {
        lhs: path(input.lhs),
        rhs: path(input.rhs),
        rationale: input.rationale.to_string(),
        status: input.status,
    }
}

/// The parts of one schema fixture, grouped so builders stay within the
/// two-parameter test limit.
pub(crate) struct SchemaInput<'a> {
    pub objects: &'a [&'a str],
    pub arrows: &'a [(&'a str, &'a str, &'a str)],
    pub equations: Vec<Equation>,
}

pub(crate) fn schema(input: SchemaInput) -> Schema {
    Schema {
        objects: input
            .objects
            .iter()
            .map(|o| (ident(o), ()))
            .collect::<BTreeMap<_, _>>(),
        arrows: input
            .arrows
            .iter()
            .map(|(name, src, tgt)| (ident(name), arrow(src, tgt)))
            .collect::<BTreeMap<_, _>>(),
        equations: input.equations,
        keys: Vec::new(),
        coproducts: Vec::new(),
        pullbacks: Vec::new(),
        stored: BTreeMap::new(),
    }
}

/// Per-object row maxima, spelled in test order (stored in a `BTreeMap`).
pub(crate) fn rows(pairs: &[(&str, usize)]) -> BTreeMap<Ident, usize> {
    let mut maxima = BTreeMap::new();
    for (object, count) in pairs {
        maxima.insert(ident(object), *count);
    }
    maxima
}

/// A total arrow table: every source row resolves to `pick(row)`.
pub(crate) fn total_table(n_src: usize, pick: impl Fn(usize) -> usize) -> Vec<Option<RowId>> {
    (0..n_src).map(|i| Some(RowId(pick(i)))).collect()
}

/// One table-driven fixture per analysis RED task; the name selects the
/// objects, arrows, and accepted equations.
///
/// - `"contradiction"` (RED task 1 + 2): arrow `f: a→b` with the bounds
///   leaving `b` empty — *no fully resolved instance exists* (every
///   candidate dangles on `f`), so exhaustion proves the accepted equation
///   set admits no instance. `e0` routes through `b`; `e1` is an accepted
///   self-loop equation — together the enforced scope.
/// - `"parallel"` (RED task 3): `f∘g` and `h` share endpoints (a→c) and no
///   equation relates them — the discovery pattern.
/// - `"rewrite"` (RED task 4): `e2` (the route `f,g,k` equals `n`) is
///   derivable from `e0` (`f,g = h`) and `e1` (`h,k = n`) by rewriting;
///   `e0` and `e1` are not derivable from the others. Paths are spelled in
///   the kernel's application order (the lead arrow first — the same
///   convention as the bowling fixture's `staff_unit∘assigned_staff`).
/// - `"transitive-chain"` (catherine-7e8): three parallel arrows
///   `p,q,r: a→c`; the accepted equations `p=q` and `q=r` relate the outer
///   pair only transitively — rewriting can derive `p=r`, yet the (p,r)
///   question stays raised (documented direct-only limit).
/// - `"distinguish"` (RED task 5): one schema, two arrow tables; the two
///   instances differ only in where `f` sends the single `a` row.
pub(crate) fn fixture_schema(name: &str) -> Schema {
    type FixtureShape<'a> = (
        &'a [&'a str],
        &'a [(&'a str, &'a str, &'a str)],
        Vec<Equation>,
    );
    let (objects, arrows, equations): FixtureShape = match name {
        "contradiction" => (
            &["a", "b", "c"],
            &[
                ("f", "a", "b"),
                ("g", "b", "c"),
                ("h", "a", "c"),
                ("s", "a", "a"),
                ("t", "a", "a"),
            ],
            vec![
                accepted((&["f", "g"], &["h"]), "the composite lands via b"),
                accepted((&["s"], &["t"]), "the two endomorphisms agree"),
            ],
        ),
        "parallel" => (
            &["a", "b", "c"],
            &[("f", "a", "b"), ("g", "b", "c"), ("h", "a", "c")],
            vec![],
        ),
        "rewrite" => (
            &["a", "b", "c", "d"],
            &[
                ("f", "a", "b"),
                ("g", "b", "c"),
                ("h", "a", "c"),
                ("k", "c", "d"),
                ("n", "a", "d"),
            ],
            vec![
                accepted((&["f", "g"], &["h"]), "h is the composite"),
                accepted((&["h", "k"], &["n"]), "n continues the composite"),
                accepted(
                    (&["f", "g", "k"], &["n"]),
                    "same route spelled through the generators",
                ),
            ],
        ),
        "transitive-chain" => (
            &["a", "c"],
            &[("p", "a", "c"), ("q", "a", "c"), ("r", "a", "c")],
            vec![
                accepted((&["p"], &["q"]), "p and q declared equal"),
                accepted((&["q"], &["r"]), "q and r declared equal"),
            ],
        ),
        "distinguish" => (&["a", "b"], &[("f", "a", "b"), ("g", "a", "b")], vec![]),
        other => panic!("no fixture schema named {other}"),
    };
    schema(SchemaInput {
        objects,
        arrows,
        equations,
    })
}

/// An accepted equation fixture from its two paths and its rationale.
fn accepted(paths: (&[&str], &[&str]), rationale: &str) -> Equation {
    equation(EquationInput {
        lhs: paths.0,
        rhs: paths.1,
        rationale,
        status: ConstraintStatus::Accepted,
    })
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
    (fixture_schema("distinguish"), instance_a, instance_b)
}

pub(crate) fn rows_of(objects: &[(&str, usize)]) -> BTreeMap<Ident, Vec<RowId>> {
    objects
        .iter()
        .map(|(o, n)| (ident(o), (0..*n).map(RowId).collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>()
}
