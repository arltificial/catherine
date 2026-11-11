//! Purpose: the findings-taxonomy golden fixtures (openspec change
//! `add-findings-taxonomy`, RED tasks 1–2) — per finding kind, the exact
//! expected [`Finding`] (kind + witness fields + label) and its
//! serialized JSON shape, asserted structurally through parsed
//! `serde_json::Value` fields, never snapshots. Kernel-run fixtures ride
//! the real pipeline (parse → build → `check`); the analysis-kind
//! fixtures (redundant, contradiction, underconstraint) are constructed
//! directly — their producers are P4's and deliberately do not exist yet,
//! so they pin taxonomy shape only. Every constructed finding names real
//! ids from its document (no finding without a witness).

use catherine_core::checks::check;
use catherine_core::findings::{
    ContradictionWitness, DanglingWitness, DomainQuestion, EpistemicLabel, EquationWitness,
    Finding, FindingKind, RedundantWitness, UnderconstraintWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Path, RowId};
use catherine_core::parse::parse;

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn read_fixture(file: &str) -> String {
    std::fs::read_to_string(format!("{GOLDEN_DIR}/{file}")).expect("fixture exists")
}

/// The full pipeline for a fixture P0's parser accepts.
fn check_parsed(file: &str) -> Vec<Finding> {
    let text = read_fixture(file);
    let model = parse(&text).expect("fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    check(&model.schema, &built)
}

/// The findings list serialized to its pinned JSON form.
fn json_of(findings: &[Finding]) -> serde_json::Value {
    serde_json::to_value(findings).expect("findings serialize")
}

/// The serde round-trip assertion: serialize → deserialize → equal.
fn round_trips(findings: &[Finding]) {
    let text = serde_json::to_string(findings).expect("findings serialize");
    let back: Vec<Finding> = serde_json::from_str(&text).expect("findings deserialize");
    assert_eq!(findings, back.as_slice(), "serde round-trip is lossless");
}

/// Task 1 — violation on an accepted equation, from a real kernel run:
/// exactly one finding, `executed` label, witness equation + row + both
/// composed path values. The equation covers two rows; the consistent row
/// emits nothing — one finding per violating row, not one per equation.
#[test]
fn violation_from_a_real_kernel_run_carries_label_and_witness() {
    let findings = check_parsed("violation_accepted.json");
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(EquationWitness {
                equation: 0,
                row: RowId(1),
                lhs: RowId(0),
                rhs: RowId(1),
            }),
        }],
        "expected one executed violation naming equation 0, row a2 (RowId(1)), \
         lhs=b1 rhs=b2"
    );
    // The pinned JSON shape, asserted through parsed fields — no snapshot.
    let json = json_of(&findings);
    assert_eq!(json[0]["label"].as_str(), Some("executed"));
    assert_eq!(json[0]["kind"].as_str(), Some("violation"));
    assert_eq!(json[0]["witness"]["equation"].as_u64(), Some(0));
    assert_eq!(json[0]["witness"]["row"].as_u64(), Some(1));
    assert_eq!(json[0]["witness"]["lhs"].as_u64(), Some(0));
    assert_eq!(json[0]["witness"]["rhs"].as_u64(), Some(1));
    round_trips(&findings);
}

/// Task 1 — a dangling arrow value from a real kernel run: an `executed`
/// dangling finding naming arrow + source row; the dangling row is never
/// a violation; the run continues and the other rows' failures still
/// surface. List order is the taxonomy's total order: violations before
/// dangling (kind declaration order), then witness ids.
#[test]
fn dangling_from_a_real_kernel_run_is_recorded_and_the_run_continues() {
    let findings = check_parsed("dangling_value.json");
    assert_eq!(
        findings,
        vec![
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(2),
                    lhs: RowId(0),
                    rhs: RowId(1),
                }),
            },
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Dangling(DanglingWitness {
                    arrow: Ident("f".into()),
                    src: RowId(0),
                }),
            },
        ],
        "expected the executed violation (equation 0, row a3, lhs=b1 rhs=b2) before the \
         executed dangling (f@a1) — kind order, then witness ids; the dangling row emits \
         no violation, the others are still checked"
    );
    assert!(
        findings
            .iter()
            .all(|f| matches!(f.kind, FindingKind::Violation(_) | FindingKind::Dangling(_))),
        "a dangling value is its own kind — never a violation"
    );
    let json = json_of(&findings);
    assert_eq!(json[1]["label"].as_str(), Some("executed"));
    assert_eq!(json[1]["kind"].as_str(), Some("dangling"));
    assert_eq!(json[1]["witness"]["arrow"].as_str(), Some("f"));
    assert_eq!(json[1]["witness"]["src"].as_u64(), Some(0));
    round_trips(&findings);
}

