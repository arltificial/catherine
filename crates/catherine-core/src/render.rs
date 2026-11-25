//! Compact-to-public finding rendering — the only step that
//! materializes names, on the parsed flavor (concrete data).

use super::engine::{
    Bounded, CompactFinding, CompactPullback, CompactViolation, EngineSchema, Nid, ARROWS_CAP,
};
use crate::findings::{
    CoproductWitness, DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind,
    KeyWitness, PullbackWitness, ViolationWitness,
};
use crate::model::RowId;

/// Render one compact finding against the parsed flavor's canonical
/// names. Lane flavor never calls this (no names to render from; the
/// harnesses assert compact shapes).
pub(crate) fn render_compact(
    schema: &EngineSchema,
    label: EpistemicLabel,
    compact: &CompactFinding,
) -> Finding {
    let names = Names { schema, label };
    match compact {
        CompactFinding::Dangling { arrow, src } => render_dangling(&names, *arrow, *src),
        CompactFinding::Violation(violation) => render_violation(&names, violation),
    }
}

/// The parsed flavor's canonical names plus the finding's label — what
/// every rendering helper needs beyond its own witness data.
struct Names<'a> {
    schema: &'a EngineSchema,
    label: EpistemicLabel,
}

/// The dangling family: the unresolved arrow cell that broke composition.
fn render_dangling(names: &Names<'_>, arrow: usize, src: usize) -> Finding {
    Finding {
        label: names.label,
        kind: FindingKind::Dangling(DanglingWitness {
            arrow: names.schema.arrow_names[arrow].clone(),
            src: RowId(src),
        }),
    }
}

/// The violation families, one helper per compact variant.
fn render_violation(names: &Names<'_>, violation: &CompactViolation) -> Finding {
    match violation {
        CompactViolation::Equation {
            index,
            row,
            lhs,
            rhs,
        } => render_equation(names.label, &sides(*index, *row, *lhs, *rhs)),
        CompactViolation::Key { index, lo, hi } => render_key(names.label, *index, *lo, *hi),
        CompactViolation::Coproduct {
            index,
            row,
            injections,
        } => render_coproduct(
            names,
            &Spot {
                index: *index,
                row: *row,
            },
            injections,
        ),
        CompactViolation::Pullback(pullback) => render_pullback(names.label, pullback),
    }
}

/// The witness sides of an equation-style finding, in field order.
fn sides(index: usize, row: usize, lhs: usize, rhs: usize) -> Sides {
    Sides {
        index,
        row,
        lhs,
        rhs,
    }
}
/// The equation-style witness data shared by the equation and commutes
/// families: the finding index, the row compared, and the recomposed
/// sides.
struct Sides {
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
}

/// The equation witness: recomposed lhs/rhs disagreeing on the row.
fn render_equation(label: EpistemicLabel, sides: &Sides) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: sides.index,
            row: RowId(sides.row),
            lhs: RowId(sides.lhs),
            rhs: RowId(sides.rhs),
        })),
    }
}

/// The key witness: the colliding row pair, ascending as carried.
fn render_key(label: EpistemicLabel, index: usize, lo: usize, hi: usize) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Key(KeyWitness {
            key: index,
            rows: [RowId(lo), RowId(hi)],
        })),
    }
}

/// The coproduct witness location: the finding index and the row compared.
struct Spot {
    index: usize,
    row: usize,
}

/// The coproduct witness: the covering (or colliding) injection arrows.
fn render_coproduct(
    names: &Names<'_>,
    spot: &Spot,
    injections: &Bounded<Nid, ARROWS_CAP>,
) -> Finding {
    Finding {
        label: names.label,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: spot.index,
            row: RowId(spot.row),
            injections: injections
                .iter()
                .filter_map(|cell| cell.as_ref())
                .map(|nid| names.schema.arrow_names[*nid].clone())
                .collect(),
        })),
    }
}

/// The pullback family's three verdicts, one helper per variant.
fn render_pullback(label: EpistemicLabel, pullback: &CompactPullback) -> Finding {
    match pullback {
        CompactPullback::Commutes {
            index,
            row,
            lhs,
            rhs,
        } => render_commutes(
            label,
            &Sides {
                index: *index,
                row: *row,
                lhs: *lhs,
                rhs: *rhs,
            },
        ),
        CompactPullback::Covers { index, pair } => render_covers(label, *index, pair[0], pair[1]),
        CompactPullback::Unique { index, rows } => render_unique(label, *index, rows),
    }
}

/// The commutes witness: the two cospan compositions disagreeing.
fn render_commutes(label: EpistemicLabel, sides: &Sides) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Commutes {
            pullback: sides.index,
            row: RowId(sides.row),
            lhs: RowId(sides.lhs),
            rhs: RowId(sides.rhs),
        })),
    }
}

/// The covers witness: the apex-row pair the square fails to cover.
fn render_covers(label: EpistemicLabel, index: usize, a: usize, b: usize) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Covers {
            pullback: index,
            pair: [RowId(a), RowId(b)],
        })),
    }
}

/// The unique witness: the pair of rows where projections collide.
fn render_unique(label: EpistemicLabel, index: usize, rows: &[usize; 2]) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Unique {
            pullback: index,
            rows: [RowId(rows[0]), RowId(rows[1])],
        })),
    }
}
