//! Purpose: the executed-checks properties (proptest, never
//! insta-snapshots; openspec change `add-executed-checks`, task 10,
//! reshaped onto the findings taxonomy by `add-findings-taxonomy`) —
//! totality (empty schema or empty instance → zero findings), self-loops
//! legal and evaluated like any arrow, and the findings matching an
//! independent recomputation of the check contract over arbitrary
//! well-typed models. Findings are asserted structurally — kind +
//! witness + label — never byte snapshots; the vendored parity sets in
//! `fixtures/parity/` are the parity contract, not snapshots.

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Object names, unique by construction.
const OBJECTS: [&str; 3] = ["O0", "O1", "O2"];

#[derive(Debug, Clone)]
struct Case {
    arrows: Vec<(usize, usize)>, // (src object idx, tgt object idx); src == tgt is a self-loop
    row_counts: Vec<usize>,
    cells: Vec<Vec<Option<usize>>>, // per arrow, parallel to src rows; value = tgt row idx
    equations: Vec<Equation>,       // canonical order, as the parser stores them
}

fn arb_case() -> impl Strategy<Value = Case> {
    (1usize..=3usize, 1usize..=4usize)
        .prop_flat_map(|(n_objects, n_arrows)| {
            (
                Just(n_objects),
                proptest::collection::vec((0..n_objects, 0..n_objects), n_arrows),
                proptest::collection::vec(0usize..=3usize, n_objects),
            )
        })
        .prop_flat_map(|(_n_objects, shapes, row_counts)| {
            let n_cells: usize = shapes
                .iter()
                .enumerate()
                .map(|(i, &(s, _))| row_counts[s] * (i + 1))
                .sum::<usize>()
                .max(1);
            let cell_pool = proptest::collection::vec(0usize..=8usize, n_cells);
            let eq_seeds = proptest::collection::vec((0usize..=7usize, 0usize..=7usize), 0..=3);
            let statuses = proptest::collection::vec(any::<bool>(), 0..=3);
            (
                Just(shapes),
                Just(row_counts),
                cell_pool,
                eq_seeds,
                statuses,
            )
        })
        .prop_map(|(shapes, row_counts, cell_pool, eq_seeds, statuses)| {
            build_case(&shapes, &row_counts, &cell_pool, &eq_seeds, &statuses)
        })
}

/// Build the case from drawn parts: well-typed tables (a drawn value is
/// clamped into the target object's row space), equations over candidate
/// paths with matching endpoints, stored in canonical order.
fn build_case(
    shapes: &[(usize, usize)],
    row_counts: &[usize],
    cell_pool: &[usize],
    eq_seeds: &[(usize, usize)],
    statuses: &[bool],
) -> Case {
    let mut pool = cell_pool.iter().copied();
    let mut cells = Vec::new();
    for &(s, t) in shapes.iter() {
        let tgt_rows = row_counts[t];
        let arrow_cells = (0..row_counts[s])
            .map(|_| match pool.next() {
                // Clamp into the target's row space; an empty target space
                // leaves the cell dangling (a representable value).
                Some(v) if tgt_rows > 0 => Some(v % tgt_rows),
                _ => None,
            })
            .collect::<Vec<_>>();
        cells.push(arrow_cells);
    }

    // Candidate paths between an endpoint pair (s, t): 1-hop arrows and
    // 2-hop compositions through an intermediate object.
    let mut paths: Vec<Vec<usize>> = Vec::new();
    for (s, t) in endpoint_pairs(shapes.len(), row_counts.len()) {
        for (i, &(src, tgt)) in shapes.iter().enumerate() {
            if src == s && tgt == t {
                paths.push(vec![i]);
            }
            if src == s {
                for (j, &(src2, tgt2)) in shapes.iter().enumerate() {
                    if i != j && src2 == tgt && tgt2 == t {
                        paths.push(vec![i, j]);
                    }
                }
            }
        }
    }
    // Group candidate paths by (s, t) so lhs/rhs come from the same
    // endpoints — parse rejects mismatched endpoints (ill_typed_equation),
    // so the generator must stay well-typed.
    let mut by_endpoints: BTreeMap<(usize, usize), Vec<Vec<usize>>> = BTreeMap::new();
    for (s, t) in endpoint_pairs(shapes.len(), row_counts.len()) {
        let matching: Vec<Vec<usize>> = paths
            .iter()
            .filter(|p| path_endpoints(shapes, p) == (s, t))
            .cloned()
            .collect();
        if !matching.is_empty() {
            by_endpoints.insert((s, t), matching);
        }
    }
    let candidates: Vec<&Vec<Vec<usize>>> = by_endpoints.values().collect();
    let mut equations = Vec::new();
    for (k, &(l, r)) in eq_seeds.iter().enumerate() {
        if candidates.is_empty() {
            break;
        }
        let group = candidates[(l + r + k) % candidates.len()];
        let lhs = group[l % group.len()].clone();
        let rhs = group[r % group.len()].clone();
        let status = if statuses.get(k).copied().unwrap_or(false) {
            ConstraintStatus::Proposed
        } else {
            ConstraintStatus::Accepted
        };
        equations.push(Equation {
            lhs: Path(path_idents(&lhs)),
            rhs: Path(path_idents(&rhs)),
            rationale: format!("property equation {k}"),
            status,
        });
    }
    // Canonical order — the parser's storage rule.
    equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });

    Case {
        arrows: shapes.to_vec(),
        row_counts: row_counts.to_vec(),
        cells,
        equations,
    }
}

