//! The five analysis-fixture RED-task tests plus determinism assertions.

use catherine_analysis::bounds::{ScopeCaps, SearchBounds};
use catherine_analysis::distinguish::{distinguish, DistinguishOutcome, Observation};
use catherine_analysis::parallel::underconstraint;
use catherine_analysis::rewrite::redundant;
use catherine_analysis::search::{search, SearchOutcome};
use catherine_core::findings::{EpistemicLabel, FindingKind};
use catherine_core::instance::TypedInstance;
use catherine_core::model::RowId;

use super::builders::{
    contradiction_schema, distinguish_pair, parallel_schema, path, rewrite_schema, rows,
    transitive_chain_schema,
};
use crate::common::ident;

// ── RED task 1: over-constrained → contradiction within bounds ──────

#[test]
fn task1_over_constrained_schema_exhausts_to_contradiction() {
    let schema = contradiction_schema();
    // `b` has no rows under the bound: `f` cannot resolve, so no fully
    // resolved instance exists. The space (8 candidates) exhausts well
    // within the budget.
    let bound = SearchBounds {
        max_rows: rows(&[("a", 1), ("b", 0), ("c", 1)]),
        max_candidates: 64,
    };
    let report = search(&schema, &bound);
    // The bound is restated on the outcome (spec scenario).
    assert_eq!(
        report.bound.max_rows.get(&ident("b")),
        Some(&0),
        "the bound is restated with the explored shape"
    );
    match &report.outcome {
        SearchOutcome::Contradiction { finding } => {
            assert_eq!(
                finding.label,
                EpistemicLabel::Executed,
                "contradiction-by-completed-exhaustion is a kernel-computed proof"
            );
            match &finding.kind {
                FindingKind::Contradiction(witness) => {
                    assert_eq!(
                        witness.scope,
                        vec![0, 1],
                        "the scope names the accepted equations, canonical indices in order"
                    );
                }
                other => panic!("contradiction kind expected, got {other:?}"),
            }
        }
        other => panic!("contradiction expected after real exhaustion, got {other:?}"),
    }
}

#[test]
fn task1_the_same_schema_admits_a_witness_once_the_bound_frees_b() {
    // Sanity anchor: the contradiction comes from the bounds (exhaustion
    // over a real space), not from a broken engine — with `b` nonempty
    // the smallest total instance (everything → row 0) witnesses.
    let schema = contradiction_schema();
    let bound = SearchBounds {
        max_rows: rows(&[("a", 1), ("b", 1), ("c", 1)]),
        max_candidates: 64,
    };
    let report = search(&schema, &bound);
    match &report.outcome {
        SearchOutcome::Witness { instance } => {
            assert_eq!(
                instance.rows.get(&ident("b")).map(Vec::len),
                Some(1),
                "the witness lives inside the declared bounds"
            );
        }
        other => panic!("witness expected within the freed bound, got {other:?}"),
    }
}

// ── RED task 2: budget smaller than the space → UNKNOWN with scope ──

#[test]
fn task2_small_budget_yields_unknown_with_the_explored_scope() {
    let schema = contradiction_schema();
    // The same shape as task 1 (an 8-candidate space), budget 2: the
    // enumeration cannot complete, so the answer is UNKNOWN with the
    // exact explored scope — never a contradiction claim, never a naked
    // "no violation found".
    let bound = SearchBounds {
        max_rows: rows(&[("a", 1), ("b", 0), ("c", 1)]),
        max_candidates: 2,
    };
    let report = search(&schema, &bound);
    match &report.outcome {
        SearchOutcome::Unknown { scope } => {
            assert_eq!(
                scope.candidates_explored, 2,
                "the scope records exactly what was explored"
            );
            assert_eq!(
                scope.rows.get(&ident("a")),
                Some(&1),
                "the scope names the explored row bounds"
            );
            assert_eq!(
                scope.rows.get(&ident("b")),
                None,
                "b's empty space is not explored"
            );
            assert_eq!(scope.rows.get(&ident("c")), Some(&1));
        }
        other => panic!("UNKNOWN expected under the small budget, got {other:?}"),
    }
}

