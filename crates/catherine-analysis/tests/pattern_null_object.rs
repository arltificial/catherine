//! Purpose: the Null Object lens's own Meter (beads `catherine-mx1.4`) —
//! runs [`Lens`] directly over every recorded `null-object` row in the
//! checked-in expectation table and asserts the ledger's exact
//! `(kind, label)` multiset across both channels, per fixture and fault.
//!
//! Responsibilities: filter the table by the `null-object` id, load each
//! row's fixture, execute the lens (never the registry), compare the P2
//! checks and the migrate surface's `lost_row`/`lost_cell` findings as one
//! multiset, pin the recorded witnesses on the nil-row and flatten faults,
//! assert the clean arm is silent, and pin the bounded non-answer for a
//! model that declares no equations.
//!
//! Rationale: the ledger is sealed and the table is checked in, so a
//! mismatch is a lens or spec bug, never a table edit. Parity rows are
//! replayed exactly like unique ones — they are reported as parity only,
//! so the assertion is the recorded multiset with no extra finding and no
//! unique-detection claim. The registry and replay harness belong to the
//! orchestrator; this file proves the module on its own, so a wiring
//! mistake cannot hide a lens defect.

#[path = "pattern_expectations.rs"]
mod expectations;

// The production module is compiled by `src/pattern/mod.rs` once the
// orchestrator wires the registry. Until then this test compiles the exact
// same source under the test crate, with the library's `crate::{bounds,
// pattern}` roots re-exported so the module's imports resolve unchanged —
// the behaviour under test is the production file, not a copy.
mod bounds {
    pub use catherine_analysis::bounds::*;
}
mod pattern {
    pub use catherine_analysis::pattern::*;
}
#[path = "../src/pattern/null_object.rs"]
mod null_object;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LawKind, LensFindings, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::{EpistemicLabel, Finding, FindingKind, ViolationWitness};
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::Model;
use expectations::{Expectation, FindingKey, EXPECTATIONS};

/// The `null-object` rows, in table order.
fn null_object_rows() -> Vec<&'static Expectation> {
    EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "null-object")
        .collect()
}

/// A repository-relative fixture path.
fn fixture_path(relative: &str) -> String {
    format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"))
}

/// Load one fixture model.
fn fixture(relative: &str) -> Model {
    let text = std::fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"));
    catherine_core::parse::parse(&text)
        .unwrap_or_else(|error| panic!("{relative} parses: {error:?}"))
}

/// The row's `--values` spellings as the CLI hands them to the lens.
fn values(row: &Expectation) -> Vec<String> {
    row.values
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

/// Execute one row's lens; a completed outcome is required because every
/// recorded row's fixture declares the equation surface.
fn run_row(row: &Expectation) -> LensFindings {
    let model = fixture(row.fixture);
    let bound = values(row);
    let input = LensInput {
        model: &model,
        values: &bound,
    };
    match null_object::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => panic!(
            "{} ({}) must evaluate, got UNKNOWN: {scope}",
            row.fixture, row.fault
        ),
    }
}

/// One check finding as the taxonomy's stable `(kind, label)` spelling.
fn check_key(finding: &Finding) -> FindingKey {
    let kind = match &finding.kind {
        FindingKind::Violation(_) => "violation",
        FindingKind::Dangling(_) => "dangling",
        FindingKind::Redundant(_) => "redundant",
        FindingKind::Contradiction(_) => "contradiction",
        FindingKind::Underconstraint(_) => "underconstraint",
    };
    (kind, label_name(finding.label))
}

/// One migration finding as the migrate surface's stable `(kind, label)`
/// spelling — `lost_row`/`lost_cell`, never a P2 kind.
fn migration_key(finding: &MigrationFinding) -> FindingKey {
    let kind = match &finding.kind {
        MigrationFindingKind::LostRow { .. } => "lost_row",
        MigrationFindingKind::LostCell { .. } => "lost_cell",
    };
    (kind, label_name(finding.label))
}

/// The epistemic label's stable spelling.
fn label_name(label: EpistemicLabel) -> &'static str {
    match label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    }
}

/// The completed outcome's multiset over both channels, sorted.
fn multiset(findings: &LensFindings) -> Vec<FindingKey> {
    let mut keys: Vec<FindingKey> = findings
        .checks
        .iter()
        .map(check_key)
        .chain(findings.migrations.iter().map(migration_key))
        .collect();
    keys.sort_unstable();
    keys
}

/// Assert one recorded row: the lens completes with exactly the ledger's
/// multiset, so no extra finding hides behind a parity row.
fn assert_row(row: &Expectation) {
    assert_eq!(row.class, "finding", "{} is a completed row", row.fault);
    let findings = run_row(row);
    let actual = multiset(&findings);
    let mut expected = row.findings.to_vec();
    expected.sort_unstable();
    assert_eq!(
        actual, expected,
        "{} ({}, parity={}) recorded multiset",
        row.fixture, row.fault, row.parity
    );
}

