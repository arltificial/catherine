//! Purpose: the arbitrary well-typed `Model` generator the round-trip
//! properties draw from, plus its construction from drawn parts.
//! Responsibilities:
//!   - `arb_model`: unique object/arrow idents, equations whose path
//!     endpoints match, total per-arrow tables, and dense rows.
//!   - The builders underneath it: objects, clamped arrows, dense row
//!     spaces, canonical-order equations, and cycling cell tables.
//!   - The straight-line builders the deterministic ratchet fixtures in
//!     the parent binary construct models from: declared objects, a
//!     self-loop arrow, an opposite arrow pair, dense row spaces, and
//!     one total arrow table.
//!
//! Rationale: split out of the round-trip binary because the new
//! `instance_map` field pushed that binary one line past the 400-line
//! file-size gate; the generator and its builders are one cohesive
//! concern, and no property assertion moved with them. The
//! straight-line construction helpers the ratchet's deterministic
//! fixtures share sit here too, so both model families keep their
//! construction beside the strategy. The generator is a
//! `#[hegel::composite]`: every drawn range and the draw order mirror
//! the strategy it replaced, so the generated distribution (identifier
//! shapes, unique names, clamped arrow slots, dense row spaces, cycling
//! cell tables) and the well-typedness guarantees are unchanged.

use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Model, Path, RowId, Schema, Version,
};
use hegel::generators as gs;

/// Object names: `[A-Za-z_][A-Za-z0-9_]{0,7}`, made unique by the set
/// draw in `arb_model`.
fn ident() -> gs::RegexGenerator {
    gs::from_regex("[A-Za-z_][A-Za-z0-9_]{0,7}")
}

/// An arbitrary well-typed model: unique object/arrow idents, equations
/// whose path endpoints match, total per-arrow tables, and dense rows.
/// Equations are stored in canonical order, exactly as the parser stores
/// them, so equality with a re-parse is meaningful.
#[hegel::composite]
pub fn arb_model(tc: &hegel::TestCase) -> Model {
    let names = draw_object_names(tc);
    let slots = names.len().max(1);
    let arrow_defs = draw_arrows(tc, slots);
    let row_counts = draw_row_counts(tc, names.len());
    let slots = arrow_defs.len().max(1);
    let raw_equations = draw_equations(tc, slots);
    let cell_pool = draw_cell_pool(tc);
    build_model(&names, &arrow_defs, &row_counts, &raw_equations, &cell_pool)
}

/// Draw the unique object names.
fn draw_object_names(tc: &hegel::TestCase) -> Vec<String> {
    let objects: std::collections::BTreeSet<String> = tc.draw(gs::btree_sets(ident()).max_size(6));
    objects.into_iter().collect()
}

/// Draw the arrows as `(name, src, tgt)` slots into the object list.
fn draw_arrows(tc: &hegel::TestCase, slots: usize) -> Vec<(String, usize, usize)> {
    let arrows: std::collections::BTreeMap<String, (usize, usize)> = tc.draw(
        gs::btree_maps(
            gs::from_regex("[a-z_][a-z0-9_]{0,5}"),
            hegel::tuples!(
                gs::integers::<usize>().min_value(0).max_value(slots - 1),
                gs::integers::<usize>().min_value(0).max_value(slots - 1)
            ),
        )
        .max_size(4),
    );
    arrows
        .into_iter()
        .map(|(name, (s, t))| (name, s, t))
        .collect()
}

/// Draw one dense row count per object.
fn draw_row_counts(tc: &hegel::TestCase, names_len: usize) -> Vec<usize> {
    tc.draw(
        gs::vecs(gs::integers::<usize>().min_value(0).max_value(4))
            .min_size(names_len)
            .max_size(names_len),
    )
}

/// Draw up to three equation pairs of arrow-slot paths.
fn draw_equations(tc: &hegel::TestCase, slots: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
    tc.draw(gs::vecs(hegel::tuples!(drawn_path(slots), drawn_path(slots))).max_size(3))
}

/// Draw the 64-entry cycling cell pool.
fn draw_cell_pool(tc: &hegel::TestCase) -> Vec<usize> {
    tc.draw(
        gs::vecs(gs::integers::<usize>().min_value(0).max_value(4))
            .min_size(64)
            .max_size(64),
    )
}