fn endpoint_pairs(max_arrows: usize, max_objects: usize) -> Vec<(usize, usize)> {
    let n = max_arrows.min(max_objects);
    let mut pairs = Vec::new();
    for s in 0..n {
        for t in 0..n {
            pairs.push((s, t));
        }
    }
    pairs
}

fn path_endpoints(shapes: &[(usize, usize)], path: &[usize]) -> (usize, usize) {
    (
        shapes[path[0]].0,
        shapes[*path.last().expect("non-empty path")].1,
    )
}

fn path_idents(path: &[usize]) -> Vec<Ident> {
    path.iter().map(|i| Ident(format!("A{i}"))).collect()
}

/// Assemble the `Schema` + `Instance` the case describes.
fn assemble(case: &Case) -> (Schema, Instance) {
    let mut objects = BTreeMap::new();
    for name in OBJECTS.iter().take(case.row_counts.len()) {
        objects.insert(Ident(name.to_string()), ());
    }
    let mut arrows = BTreeMap::new();
    for (i, &(s, t)) in case.arrows.iter().enumerate() {
        arrows.insert(
            Ident(format!("A{i}")),
            Arrow {
                src: Ident(OBJECTS[s].to_string()),
                tgt: Ident(OBJECTS[t].to_string()),
            },
        );
    }
    let mut rows = BTreeMap::new();
    for (o, &count) in case.row_counts.iter().enumerate() {
        rows.insert(
            Ident(OBJECTS[o].to_string()),
            (0..count).map(RowId).collect::<Vec<_>>(),
        );
    }
    let mut values = BTreeMap::new();
    for (i, arrow_cells) in case.cells.iter().enumerate() {
        values.insert(
            Ident(format!("A{i}")),
            arrow_cells.iter().map(|c| c.map(RowId)).collect::<Vec<_>>(),
        );
    }
    (
        Schema {
            objects,
            arrows,
            equations: case.equations.clone(),
        },
        Instance { rows, values },
    )
}

/// The independent recomputation of the check contract (written from the
/// kernel-checks + findings specs, not from `check`'s code): one executed
/// dangling finding per unresolved, non-typing-violated cell (arrow-name
/// order, then source row order), then per equation (canonical order) per
/// domain row, a violation-shaped finding labeled by status when both
/// paths compose and disagree — sorted under the taxonomy's total order
/// (kind first, then witness ids), exactly as `check` emits.
fn expected_findings(schema: &Schema, built: &TypedInstance) -> Vec<Finding> {
    let mut out = Vec::new();
    let violated: Vec<(String, usize)> = built
        .typing_violations()
        .iter()
        .map(|v| (v.arrow.0.clone(), v.src.0))
        .collect();
    for (name, cells) in built.tables() {
        for (i, cell) in cells.iter().enumerate() {
            if cell.is_none() && !violated.contains(&(name.0.clone(), i)) {
                out.push(Finding {
                    label: EpistemicLabel::Executed,
                    kind: FindingKind::Dangling(DanglingWitness {
                        arrow: name.clone(),
                        src: RowId(i),
                    }),
                });
            }
        }
    }
    for (index, equation) in schema.equations.iter().enumerate() {
        let Some(lead) = schema.arrows.get(&equation.lhs.0[0]) else {
            continue;
        };
        let n_rows = built.rows().get(&lead.src).map(Vec::len).unwrap_or(0);
        for row in 0..n_rows {
            let lhs = naive_compose(built, &equation.lhs, row);
            let rhs = naive_compose(built, &equation.rhs, row);
            if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                if lhs != rhs {
                    out.push(Finding {
                        label: match equation.status {
                            ConstraintStatus::Accepted => EpistemicLabel::Executed,
                            ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
                        },
                        kind: FindingKind::Violation(EquationWitness {
                            equation: index,
                            row: RowId(row),
                            lhs,
                            rhs,
                        }),
                    });
                }
            }
        }
    }
    out.sort();
    out
}