#[test]
fn the_metadata_is_the_null_object_disposition_row() {
    let meta = null_object::Lens.meta();
    assert_eq!(meta.id, "null-object");
    assert_eq!(meta.name, "Null Object");
    assert_eq!(meta.abstraction, "monoid identity element");
    assert_eq!(meta.law, LawKind::Identity);
    assert_eq!(meta.law.as_str(), "identity");
    assert_eq!(
        meta.scope,
        "nil-identity equations (identity/totality for the nil element)"
    );
}

#[test]
fn the_table_has_the_recorded_null_object_rows() {
    let rows = null_object_rows();
    assert_eq!(rows.len(), 6);
    let faults: Vec<&str> = rows.iter().map(|row| row.fault).collect();
    assert_eq!(
        faults,
        [
            "broken-child-parent-link",
            "nil-treated-as-real-leaf",
            "floor-near-trivial",
            "broken-child-parent-link",
            "nil-treated-as-real-leaf",
            "lost-child-on-flatten",
        ]
    );
}

#[test]
fn every_recorded_row_matches_the_ledger_multiset() {
    let rows = null_object_rows();
    assert!(
        !rows.is_empty(),
        "the ledger records at least one null-object row"
    );
    for row in rows {
        assert_row(row);
    }
}

#[test]
fn the_parity_rows_report_only_the_recorded_multiset() {
    let parity: Vec<&Expectation> = null_object_rows()
        .into_iter()
        .filter(|row| row.parity)
        .collect();
    assert_eq!(
        parity.len(),
        3,
        "the floor control is the recorded parity arm"
    );
    for row in parity {
        assert_row(row);
        let findings = run_row(row);
        assert_eq!(
            multiset(&findings).len(),
            row.findings.len(),
            "{} ({}) adds no unique finding beyond parity",
            row.fixture,
            row.fault
        );
    }
}

#[test]
fn the_floor_control_names_the_nil_row_and_the_child_cell() {
    let row = null_object_rows()
        .into_iter()
        .find(|row| row.fault == "nil-treated-as-real-leaf" && row.parity)
        .expect("floor control row exists");
    let findings = run_row(row);
    assert!(findings.migrations.is_empty(), "the floor has no Δ surface");
    assert_eq!(findings.checks.len(), 2, "one violation and one dangling");
    let violation = findings
        .checks
        .iter()
        .find_map(|finding| match &finding.kind {
            FindingKind::Violation(ViolationWitness::Equation(witness)) => Some(witness),
            _ => None,
        })
        .expect("the nil-identity equation fails");
    assert_eq!(violation.equation, 0);
    assert_eq!(violation.row.0, 1, "the nil row is the failed row");
    assert_ne!(violation.lhs, violation.rhs);
    let dangling = findings
        .checks
        .iter()
        .find_map(|finding| match &finding.kind {
            FindingKind::Dangling(witness) => Some(witness),
            _ => None,
        })
        .expect("the child's parent cell is dangling");
    assert_eq!(dangling.arrow.0, "parent");
    assert_eq!(dangling.src.0, 0, "the child row's parent cell");
}

#[test]
fn the_flatten_fault_loses_every_parent_cell() {
    let row = null_object_rows()
        .into_iter()
        .find(|row| row.fault == "lost-child-on-flatten")
        .expect("flatten row exists");
    let findings = run_row(row);
    assert!(
        findings.checks.is_empty(),
        "the flatten fixture's base model is clean"
    );
    assert_eq!(findings.migrations.len(), 7, "seven valued parent cells");
    for (index, finding) in findings.migrations.iter().enumerate() {
        match &finding.kind {
            MigrationFindingKind::LostCell { arrow, row } => {
                assert_eq!(arrow.0, "parent");
                assert_eq!(row.0, index, "lost cells name rows in order");
            }
            other => panic!("only parent cells are dropped, got {other:?}"),
        }
    }
}

#[test]
fn the_clean_arm_is_silent_but_completed() {
    for relative in [
        "fixtures/katas/composite-null-object/floor/floor-composite-null-object.ct.json",
        "fixtures/katas/composite-null-object/composite-null-object.ct.json",
    ] {
        let model = fixture(relative);
        let input = LensInput {
            model: &model,
            values: &[],
        };
        match null_object::Lens.run(&input, &ScopeCaps::default()) {
            LensOutcome::Findings(findings) => {
                assert!(findings.checks.is_empty(), "{relative} holds the law");
                assert!(
                    findings.migrations.is_empty(),
                    "{relative} has no Δ surface"
                );
            }
            LensOutcome::Unknown { scope } => panic!("{relative} must evaluate: {scope}"),
        }
    }
}

#[test]
fn a_model_without_equations_is_unknown_not_clean() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match null_object::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("equations")),
        LensOutcome::Findings(findings) => {
            panic!("an absent surface must not be clean: {findings:?}")
        }
    }
}