/// One drawn arrow-slot path: 1..=3 slots into the `slots`-long arrow
/// list, exactly as the equation pairs draw them.
fn drawn_path(slots: usize) -> gs::VecGenerator<gs::IntegerGenerator<usize>, usize> {
    gs::vecs(gs::integers::<usize>().min_value(0).max_value(slots - 1))
        .min_size(1)
        .max_size(3)
}

/// Build the model from drawn parts. Empty-name models are the empty
/// document; equation pairs whose path endpoints do not match are dropped
/// at generation time (the generator, not the parser, must stay
/// well-typed).
fn build_model(
    names: &[String],
    arrows: &[(String, usize, usize)],
    row_counts: &[usize],
    raw_equations: &[(Vec<usize>, Vec<usize>)],
    cell_pool: &[usize],
) -> Model {
    let format_version = Version { major: 1, minor: 0 }; // within KNOWN_MINOR (parse::schema)
    if names.is_empty() {
        return Model {
            format_version,
            ..Model::empty()
        };
    }
    let arrows = arrow_map(names, arrows);
    let equations = build_equations(&arrows, raw_equations);
    let rows = row_map(names, row_counts);
    let values = value_map(&arrows, &rows, cell_pool);
    Model {
        format_version,
        schema: Schema {
            objects: object_map(names),
            arrows,
            equations,
            keys: Vec::new(),
            coproducts: Vec::new(),
            pullbacks: Vec::new(),
        },
        instance: Instance { rows, values },
        schema_map: None,
        rule_presentation: None,
        instance_map: None,
    }
}

/// The declared objects.
fn object_map(names: &[String]) -> std::collections::BTreeMap<Ident, ()> {
    names.iter().map(|n| (Ident(n.clone()), ())).collect()
}

/// The declared arrows: drawn (src, tgt) slots clamped into the object
/// list.
fn arrow_map(
    names: &[String],
    arrows: &[(String, usize, usize)],
) -> std::collections::BTreeMap<Ident, Arrow> {
    arrows
        .iter()
        .map(|(name, s, t)| {
            (
                Ident(name.clone()),
                Arrow {
                    src: Ident(names[s % names.len()].clone()),
                    tgt: Ident(names[t % names.len()].clone()),
                },
            )
        })
        .collect()
}

/// Dense row id spaces per object.
fn row_map(
    names: &[String],
    row_counts: &[usize],
) -> std::collections::BTreeMap<Ident, Vec<RowId>> {
    names
        .iter()
        .zip(row_counts.iter())
        .map(|(name, count)| (Ident(name.clone()), (0..*count).map(RowId).collect()))
        .collect()
}

/// The equations whose drawn path endpoints match, in canonical order as
/// the parser stores them (see `build_model` for the generator rule).
fn build_equations(
    arrows: &std::collections::BTreeMap<Ident, Arrow>,
    raw_equations: &[(Vec<usize>, Vec<usize>)],
) -> Vec<Equation> {
    if arrows.is_empty() {
        return Vec::new();
    }
    let arrow_names: Vec<Ident> = arrows.keys().cloned().collect();
    let arrow_list: Vec<&Arrow> = arrows.values().collect();
    let mut equations: Vec<Equation> = raw_equations
        .iter()
        .enumerate()
        .filter_map(|(i, (lhs_idx, rhs_idx))| {
            equation_candidate(&arrow_names, &arrow_list, i, lhs_idx, rhs_idx)
        })
        .collect();
    // Canonical order, as the parser stores them.
    equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });
    equations
}

/// One drawn equation pair, if its path endpoints match and its arrow
/// slots are in range.
fn equation_candidate(
    arrow_names: &[Ident],
    arrow_list: &[&Arrow],
    i: usize,
    lhs_idx: &[usize],
    rhs_idx: &[usize],
) -> Option<Equation> {
    if out_of_range(arrow_list, lhs_idx, rhs_idx) {
        return None;
    }
    if endpoints_of(arrow_list, lhs_idx) != endpoints_of(arrow_list, rhs_idx) {
        return None; // ill-typed candidate — a generator artifact, never a model
    }
    let to_path =
        |idx: &[usize]| -> Path { Path(idx.iter().map(|j| arrow_names[*j].clone()).collect()) };
    Some(Equation {
        lhs: to_path(lhs_idx),
        rhs: to_path(rhs_idx),
        rationale: format!("equation-{i}"),
        status: drawn_status(i),
    })
}

