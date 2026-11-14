//! Purpose: the State lens's own Meter (beads `catherine-mx1.12`) — runs
//! [`Lens`] directly over every recorded `state` row in the checked-in
//! expectation table and asserts the ledger's exact `(kind, label)`
//! multiset, per fixture and fault.
//!
//! Responsibilities: filter the table by the `state` id, load each row's
//! fixture (a `construction` row asserts the recorded parse refusal
//! instead), execute the lens (never the registry), compare the checks
//! channel as a multiset, assert the migration channel stays empty, and
//! pin the control (`killed`) and null-kill rows: the clean and
//! unreachable fixtures emit no executed finding, only the recorded
//! hypothesized questions. Also pins the metadata row.
//!
//! Rationale: the ledger is sealed and the table is checked in, so a
//! mismatch is a lens or spec bug, never a table edit. The registry and
//! replay harness are the orchestrator's; this file proves the module's
//! behaviour on its own, so a wiring mistake cannot hide a lens defect.
//! The production module is compiled under the test crate with the
//! library's `crate::{bounds, parallel, pattern}` roots re-exported so
//! the module's imports resolve unchanged.

#[path = "pattern_expectations.rs"]
mod expectations;

mod bounds {
    pub use catherine_analysis::bounds::*;
}
mod parallel {
    pub use catherine_analysis::parallel::*;
}
mod pattern {
    pub use catherine_analysis::pattern::*;
}
#[path = "../src/pattern/state.rs"]
mod state;

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
    };
    let label = match finding.label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    };
    (kind, label)
}

/// The `state` rows, in table order.
fn state_rows() -> Vec<&'static Expectation> {
    EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "state")
        .collect()
}

/// A repository-relative fixture path.
fn fixture_path(relative: &str) -> String {
    format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"))
}

/// Load one fixture's source text.
fn fixture_text(relative: &str) -> String {
    std::fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"))
}

/// Load one fixture model.
fn fixture(relative: &str) -> Model {
    let text = fixture_text(relative);
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

/// Run the lens on a model and return its completed findings, or the
/// exact scope of a bounded non-answer.
fn run(model: &Model, values: &[String]) -> LensOutcome {
    let input = LensInput { model, values };
    state::Lens.run(&input, &ScopeCaps::default())
}

/// Assert one recorded row: a `construction` row is the recorded parse
/// refusal; every other row completes with the recorded multiset on the
/// checks channel and an empty migration channel.
fn assert_row(row: &Expectation) {
    if row.class == "construction" {
        assert!(
            catherine_core::parse::parse(&fixture_text(row.fixture)).is_err(),
            "{} is the recorded parse refusal",
            row.fixture
        );
        return;
    }
    let model = fixture(row.fixture);
    let bound = values(row);
    let findings = match run(&model, &bound) {
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
        "{} ({}, class={}, parity={}) recorded multiset",
        row.fixture, row.fault, row.class, row.parity
    );
    assert!(
        findings.migrations.is_empty(),
        "{} ({}) has no migration surface",
        row.fixture,
        row.fault
    );
}

#[test]
fn the_metadata_is_the_state_disposition_row() {
    let meta = state::Lens.meta();
    assert_eq!(meta.id, "state");
    assert_eq!(meta.name, "State");
    assert_eq!(meta.abstraction, "state shape");
    assert_eq!(meta.law, LawKind::Totality);
    assert_eq!(meta.law.as_str(), "totality");
    assert_eq!(
        meta.scope,
        "transition laws and cell totality; unreachable-state unexpressible; one null kill"
    );
}

#[test]
fn the_table_has_the_recorded_state_rows() {
    let rows = state_rows();
    assert_eq!(rows.len(), 10);
    let faults: Vec<&str> = rows.iter().map(|row| row.fault).collect();
    assert_eq!(
        faults,
        [
            "clean",
            "NF1",
            "NF2",
            "dangling-target",
            "missing-exit",
            "underconstraint-question",
            "drift-target",
            "undeclared-state",
            "unreachable-state",
            "underconstraint-question",
        ]
    );
    assert!(
        rows.iter()
            .all(|row| matches!(row.class, "finding" | "gap" | "construction")),
        "the ledger's three classes are the only recorded ones"
    );
}

#[test]
fn every_recorded_row_matches_the_ledger_multiset() {
    let rows = state_rows();
    assert!(
        !rows.is_empty(),
        "the ledger records at least one state row"
    );
    for row in rows {
        assert_row(row);
    }
}

#[test]
fn the_killed_floor_and_parity_rows_stay_silent_or_match() {
    let parity: Vec<&'static Expectation> =
        state_rows().into_iter().filter(|row| row.parity).collect();
    assert_eq!(parity.len(), 7, "the floor and parity arms are recorded");
    for row in parity {
        assert_row(row);
    }
}

#[test]
fn the_clean_control_and_null_kill_emit_no_executed_finding() {
    for fixture_name in [
        "fixtures/katas/state-discovery/near-floor.ct.json",
        "fixtures/katas/state-discovery/unreachable.ct.json",
    ] {
        let model = fixture(fixture_name);
        let findings = match run(&model, &[]) {
            LensOutcome::Findings(findings) => findings,
            LensOutcome::Unknown { scope } => {
                panic!("{fixture_name} is inside the lens's domain: {scope}")
            }
        };
        assert!(
            findings.checks.iter().all(|finding| {
                matches!(finding.label, EpistemicLabel::Hypothesized)
                    && matches!(finding.kind, FindingKind::Underconstraint(_))
            }),
            "{fixture_name} records no executed finding: {:?}",
            findings.checks
        );
        assert!(findings.migrations.is_empty());
    }
    // The killed control is silent outright; the null-kill fixture keeps
    // only the two recorded hypothesized schema questions.
    let control = match run(
        &fixture("fixtures/katas/state-discovery/near-floor.ct.json"),
        &[],
    ) {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => panic!("the control must evaluate: {scope}"),
    };
    assert!(control.checks.is_empty(), "the killed control is silent");
    let null_kill = match run(
        &fixture("fixtures/katas/state-discovery/unreachable.ct.json"),
        &[],
    ) {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => panic!("the null kill must evaluate: {scope}"),
    };
    assert_eq!(null_kill.checks.len(), 2, "the recorded question pair");
}

#[test]
fn the_recorded_faults_are_never_silent() {
    for fixture_name in [
        "fixtures/katas/state-discovery/near-floor-fault-retarget.ct.json",
        "fixtures/katas/state-discovery/near-floor-fault-missing-exit.ct.json",
        "fixtures/katas/state-discovery/dangling-target.ct.json",
        "fixtures/katas/state-discovery/missing-exit.ct.json",
        "fixtures/katas/state-discovery/drift-target.ct.json",
    ] {
        let model = fixture(fixture_name);
        let findings = match run(&model, &[]) {
            LensOutcome::Findings(findings) => findings,
            LensOutcome::Unknown { scope } => {
                panic!("{fixture_name} is inside the lens's domain: {scope}")
            }
        };
        assert!(
            findings
                .checks
                .iter()
                .any(|finding| matches!(finding.label, EpistemicLabel::Executed)),
            "{fixture_name} carries executed evidence"
        );
    }
}

#[test]
fn an_empty_model_completes_clean() {
    let model = Model::empty();
    match run(&model, &[]) {
        LensOutcome::Findings(findings) => {
            assert!(findings.checks.is_empty());
            assert!(findings.migrations.is_empty());
        }
        LensOutcome::Unknown { scope } => panic!("an empty model is total: {scope}"),
    }
}
