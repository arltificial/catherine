//! Purpose: the findings-taxonomy properties (proptest, never snapshots;
//! openspec change `add-findings-taxonomy`, task 6) — serde JSON
//! round-trip losslessness, byte-identical serialization across repeated
//! checks, the total order holding on check output, and kernel
//! label/kind soundness over arbitrary well-typed models, plus every
//! finding kind (analysis kinds included — representable values, though
//! this crate produces none) round-tripping from arbitrary witnesses.
//! Assertions are structural and by parsed value — never byte snapshots.

mod common;

use common::fixture::{self, FixtureCase};

use catherine_core::checks::check;
use catherine_core::findings::{
    ContradictionWitness, DanglingWitness, DomainQuestion, EpistemicLabel, EquationWitness,
    Finding, FindingKind, RedundantWitness, UnderconstraintWitness, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Ident, Path, RowId};
use proptest::prelude::*;

type Case = FixtureCase;

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
            fixture::build_case(&shapes, &row_counts, &cell_pool, &eq_seeds, &statuses)
        })
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
            FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: e,
                row: RowId(r),
                lhs: RowId(l),
                rhs: RowId(rh),
            }))
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
        let (schema, instance) = fixture::assemble(&case);
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
        let (schema, instance) = fixture::assemble(&case);
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
                FindingKind::Violation(ViolationWitness::Equation(w)) => {
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

                FindingKind::Violation(ViolationWitness::Key(w)) => {
                    prop_assert!(w.key < schema.keys.len(), "key index valid");
                    let key = &schema.keys[w.key];
                    let n = built.rows().get(&key.object).map(Vec::len).unwrap_or(0);
                    prop_assert!(
                        w.rows[0].0 < n && w.rows[1].0 < n,
                        "witness rows are rows of the key's object"
                    );
                    prop_assert!(w.rows[0] < w.rows[1], "witness rows ascending");
                    prop_assert_eq!(
                        finding.label == EpistemicLabel::Hypothesized,
                        key.status == ConstraintStatus::Proposed,
                        "the label mirrors the key's status"
                    );
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