/// Task 2 — a proposed equation failing on a row, from a real kernel run:
/// a violation-*shaped* finding with the `hypothesized` label — the same
/// kind as the accepted failure beside it, countable apart by label only.
#[test]
fn proposed_failure_is_violation_shaped_and_hypothesized() {
    let findings = check_parsed("proposed_hypothesis.json");
    assert_eq!(
        findings,
        vec![
            Finding {
                label: EpistemicLabel::Hypothesized,
                kind: FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(0),
                    lhs: RowId(0),
                    rhs: RowId(1),
                }),
            },
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Violation(EquationWitness {
                    equation: 1,
                    row: RowId(0),
                    lhs: RowId(0),
                    rhs: RowId(1),
                }),
            },
        ],
        "expected hypothesized (equation 0, proposed) then executed (equation 1, accepted) \
         — violation-shaped findings over the same failing row, countable apart by label"
    );
    let hypothesized = findings
        .iter()
        .filter(|f| f.label == EpistemicLabel::Hypothesized)
        .count();
    let executed = findings
        .iter()
        .filter(|f| f.label == EpistemicLabel::Executed)
        .count();
    assert_eq!((hypothesized, executed), (1, 1), "separately countable");
    let json = json_of(&findings);
    assert_eq!(json[0]["label"].as_str(), Some("hypothesized"));
    assert_eq!(json[0]["kind"].as_str(), Some("violation"));
    round_trips(&findings);
}

/// Task 1 — redundant shape, constructed directly (the P4 producer does
/// not exist): the derivable path + the reducing path, ids taken from
/// redundant_shape.json — f, g, h: A→B with accepted equations f=g, g=h,
/// f=h, where f=h is derivable through the first two.
#[test]
fn redundant_shape_is_pinned_for_p4() {
    let findings = vec![Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Redundant(RedundantWitness {
            derivable: Path(vec![Ident("f".into())]),
            reducing: Path(vec![Ident("h".into())]),
        }),
    }];
    let json = json_of(&findings);
    assert_eq!(json[0]["label"].as_str(), Some("executed"));
    assert_eq!(json[0]["kind"].as_str(), Some("redundant"));
    assert_eq!(
        json[0]["witness"]["derivable"],
        serde_json::json!(["f"]),
        "the derivable path serializes as its arrow-name sequence"
    );
    assert_eq!(json[0]["witness"]["reducing"], serde_json::json!(["h"]));
    round_trips(&findings);
}

/// Task 1 — contradiction shape, constructed directly: the explicit
/// scope — the equations (canonical indices) whose joint enforcement the
/// bounded analysis (P4) found to admit no instance. Ids from
/// contradiction_shape.json's two accepted equations.
#[test]
fn contradiction_shape_is_pinned_for_p4() {
    let findings = vec![Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Contradiction(ContradictionWitness { scope: vec![0, 1] }),
    }];
    let json = json_of(&findings);
    assert_eq!(json[0]["label"].as_str(), Some("executed"));
    assert_eq!(json[0]["kind"].as_str(), Some("contradiction"));
    assert_eq!(
        json[0]["witness"]["scope"],
        serde_json::json!([0, 1]),
        "the explicit scope serializes as canonical equation indices"
    );
    round_trips(&findings);
}

