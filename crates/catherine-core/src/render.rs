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
    match compact {
        CompactFinding::Dangling { arrow, src } => render_dangling(schema, label, *arrow, *src),
        CompactFinding::Violation(violation) => render_violation(schema, label, violation),
    }
}

/// The dangling family: the unresolved arrow cell that broke composition.
fn render_dangling(
    schema: &EngineSchema,
    label: EpistemicLabel,
    arrow: usize,
    src: usize,
) -> Finding {
    Finding {
        label,
        kind: FindingKind::Dangling(DanglingWitness {
            arrow: schema.arrow_names[arrow].clone(),
            src: RowId(src),
        }),
    }
}

/// The violation families, one helper per compact variant.
fn render_violation(
    schema: &EngineSchema,
    label: EpistemicLabel,
    violation: &CompactViolation,
) -> Finding {
    match violation {
        CompactViolation::Equation {
            index,
            row,
            lhs,
            rhs,
        } => render_equation(label, *index, *row, *lhs, *rhs),
        CompactViolation::Key { index, lo, hi } => render_key(label, *index, *lo, *hi),
        CompactViolation::Coproduct {
            index,
            row,
            injections,
        } => render_coproduct(schema, label, *index, *row, injections),
        CompactViolation::Pullback(pullback) => render_pullback(label, pullback),
    }
}

/// The equation witness: recomposed lhs/rhs disagreeing on the row.
fn render_equation(
    label: EpistemicLabel,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: index,
            row: RowId(row),
            lhs: RowId(lhs),
            rhs: RowId(rhs),
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

/// The coproduct witness: the covering (or colliding) injection arrows.
fn render_coproduct(
    schema: &EngineSchema,
    label: EpistemicLabel,
    index: usize,
    row: usize,
    injections: &Bounded<Nid, ARROWS_CAP>,
) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: index,
            row: RowId(row),
            injections: injections
                .iter()
                .filter_map(|cell| cell.as_ref())
                .map(|nid| schema.arrow_names[*nid].clone())
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
        } => render_commutes(label, *index, *row, *lhs, *rhs),
        CompactPullback::Covers { index, pair } => render_covers(label, *index, pair[0], pair[1]),
        CompactPullback::Unique { index, rows } => render_unique(label, *index, rows[0], rows[1]),
    }
}

/// The commutes witness: the two cospan compositions disagreeing.
fn render_commutes(
    label: EpistemicLabel,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Commutes {
            pullback: index,
            row: RowId(row),
            lhs: RowId(lhs),
            rhs: RowId(rhs),
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
fn render_unique(label: EpistemicLabel, index: usize, lo: usize, hi: usize) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Unique {
            pullback: index,
            rows: [RowId(lo), RowId(hi)],
        })),
    }
}