#[test]
fn task2_zero_budget_yields_unknown_before_any_enumeration() {
    // Budget 0 on a *satisfiable* schema: the answer is still UNKNOWN
    // with zero candidates explored — never a witness smuggled out from
    // an enumeration that did not run, never "no violation found".
    let schema = contradiction_schema();
    let bound = SearchBounds {
        max_rows: rows(&[("a", 1), ("b", 1), ("c", 1)]),
        max_candidates: 0,
    };
    let report = search(&schema, &bound);
    match &report.outcome {
        SearchOutcome::Unknown { scope } => {
            assert_eq!(scope.candidates_explored, 0);
            assert_eq!(
                scope.rows.get(&ident("a")),
                Some(&1),
                "the shape is declared"
            );
        }
        other => panic!("budget 0 is UNKNOWN, got {other:?}"),
    }
}

#[test]
fn task2_zero_size_bound_yields_unknown_with_the_empty_scope() {
    let schema = contradiction_schema();
    let bound = SearchBounds {
        max_rows: rows(&[("a", 0), ("b", 0), ("c", 0)]),
        max_candidates: 64,
    };
    let report = search(&schema, &bound);
    match &report.outcome {
        SearchOutcome::Unknown { scope } => {
            assert!(
                scope.rows.is_empty(),
                "a zero-size bound explores the empty scope"
            );
            assert_eq!(scope.candidates_explored, 0, "nothing was explored");
        }
        other => panic!(
            "zero-size bounds are UNKNOWN with the empty scope, never a contradiction: {other:?}"
        ),
    }
}

// ── RED task 3: parallel paths, no equation → hypothesized question ──

#[test]
fn task3_parallel_paths_raise_the_hypothesized_domain_question() {
    let findings = underconstraint(&parallel_schema(), &ScopeCaps::default());
    assert_eq!(findings.len(), 1, "one parallel pair within the cap");
    let finding = &findings[0];
    assert_eq!(
        finding.label,
        EpistemicLabel::Hypothesized,
        "underconstraint is a question, never a violation"
    );
    let serialized = serde_json::to_value(finding).expect("findings serialize");
    assert_eq!(serialized["kind"], "underconstraint");
    assert_eq!(serialized["label"], "hypothesized");
    assert_eq!(
        serialized["witness"]["case"], "parallel_paths",
        "the flattened domain-question shape: case first"
    );
    assert_eq!(
        serialized["witness"]["detail"]["lhs"],
        serde_json::json!(["f", "g"]),
        "the longer composite sorts first"
    );
    assert_eq!(
        serialized["witness"]["detail"]["rhs"],
        serde_json::json!(["h"])
    );
    assert_eq!(
        serialized["witness"]["question"], "should these agree?",
        "the domain question, text last"
    );
}

#[test]
fn task3_zero_cap_enumerates_no_paths() {
    // A cap of 0 bounds the op from the start: no paths, no hypotheses —
    // a cap hit is a bounded non-answer, never a quiet truncation.
    let findings = underconstraint(
        &parallel_schema(),
        &ScopeCaps {
            max_path_length: 0,
            ..ScopeCaps::default()
        },
    );
    assert!(findings.is_empty(), "cap 0 enumerates nothing");
}

#[test]
fn transitive_equation_chain_does_not_suppress_the_end_pair_question() {
    // catherine-7e8, documented limit: `related` is direct-only — the
    // accepted chain p=q, q=r proves p=r by rewriting, but does not
    // suppress the (p,r) domain question. Question-raiser v1 semantics:
    // noise over silence; congruence closure is future capability.
    let findings = underconstraint(&transitive_chain_schema(), &ScopeCaps::default());
    assert_eq!(
        findings.len(),
        1,
        "only the directly-unrelated (p,r) pair is raised"
    );
    let serialized = serde_json::to_value(&findings[0]).expect("findings serialize");
    assert_eq!(
        serialized["witness"]["detail"]["lhs"],
        serde_json::json!(["p"])
    );
    assert_eq!(
        serialized["witness"]["detail"]["rhs"],
        serde_json::json!(["r"])
    );
    assert_eq!(serialized["witness"]["question"], "should these agree?");
}

// ── RED task 4: path derivable under the declared equations ─────────

