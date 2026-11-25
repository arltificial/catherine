//! Purpose: the Interpreter lens's own Meter (beads `catherine-mx1.10`) —
//! runs [`Lens`] directly over every recorded `interpreter` row in the
//! checked-in expectation table and asserts the ledger's exact
//! `(kind, label)` multiset, per fixture and fault.
//!
//! Responsibilities: filter the table by the `interpreter` id, load each
//! row's fixture, bind the recorded `--values` spellings, execute the lens
//! (never the registry), compare the checks channel as a multiset, assert
//! the migration channel stays empty, and pin that the killed/parity rows
//! stay silent. Also pins the metadata row and the two bounded
//! non-answers.
//!
//! Rationale: the ledger is sealed and the table is checked in, so a
//! mismatch is a lens or spec bug, never a table edit. The registry and
//! replay harness are the orchestrator's; this file proves the module's
//! behaviour on its own, so a wiring mistake cannot hide a lens defect.

#[path = "pattern_expectations.rs"]
mod expectations;

// The production module is compiled by `src/pattern/mod.rs` once the
// orchestrator wires the registry. Until then this test compiles the exact
// same source under the test crate, with the library's `crate::{bounds,
// pattern, rule_lens}` roots re-exported so the module's imports resolve
// unchanged — the behaviour under test is the production file, not a copy.
mod bounds {
    pub use catherine_analysis::bounds::*;
}
mod pattern {
    pub use catherine_analysis::pattern::*;
}
mod rule_lens {
    pub use catherine_analysis::rule_lens::*;
}
#[path = "../src/pattern/interpreter.rs"]
mod interpreter;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LawKind, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::{EpistemicLabel, Finding, FindingKind};
use catherine_core::model::Model;
use expectations::{Expectation, EXPECTATIONS};

/// One check finding as the taxonomy's stable `(kind, label)` spelling.
fn check_key(finding: &Finding) -> (&'static str, &'static str) {
    let kind = match &finding.kind {
        FindingKind::Violation(_) => "violation",
        FindingKind::Dangling(_) => "dangling",
        FindingKind::Contradiction(_) => "contradiction",
        FindingKind::Underconstraint(_) => "underconstraint",
        FindingKind::Redundant(_) => "redundant",
        FindingKind::Granularity(_) => "granularity",
    };
    let label = match finding.label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    };
    (kind, label)
}

/// The `interpreter` rows, in table order.
fn interpreter_rows() -> Vec<&'static Expectation> {
    EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "interpreter")
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

/// Assert one recorded row: the lens completes, its checks match the
/// recorded multiset, and its migration channel is empty.
fn assert_row(row: &Expectation) {
    assert_eq!(row.class, "finding", "{} is a completed row", row.fault);
    let model = fixture(row.fixture);
    let bound = values(row);
    let input = LensInput {
        model: &model,
        values: &bound,
    };
    let outcome = interpreter::Lens.run(&input, &ScopeCaps::default());
    let findings = match outcome {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => panic!(
            "{} ({}) must evaluate, got UNKNOWN: {scope}",
            row.fixture, row.fault
        ),
    };
    let mut actual: Vec<(&'static str, &'static str)> =
        findings.checks.iter().map(check_key).collect();
    actual.sort_unstable();
    let mut expected = row.findings.to_vec();
    expected.sort_unstable();
    assert_eq!(
        actual, expected,
        "{} ({}, parity={}) recorded multiset",
        row.fixture, row.fault, row.parity
    );
    assert!(
        findings.migrations.is_empty(),
        "{} ({}) has no migration surface",
        row.fixture,
        row.fault
    );
}

#[test]
fn the_metadata_is_the_interpreter_disposition_row() {
    let meta = interpreter::Lens.meta();
    assert_eq!(meta.id, "interpreter");
    assert_eq!(meta.name, "Interpreter");
    assert_eq!(meta.abstraction, "sum type / recursive dispatch");
    assert_eq!(meta.law, LawKind::Coverage);
    assert_eq!(meta.law.as_str(), "coverage");
    assert_eq!(meta.scope, "variant-domain coverage only");
}

#[test]
fn the_table_has_the_recorded_interpreter_rows() {
    let rows = interpreter_rows();
    assert_eq!(rows.len(), 10);
    let faults: Vec<&str> = rows.iter().map(|row| row.fault).collect();
    assert_eq!(
        faults,
        ["clean", "NF1", "NF2", "NF3", "clean", "F1", "F2", "F3", "F4", "F5"]
    );
}

#[test]
fn every_recorded_row_matches_the_ledger_multiset() {
    let rows = interpreter_rows();
    assert!(
        !rows.is_empty(),
        "the ledger records at least one interpreter row"
    );
    for row in rows {
        assert_row(row);
    }
}

#[test]
fn the_killed_floor_and_parity_rows_stay_silent_or_match() {
    let parity: Vec<&'static Expectation> = interpreter_rows()
        .into_iter()
        .filter(|row| row.parity)
        .collect();
    assert_eq!(parity.len(), 7, "the floor and parity arms are recorded");
    for row in parity {
        assert_row(row);
    }
}

#[test]
fn a_model_without_a_rule_presentation_is_unknown_not_clean() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match interpreter::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("rule_presentation")),
        LensOutcome::Findings(findings) => {
            panic!("an absent surface must not be clean: {findings:?}")
        }
    }
}

#[test]
fn unbound_values_are_an_unknown_scope() {
    let model = fixture("fixtures/katas/visitor-sum-types/visitor.ct.json");
    let values = vec!["nope=1".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match interpreter::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("does not declare")),
        LensOutcome::Findings(_) => panic!("an unbound domain must not be evaluated"),
    }
}
