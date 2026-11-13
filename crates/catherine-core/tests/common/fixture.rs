//! Shared property-test fixture builders: the `FixtureCase` the
//! properties tests draw at random, its well-typed construction from
//! drawn parts, and its assembly into a `Schema` + `Instance`.

use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};
use std::collections::BTreeMap;

/// Object names, unique by construction.
pub const OBJECTS: [&str; 3] = ["O0", "O1", "O2"];

/// A drawn property-test case: arrow shapes, per-object row counts,
/// per-arrow cell tables (value = tgt row idx, `None` = dangling), and
/// the equations the seeds generated. The executed-checks properties
/// sort the equations into the parser's canonical order; the findings
/// taxonomy properties keep the deterministic generation order.
#[derive(Debug, Clone)]
pub struct FixtureCase {
    pub arrows: Vec<(usize, usize)>, // (src object idx, tgt object idx); src == tgt is a self-loop
    pub row_counts: Vec<usize>,
    pub cells: Vec<Vec<Option<usize>>>, // per arrow, parallel to src rows; value = tgt row idx
    pub equations: Vec<Equation>,
}

/// Build the case from drawn parts: well-typed tables (a drawn value is
/// clamped into the target object's row space, an empty target space
/// leaves the cell dangling), equations over endpoint-matched candidate
/// paths, in deterministic generation order.
pub fn build_case(
    shapes: &[(usize, usize)],
    row_counts: &[usize],
    cell_pool: &[usize],
    eq_seeds: &[(usize, usize)],
    statuses: &[bool],
) -> FixtureCase {
    FixtureCase {
        arrows: shapes.to_vec(),
        row_counts: row_counts.to_vec(),
        cells: cells_for(shapes, row_counts, cell_pool),
        equations: equations_for_seeds(
            eq_seeds,
            &candidate_groups(shapes, row_counts.len()),
            statuses,
        ),
    }
}

