//! Purpose: the findings-taxonomy properties (proptest, never snapshots;
//! openspec change `add-findings-taxonomy`, task 6) — serde JSON
//! round-trip losslessness, byte-identical serialization across repeated
//! checks, the total order holding on check output, and kernel
//! label/kind soundness over arbitrary well-typed models, plus every
//! finding kind (analysis kinds included — representable values, though
//! this crate produces none) round-tripping from arbitrary witnesses.
//! Assertions are structural and by parsed value — never byte snapshots.

use catherine_core::checks::check;
use catherine_core::findings::{
    ContradictionWitness, DanglingWitness, DomainQuestion, EpistemicLabel, EquationWitness,
    Finding, FindingKind, RedundantWitness, UnderconstraintWitness,
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
    equations: Vec<Equation>,       // deterministic insertion order
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
            let n_cells = shapes
                .iter()
                .map(|&(s, _)| row_counts[s])
                .sum::<usize>()
                .max(1);
            (
                Just(shapes),
                Just(row_counts),
                proptest::collection::vec(0usize..=8usize, n_cells),
                proptest::collection::vec((0usize..=7usize, 0usize..=7usize), 0..=3),
                proptest::collection::vec(any::<bool>(), 0..=3),
            )
        })
        .prop_map(|(shapes, row_counts, cell_pool, eq_seeds, statuses)| {
            build_case(&shapes, &row_counts, &cell_pool, &eq_seeds, &statuses)
        })
}

/// Build the case from drawn parts: well-typed tables (a drawn value is
/// clamped into the target object's row space, an empty target space
/// leaves the cell dangling), equations over endpoint-matched candidate
/// paths, in deterministic order.
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
                Some(v) if tgt_rows > 0 => Some(v % tgt_rows),
                _ => None,
            })
            .collect::<Vec<_>>();
        cells.push(arrow_cells);
    }

    // Candidate paths between an endpoint pair (s, t): 1-hop arrows and
    // 2-hop compositions through an intermediate object, grouped by
    // endpoints so lhs/rhs are always well-typed.
    let mut by_endpoints: BTreeMap<(usize, usize), Vec<Vec<usize>>> = BTreeMap::new();
    let n = shapes.len().min(row_counts.len());
    for s in 0..n {
        for t in 0..n {
            let mut matching = Vec::new();
            for (i, &(src, tgt)) in shapes.iter().enumerate() {
                if src == s && tgt == t {
                    matching.push(vec![i]);
                }
                if src == s {
                    for (j, &(src2, tgt2)) in shapes.iter().enumerate() {
                        if i != j && src2 == tgt && tgt2 == t {
                            matching.push(vec![i, j]);
                        }
                    }
                }
            }
            if !matching.is_empty() {
                by_endpoints.insert((s, t), matching);
            }
        }
    }
    let candidates: Vec<&Vec<Vec<usize>>> = by_endpoints.values().collect();
    let mut equations = Vec::new();
    for (k, &(l, r)) in eq_seeds.iter().enumerate() {
        if candidates.is_empty() {
            break;
        }
        let group = candidates[(l + r + k) % candidates.len()];
        equations.push(Equation {
            lhs: Path(path_idents(&group[l % group.len()])),
            rhs: Path(path_idents(&group[r % group.len()])),
            rationale: format!("property equation {k}"),
            status: if statuses.get(k).copied().unwrap_or(false) {
                ConstraintStatus::Proposed
            } else {
                ConstraintStatus::Accepted
            },
        });
    }

    Case {
        arrows: shapes.to_vec(),
        row_counts: row_counts.to_vec(),
        cells,
        equations,
    }
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

fn arb_label() -> impl Strategy<Value = EpistemicLabel> {
    prop_oneof![
        Just(EpistemicLabel::Executed),
        Just(EpistemicLabel::Hypothesized),
        Just(EpistemicLabel::Unknown),
    ]
}

fn arb_ident() -> impl Strategy<Value = Ident> {
    ("[a-z][a-z0-9_]{0,7}", 0usize..8usize).prop_map(|(stem, n)| Ident(format!("{stem}{n}")))
}

fn arb_path() -> impl Strategy<Value = Path> {
    proptest::collection::vec(arb_ident(), 1..=3).prop_map(Path)
}

fn arb_domain_question() -> impl Strategy<Value = DomainQuestion> {
    prop_oneof![
        (arb_path(), arb_path()).prop_map(|(lhs, rhs)| DomainQuestion::ParallelPaths { lhs, rhs }),
        (
            "[a-z ]{0,20}",
            proptest::collection::vec(0usize..8usize, 0..=4)
        )
            .prop_map(|(class, examples)| DomainQuestion::UncoveredInputs {
                class,
                examples: examples.into_iter().map(RowId).collect()
            }),
    ]
}