/// Tasks 1 + 7 — underconstraint shape, constructed directly: both
/// witness arms pinned — `ParallelPaths{lhs, rhs}` and
/// `UncoveredInputs{class, examples}` (the latter reserved for P8's
/// rule-lens gap mapping; defined here, never produced in v1) — each with
/// its domain-question text and the `hypothesized` label. Ids from
/// underconstraint_shape.json (parallel arrows f, g: A→B; no equation
/// forces them to agree; rows a1, a2 of A).
#[test]
fn underconstraint_shape_pins_both_witness_arms() {
    let parallel = Finding {
        label: EpistemicLabel::Hypothesized,
        kind: FindingKind::Underconstraint(UnderconstraintWitness {
            question: "must the parallel paths f and g agree on every row?".into(),
            case: DomainQuestion::ParallelPaths {
                lhs: Path(vec![Ident("f".into())]),
                rhs: Path(vec![Ident("g".into())]),
            },
        }),
    };
    let uncovered = Finding {
        label: EpistemicLabel::Hypothesized,
        kind: FindingKind::Underconstraint(UnderconstraintWitness {
            question: "which rows of A does no equation cover?".into(),
            case: DomainQuestion::UncoveredInputs {
                class: "rows of A with no applicable equation".into(),
                examples: vec![RowId(0), RowId(1)],
            },
        }),
    };
    let json = json_of(&[parallel.clone(), uncovered.clone()]);
    assert_eq!(json[0]["label"].as_str(), Some("hypothesized"));
    assert_eq!(json[0]["kind"].as_str(), Some("underconstraint"));
    assert_eq!(
        json[0]["witness"]["question"].as_str(),
        Some("must the parallel paths f and g agree on every row?")
    );
    assert_eq!(json[0]["witness"]["case"].as_str(), Some("parallel_paths"));
    assert_eq!(
        json[0]["witness"]["detail"]["lhs"],
        serde_json::json!(["f"])
    );
    assert_eq!(
        json[0]["witness"]["detail"]["rhs"],
        serde_json::json!(["g"])
    );
    assert_eq!(json[1]["label"].as_str(), Some("hypothesized"));
    assert_eq!(
        json[1]["witness"]["case"].as_str(),
        Some("uncovered_inputs")
    );
    assert_eq!(
        json[1]["witness"]["detail"]["class"].as_str(),
        Some("rows of A with no applicable equation")
    );
    assert_eq!(
        json[1]["witness"]["detail"]["examples"],
        serde_json::json!([0, 1])
    );
    round_trips(&[parallel, uncovered]);
    // The order rule (kind, then witness ids) puts the structural
    // witness ahead of the free-form question text within the kind: for
    // two underconstraint findings, the tagged case decides before the
    // question text does.
    let zzz_parallel = Finding {
        label: EpistemicLabel::Hypothesized,
        kind: FindingKind::Underconstraint(UnderconstraintWitness {
            question: "zzz".into(),
            case: DomainQuestion::ParallelPaths {
                lhs: Path(vec![Ident("f".into())]),
                rhs: Path(vec![Ident("g".into())]),
            },
        }),
    };
    let aaa_uncovered = Finding {
        label: EpistemicLabel::Hypothesized,
        kind: FindingKind::Underconstraint(UnderconstraintWitness {
            question: "aaa".into(),
            case: DomainQuestion::UncoveredInputs {
                class: "c".into(),
                examples: vec![RowId(0)],
            },
        }),
    };
    assert!(
        zzz_parallel < aaa_uncovered,
        "the tagged case orders before the question text (witness ids first)"
    );
}

/// Task 6 — repeated checks of the same model are byte-identical:
/// independent pipelines (fresh parse, fresh build, separate `check`
/// invocations) serialize identically across every kernel-run fixture.
#[test]
fn repeated_checks_are_byte_identical() {
    for file in [
        "violation_accepted.json",
        "dangling_value.json",
        "proposed_hypothesis.json",
        "consistent.json",
    ] {
        let first = serde_json::to_string(&check_parsed(file)).expect("serialize");
        let second = serde_json::to_string(&check_parsed(file)).expect("serialize");
        assert_eq!(
            first, second,
            "[{file}] repeated checks serialize byte-identically"
        );
    }
}