/// The case's cell tables: per arrow, one clamped cell per source row.
fn cells_for(
    shapes: &[(usize, usize)],
    row_counts: &[usize],
    cell_pool: &[usize],
) -> Vec<Vec<Option<usize>>> {
    let mut pool = cell_pool.iter().copied();
    shapes
        .iter()
        .map(|&(s, t)| {
            (0..row_counts[s])
                .map(|_| cell_for(&mut pool, row_counts[t]))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// One drawn cell: clamped into the target's row space; an empty target
/// space leaves the cell dangling (a representable value).
fn cell_for<I: Iterator<Item = usize>>(pool: &mut I, tgt_rows: usize) -> Option<usize> {
    match pool.next() {
        Some(v) if tgt_rows > 0 => Some(v % tgt_rows),
        _ => None,
    }
}

/// Candidate equation paths grouped by endpoint pair `(s, t)` in pair
/// order, so lhs/rhs always come from matching endpoints — parse rejects
/// mismatched endpoints (ill_typed_equation), so the generator must stay
/// well-typed.
fn candidate_groups(shapes: &[(usize, usize)], max_objects: usize) -> Vec<Vec<Vec<usize>>> {
    let mut by_endpoints: BTreeMap<(usize, usize), Vec<Vec<usize>>> = BTreeMap::new();
    for (s, t) in pair_keys(shapes.len().min(max_objects)) {
        let matching = paths_between(shapes, s, t);
        if !matching.is_empty() {
            by_endpoints.insert((s, t), matching);
        }
    }
    by_endpoints.into_values().collect()
}

/// All `(s, t)` pairs over `0..n`, row-major.
fn pair_keys(n: usize) -> Vec<(usize, usize)> {
    (0..n).flat_map(|s| (0..n).map(move |t| (s, t))).collect()
}

/// Candidate paths from `s` to `t`: one-hop arrows and two-hop
/// compositions through an intermediate object.
fn paths_between(shapes: &[(usize, usize)], s: usize, t: usize) -> Vec<Vec<usize>> {
    let mut paths = Vec::new();
    for (i, &(src, tgt)) in shapes.iter().enumerate() {
        if src == s {
            paths.extend(hops_through(shapes, i, tgt, t));
        }
    }
    paths
}

/// The paths starting at arrow `i` (whose target is `tgt`) and ending at
/// `t`: the one-hop `i` itself, then the two-hops `i` followed by any
/// other arrow landing on `t`.
fn hops_through(shapes: &[(usize, usize)], i: usize, tgt: usize, t: usize) -> Vec<Vec<usize>> {
    let mut paths = Vec::new();
    if tgt == t {
        paths.push(vec![i]);
    }
    paths.extend(two_hop_matches(shapes, i, tgt, t));
    paths
}

/// The second hops completing arrow `i` onto `t`: other arrows `j` whose
/// source is `i`'s target and whose target is `t`.
fn two_hop_matches(shapes: &[(usize, usize)], i: usize, tgt: usize, t: usize) -> Vec<Vec<usize>> {
    shapes
        .iter()
        .enumerate()
        .filter(|&(j, &(src2, tgt2))| j != i && src2 == tgt && tgt2 == t)
        .map(|(j, _)| vec![i, j])
        .collect()
}

/// The equations the seeds generate over the candidate groups, in
/// generation order.
fn equations_for_seeds(
    eq_seeds: &[(usize, usize)],
    candidates: &[Vec<Vec<usize>>],
    statuses: &[bool],
) -> Vec<Equation> {
    eq_seeds
        .iter()
        .enumerate()
        .filter_map(|(k, seed)| seed_equation(k, seed, candidates, statuses))
        .collect()
}

/// One seed's equation, if any candidates exist.
fn seed_equation(
    k: usize,
    seed: &(usize, usize),
    candidates: &[Vec<Vec<usize>>],
    statuses: &[bool],
) -> Option<Equation> {
    if candidates.is_empty() {
        return None;
    }
    let group = &candidates[(seed.0 + seed.1 + k) % candidates.len()];
    Some(Equation {
        lhs: Path(path_idents(&group[seed.0 % group.len()])),
        rhs: Path(path_idents(&group[seed.1 % group.len()])),
        rationale: format!("property equation {k}"),
        status: seed_status(statuses, k),
    })
}

/// A drawn equation's status: a drawn `true` marks it proposed.
fn seed_status(statuses: &[bool], k: usize) -> ConstraintStatus {
    if statuses.get(k).copied().unwrap_or(false) {
        ConstraintStatus::Proposed
    } else {
        ConstraintStatus::Accepted
    }
}

/// The candidate path's arrow-name idents (`A<i>` per hop index).
pub fn path_idents(path: &[usize]) -> Vec<Ident> {
    path.iter().map(|i| Ident(format!("A{i}"))).collect()
}

/// Assemble the `Schema` + `Instance` the case describes.
pub fn assemble(case: &FixtureCase) -> (Schema, Instance) {
    (
        Schema {
            objects: objects_of(&case.row_counts),
            arrows: arrows_of(&case.arrows),
            equations: case.equations.clone(),
            keys: Vec::new(),
        },
        Instance {
            rows: rows_of(&case.row_counts),
            values: values_of(&case.cells),
        },
    )
}

/// The declared objects: the first `row_counts.len()` names.
fn objects_of(row_counts: &[usize]) -> BTreeMap<Ident, ()> {
    OBJECTS
        .iter()
        .take(row_counts.len())
        .map(|&name| (Ident(name.to_string()), ()))
        .collect()
}

/// The declared arrows: `A<i>` over the drawn shapes.
fn arrows_of(shapes: &[(usize, usize)]) -> BTreeMap<Ident, Arrow> {
    shapes
        .iter()
        .enumerate()
        .map(|(i, &(s, t))| {
            (
                Ident(format!("A{i}")),
                Arrow {
                    src: Ident(OBJECTS[s].to_string()),
                    tgt: Ident(OBJECTS[t].to_string()),
                },
            )
        })
        .collect()
}

/// Dense row id spaces per object.
fn rows_of(row_counts: &[usize]) -> BTreeMap<Ident, Vec<RowId>> {
    row_counts
        .iter()
        .enumerate()
        .map(|(o, &count)| {
            (
                Ident(OBJECTS[o].to_string()),
                (0..count).map(RowId).collect(),
            )
        })
        .collect()
}

/// Total per-arrow value vectors in the source-row id space.
fn values_of(cells: &[Vec<Option<usize>>]) -> BTreeMap<Ident, Vec<Option<RowId>>> {
    cells
        .iter()
        .enumerate()
        .map(|(i, arrow_cells)| {
            (
                Ident(format!("A{i}")),
                arrow_cells.iter().map(|c| c.map(RowId)).collect::<Vec<_>>(),
            )
        })
        .collect()
}