fn naive_compose(built: &TypedInstance, path: &Path, start: usize) -> Option<RowId> {
    let mut current = start;
    for name in &path.0 {
        let cells = built.tables().get(name)?;
        let cell = cells.get(current)?;
        current = (*cell)?.0;
    }
    Some(RowId(current))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The findings match the spec recomputation, structurally, over
    /// arbitrary well-typed models (self-loop arrows included — drawn
    /// shapes carry src == tgt arrows and are evaluated like any arrow).
    #[test]
    fn findings_match_the_contract_recomputation(case in arb_case()) {
        let (schema, instance) = assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        prop_assert_eq!(
            check(&schema, &built),
            expected_findings(&schema, &built),
            "check matches the independent recomputation (self-loops included)"
        );
    }

    /// Every finding's witnesses stay in scope: dangling names a declared
    /// arrow and a row of its source object; violations name a valid
    /// equation index, a row of the equation's domain, and composed
    /// values — never witnesses without rows. Labels are epistemically
    /// honest: only executed/hypothesized, mirroring equation status; the
    /// analysis kinds have no producer in this crate.
    #[test]
    fn every_finding_carries_in_scope_witnesses(case in arb_case()) {
        let (schema, instance) = assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        for finding in check(&schema, &built) {
            prop_assert!(
                matches!(finding.label, EpistemicLabel::Executed | EpistemicLabel::Hypothesized),
                "kernel labels are executed/hypothesized only, got {:?}",
                finding.label
            );
            match finding.kind {
                FindingKind::Dangling(w) => {
                    prop_assert_eq!(finding.label, EpistemicLabel::Executed, "dangling is executed");
                    let cells = built.tables().get(&w.arrow);
                    prop_assert!(cells.is_some(), "dangling arrow `{w:?}` declared");
                    let n = built.rows().get(&schema.arrows[&w.arrow].src).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.src.0 < n, "dangling src row in the arrow's source space");
                }
                FindingKind::Violation(w) => {
                    prop_assert!(w.equation < schema.equations.len(), "equation index valid");
                    let eq = &schema.equations[w.equation];
                    prop_assert_eq!(
                        finding.label == EpistemicLabel::Hypothesized,
                        eq.status == ConstraintStatus::Proposed,
                        "the label mirrors the equation's status"
                    );
                    let lead = &schema.arrows[&eq.lhs.0[0]];
                    let n = built.rows().get(&lead.src).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.row.0 < n, "row is a row of the equation's domain object");
                    let tgt_space = built.rows().get(&schema.arrows[&eq.lhs.0[eq.lhs.0.len() - 1]].tgt).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.lhs.0 < tgt_space && w.rhs.0 < tgt_space, "composed values are rows of the path's target");
                }
                FindingKind::Redundant(_)
                | FindingKind::Contradiction(_)
                | FindingKind::Underconstraint(_) => {
                    prop_assert!(
                        false,
                        "the analysis kinds have no kernel producer (P4's), got {:?}",
                        finding.kind
                    );
                }
            }
        }
    }

    /// A declared schema over an instance with zero rows evaluates
    /// nothing: zero findings even when every arrow and equation is
    /// populated.
    #[test]
    fn zero_row_instance_is_silent(shapes in proptest::collection::vec((0usize..3, 0usize..3), 1..=4)) {
        let case = build_case(&shapes, &[0, 0, 0], &[0; 4], &[], &[]);
        let (schema, instance) = assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        prop_assert_eq!(check(&schema, &built), vec![], "zero rows — zero evaluations, zero findings");
    }
}