#[test]
fn task4_derivable_equation_is_proven_redundant_with_both_paths() {
    let findings = redundant(&rewrite_schema(), &ScopeCaps::default());
    assert_eq!(
        findings.len(),
        1,
        "exactly e2 is derivable; e0 and e1 are not proven redundant"
    );
    let finding = &findings[0];
    assert_eq!(
        finding.label,
        EpistemicLabel::Executed,
        "redundant-by-derivation is a kernel-computed proof"
    );
    match &finding.kind {
        FindingKind::Redundant(witness) => {
            assert_eq!(
                witness.derivable,
                path(&["f", "g", "k"]),
                "the redundant equation side (e2's lhs)"
            );
            assert_eq!(
                witness.reducing,
                path(&["n"]),
                "the path it reduces to under the other equations"
            );
        }
        other => panic!("redundant kind expected, got {other:?}"),
    }
}

// ── RED task 5: distinguish over an explicit observation set ─────────

#[test]
fn task5_differing_observation_is_named_with_both_values() {
    let (schema, instance_a, instance_b) = distinguish_pair();
    let typed_a = TypedInstance::build(&schema, &instance_a);
    let typed_b = TypedInstance::build(&schema, &instance_b);
    let observations = vec![Observation {
        path: path(&["f"]),
        object: ident("a"),
        row: RowId(0),
    }];
    let report = distinguish(&typed_a, &typed_b, &observations);
    // The observation set is part of the result.
    assert_eq!(report.observations, observations);
    match &report.outcome {
        DistinguishOutcome::Distinguishable {
            observation,
            left,
            right,
        } => {
            assert_eq!(observation, &observations[0]);
            assert_eq!(*left, Some(RowId(0)), "instance a sends a#0 to b#0");
            assert_eq!(*right, Some(RowId(1)), "instance b sends a#0 to b#1");
        }
        other => panic!("distinguishable expected, got {other:?}"),
    }
}

#[test]
fn task5_restricted_observation_set_yields_equivalent_in_scope() {
    let (schema, instance_a, instance_b) = distinguish_pair();
    let typed_a = TypedInstance::build(&schema, &instance_a);
    let typed_b = TypedInstance::build(&schema, &instance_b);
    let observations = vec![Observation {
        path: path(&["g"]),
        object: ident("a"),
        row: RowId(0),
    }];
    let report = distinguish(&typed_a, &typed_b, &observations);
    assert_eq!(report.observations, observations, "the set is echoed");
    assert_eq!(
        report.outcome,
        DistinguishOutcome::EquivalentInScope,
        "the pair agrees on the restricted set — in scope, and no claim beyond it"
    );
}

// ── determinism: repeated runs serialize byte-identically ────────────

#[test]
fn emitted_findings_serialize_byte_identically_across_repeated_runs() {
    let redundant_findings = redundant(&rewrite_schema(), &ScopeCaps::default());
    let underconstraint_findings = underconstraint(&parallel_schema(), &ScopeCaps::default());
    let first = serde_json::to_string(&redundant_findings).expect("redundant serializes")
        + &serde_json::to_string(&underconstraint_findings).expect("underconstraint serializes");
    let second = serde_json::to_string(&redundant(&rewrite_schema(), &ScopeCaps::default()))
        .expect("serializes")
        + &serde_json::to_string(&underconstraint(&parallel_schema(), &ScopeCaps::default()))
            .expect("serializes");
    assert_eq!(first, second, "repeated runs serialize byte-identically");
    // The taxonomy's total order holds through the new producers: within
    // one kind the witness ids order the findings.
    let mut sorted = underconstraint_findings.clone();
    sorted.sort();
    assert_eq!(
        sorted, underconstraint_findings,
        "producers emit sorted order"
    );
}

#[test]
fn search_runs_are_deterministic() {
    let schema = contradiction_schema();
    let bound = SearchBounds {
        max_rows: rows(&[("a", 1), ("b", 1), ("c", 1)]),
        max_candidates: 64,
    };
    let first = search(&schema, &bound);
    let second = search(&schema, &bound);
    assert_eq!(first, second, "repeated searches produce identical reports");
}