/// Even-indexed pairs are accepted, odd ones proposed.
fn drawn_status(i: usize) -> ConstraintStatus {
    if i.is_multiple_of(2) {
        ConstraintStatus::Accepted
    } else {
        ConstraintStatus::Proposed
    }
}

/// Whether either drawn path names an arrow slot beyond the arrow list.
fn out_of_range(arrow_list: &[&Arrow], lhs_idx: &[usize], rhs_idx: &[usize]) -> bool {
    lhs_idx.iter().any(|j| *j >= arrow_list.len()) || rhs_idx.iter().any(|j| *j >= arrow_list.len())
}

/// The (src, tgt) endpoint pair a drawn path's first and last arrow
/// slots resolve to.
fn endpoints_of(arrow_list: &[&Arrow], idx: &[usize]) -> Option<(Ident, Ident)> {
    let first = arrow_list.get(*idx.first()?)?;
    let last = arrow_list.get(*idx.last()?)?;
    Some((first.src.clone(), last.tgt.clone()))
}

/// Total per-arrow tables drawn from the cycling cell pool.
fn value_map(
    arrows: &std::collections::BTreeMap<Ident, Arrow>,
    rows: &std::collections::BTreeMap<Ident, Vec<RowId>>,
    cell_pool: &[usize],
) -> std::collections::BTreeMap<Ident, Vec<Option<RowId>>> {
    let mut pool = cell_pool.iter().copied().cycle();
    arrows
        .iter()
        .map(|(name, arrow)| (name.clone(), arrow_cells(&mut pool, rows, arrow)))
        .collect()
}

/// One arrow's cells: each source row draws a pool value; an empty
/// target space or the sentinel `4` leaves the cell dangling.
fn arrow_cells<I: Iterator<Item = usize>>(
    pool: &mut I,
    rows: &std::collections::BTreeMap<Ident, Vec<RowId>>,
    arrow: &Arrow,
) -> Vec<Option<RowId>> {
    let src_count = rows.get(&arrow.src).map_or(0, Vec::len);
    let tgt_count = rows.get(&arrow.tgt).map_or(0, Vec::len);
    (0..src_count)
        .map(|_| {
            let drawn = pool.next().unwrap_or(4);
            if tgt_count == 0 || drawn == 4 {
                None
            } else {
                Some(RowId(drawn % tgt_count))
            }
        })
        .collect()
}

/// One declared object.
pub(crate) fn object(names: &[&str]) -> std::collections::BTreeMap<Ident, ()> {
    names
        .iter()
        .map(|n| (Ident((*n).to_string()), ()))
        .collect()
}

/// One self-loop arrow on `obj`.
pub(crate) fn self_loop(name: &str, obj: &str) -> std::collections::BTreeMap<Ident, Arrow> {
    let mut arrows = std::collections::BTreeMap::new();
    arrows.insert(
        Ident(name.to_string()),
        Arrow {
            src: Ident(obj.to_string()),
            tgt: Ident(obj.to_string()),
        },
    );
    arrows
}

/// A pair of opposite arrows `f: p↔q` (`f` and `g` both declared).
pub(crate) fn cross_maps(
    f: &str,
    p: &str,
    q: &str,
    g: &str,
) -> std::collections::BTreeMap<Ident, Arrow> {
    let mut arrows = std::collections::BTreeMap::new();
    arrows.insert(
        Ident(f.to_string()),
        Arrow {
            src: Ident(p.to_string()),
            tgt: Ident(q.to_string()),
        },
    );
    arrows.insert(
        Ident(g.to_string()),
        Arrow {
            src: Ident(q.to_string()),
            tgt: Ident(p.to_string()),
        },
    );
    arrows
}

/// Dense row-id spaces `0..count` per named object.
pub(crate) fn dense_rows(spec: &[(&str, usize)]) -> std::collections::BTreeMap<Ident, Vec<RowId>> {
    spec.iter()
        .map(|(name, count)| (Ident((*name).to_string()), (0..*count).map(RowId).collect()))
        .collect()
}

/// One total arrow table: `values[name] = vec![target(i); n]`, parallel
/// to the source object's rows.
pub(crate) fn table(
    name: &str,
    n: usize,
    target: impl Fn(usize) -> Option<RowId>,
) -> std::collections::BTreeMap<Ident, Vec<Option<RowId>>> {
    let mut values = std::collections::BTreeMap::new();
    values.insert(
        Ident(name.to_string()),
        (0..n).map(target).collect::<Vec<_>>(),
    );
    values
}