/// Totality: the empty document — empty schema, empty instance — yields
/// zero findings. Silence is real success, not an unevaluated
/// declaration.
#[test]
fn empty_schema_and_empty_instance_are_silent() {
    let schema = Schema::default();
    let instance = Instance::default();
    let built = TypedInstance::build(&schema, &instance);
    assert_eq!(
        check(&schema, &built),
        vec![],
        "the empty document checks to zero findings"
    );
}

/// Self-loops are legal and evaluated like any arrow: a self-loop in the
/// middle of a path composes through its stored cell — a dangling cell in
/// the self-loop is reported and short-circuits its row, and when the
/// self-loop routes a row's two equation sides to different rows, the
/// difference is a witnessed violation on that row.
#[test]
fn self_loop_arrows_are_legal_and_evaluated() {
    // Objects A, B; arrows: a: A→A (self-loop), f: A→B, g: A→B; equation
    // f∘a = f (accepted). Endpoints: f∘a: A→A→B = A→B, f: A→B.
    let mut objects = BTreeMap::new();
    for name in ["A", "B"] {
        objects.insert(Ident(name.into()), ());
    }
    let mut arrows = BTreeMap::new();
    arrows.insert(
        Ident("a".into()),
        Arrow {
            src: Ident("A".into()),
            tgt: Ident("A".into()),
        },
    );
    arrows.insert(
        Ident("f".into()),
        Arrow {
            src: Ident("A".into()),
            tgt: Ident("B".into()),
        },
    );
    let equations = vec![Equation {
        lhs: Path(vec![Ident("f".into()), Ident("a".into())]),
        rhs: Path(vec![Ident("f".into())]),
        rationale: "through the self-loop".into(),
        status: ConstraintStatus::Accepted,
    }];

    // Case 1: the self-loop leaves row 0 dangling and maps row 1 to
    // itself; f maps both rows to B's only row. Row 0 cannot compose
    // (dangling, reported); row 1 composes equal (silent).
    arrows.insert(
        Ident("g".into()),
        Arrow {
            src: Ident("A".into()),
            tgt: Ident("B".into()),
        },
    );
    let schema = Schema {
        objects,
        arrows,
        equations,
    };
    let mut rows = BTreeMap::new();
    rows.insert(Ident("A".into()), vec![RowId(0), RowId(1)]);
    rows.insert(Ident("B".into()), vec![RowId(0)]);
    let mut values = BTreeMap::new();
    values.insert(Ident("a".into()), vec![None, Some(RowId(1))]);
    values.insert(Ident("f".into()), vec![Some(RowId(0)), Some(RowId(0))]);
    values.insert(Ident("g".into()), vec![Some(RowId(0)), Some(RowId(0))]);
    let built = TypedInstance::build(&schema, &Instance { rows, values });
    assert_eq!(
        check(&schema, &built),
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Dangling(DanglingWitness {
                arrow: Ident("a".into()),
                src: RowId(0),
            }),
        }],
        "the self-loop composes like any arrow; its dangling cell speaks, the run continues"
    );

    // Case 2: two rows in B; the self-loop maps row 1 away to row 0 and
    // f maps row 1 to B's second row — row 1's two routes disagree, and
    // the violation witnesses the composition through the self-loop.
    let mut rows = BTreeMap::new();
    rows.insert(Ident("A".into()), vec![RowId(0), RowId(1)]);
    rows.insert(Ident("B".into()), vec![RowId(0), RowId(1)]);
    let mut values = BTreeMap::new();
    values.insert(Ident("a".into()), vec![Some(RowId(0)), Some(RowId(0))]);
    values.insert(Ident("f".into()), vec![Some(RowId(0)), Some(RowId(1))]);
    values.insert(Ident("g".into()), vec![Some(RowId(0)), Some(RowId(1))]);
    let built = TypedInstance::build(&schema, &Instance { rows, values });
    // Row 0: f(a(0)) = f(0) = 0 vs f(0) = 0 — silent. Row 1:
    // f(a(1)) = f(0) = 0 vs f(1) = 1 — violation.
    assert_eq!(
        check(&schema, &built),
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(EquationWitness {
                equation: 0,
                row: RowId(1),
                lhs: RowId(0), // f(a(1)) = f(0) = 0
                rhs: RowId(1), // f(1) = 1
            }),
        }],
        "the self-loop's composition difference is a witnessed violation on row 1"
    );
}