/// Arbitrary findings across all five kinds and all three labels —
/// including the analysis kinds, which are representable values even
/// though this crate's kernel never produces them.
fn arb_finding() -> impl Strategy<Value = Finding> {
    let kind = prop_oneof![
        (0usize..8, 0usize..8, 0usize..8, 0usize..8).prop_map(|(e, r, l, rh)| {
            FindingKind::Violation(EquationWitness {
                equation: e,
                row: RowId(r),
                lhs: RowId(l),
                rhs: RowId(rh),
            })
        }),
        (arb_ident(), 0usize..8).prop_map(|(a, s)| FindingKind::Dangling(DanglingWitness {
            arrow: a,
            src: RowId(s),
        })),
        (arb_path(), arb_path()).prop_map(|(d, r)| FindingKind::Redundant(RedundantWitness {
            derivable: d,
            reducing: r,
        })),
        proptest::collection::vec(0usize..8usize, 0..=4)
            .prop_map(|scope| FindingKind::Contradiction(ContradictionWitness { scope })),
        (".{0,40}", arb_domain_question()).prop_map(|(q, case)| FindingKind::Underconstraint(
            UnderconstraintWitness { question: q, case }
        )),
    ];
    (arb_label(), kind).prop_map(|(label, kind)| Finding { label, kind })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Findings round-trip through serde JSON losslessly, and repeated
    /// checks — separate invocations over fresh builds — serialize
    /// byte-identically; the emitted list is in the taxonomy's total
    /// order (kind, then witness ids).
    #[test]
    fn findings_round_trip_and_serialize_stably(case in arb_case()) {
        let (schema, instance) = assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        let findings = check(&schema, &built);
        let first = serde_json::to_string(&findings).expect("findings serialize");
        let rebuilt = TypedInstance::build(&schema, &instance);
        let again = serde_json::to_string(&check(&schema, &rebuilt)).expect("findings serialize");
        prop_assert_eq!(&first, &again, "repeated checks serialize byte-identically");
        let back: Vec<Finding> = serde_json::from_str(&first).expect("findings deserialize");
        prop_assert_eq!(&findings, &back, "serde round-trip is lossless");
        prop_assert!(
            findings.windows(2).all(|w| w[0] <= w[1]),
            "check output is in the taxonomy's total order"
        );
    }

    /// Every arbitrary finding — all five kinds, all three labels —
    /// round-trips through serde JSON losslessly and serializes
    /// deterministically (same value, same bytes, twice).
    #[test]
    fn every_kind_round_trips_and_serializes_deterministically(finding in arb_finding()) {
        let first = serde_json::to_string(&finding).expect("finding serializes");
        let back: Finding = serde_json::from_str(&first).expect("finding deserializes");
        prop_assert_eq!(&finding, &back, "serde round-trip is lossless");
        let second = serde_json::to_string(&finding).expect("finding serializes");
        prop_assert_eq!(&first, &second, "serialization is deterministic");
    }

    /// The total order is kind-first, label last: with different kinds,
    /// swapping the labels leaves the order unchanged (the label never
    /// overrides kind order); with equal kinds — same witnesses —
    /// swapping the labels swaps the order (the label is the tiebreak).
    #[test]
    fn label_is_only_a_tiebreak(a in arb_finding(), b in arb_finding()) {
        let (la, lb) = (a.label, b.label);
        let a2 = Finding {
            label: lb,
            kind: a.kind.clone(),
        };
        let b2 = Finding {
            label: la,
            kind: b.kind.clone(),
        };
        if a.kind != b.kind {
            prop_assert_eq!(
                a.cmp(&b),
                a2.cmp(&b2),
                "with different kinds, the labels do not affect the order"
            );
        } else {
            prop_assert_eq!(
                a2.cmp(&b2),
                b.cmp(&a),
                "with equal kinds, swapping labels swaps the order — the label is the tiebreak"
            );
        }
    }

    /// Kernel findings are label- and kind-sound over arbitrary
    /// well-typed models: labels only executed/hypothesized and mirroring
    /// equation status, witnesses in scope, and none of the analysis
    /// kinds (their producers are P4's).
    #[test]
    fn kernel_findings_are_label_and_kind_sound(case in arb_case()) {
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
                    prop_assert!(built.tables().contains_key(&w.arrow), "dangling arrow declared");
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
                    let tgt = built.rows().get(&schema.arrows[&eq.lhs.0[eq.lhs.0.len() - 1]].tgt).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.lhs.0 < tgt && w.rhs.0 < tgt, "composed values are rows of the path's target");
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
}
