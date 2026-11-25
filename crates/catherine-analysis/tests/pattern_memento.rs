//! Purpose: the Memento lens's own Meter (beads `catherine-mx1.11`) — runs
//! [`Lens`] directly over every recorded `memento` row in the checked-in
//! expectation table and asserts the ledger's exact `(kind, label)`
//! multiset, per fixture and fault.
//!
//! Responsibilities: filter the table by the `memento` id, load each row's
//! fixture, execute the lens (never the registry), compare the migrations
//! channel as a multiset, assert the checks channel stays empty, pin the
//! identity control's silence, and pin the exact lost-cell witnesses for
//! the three faults and the non-invertibility row.
//!
//! Rationale: the ledger is sealed and the table is checked in, so a
//! mismatch is a lens or spec bug, never a table edit. The registry and
//! replay harness belong to the orchestrator; this file proves the module's
//! behaviour on its own, so a wiring mistake cannot hide a lens defect.

#[path = "pattern_expectations.rs"]
mod expectations;

// The production module is compiled by `src/pattern/mod.rs` once the
// orchestrator wires the registry. Until then this test compiles the exact
// same source under the test crate, with the library's `crate::{bounds,
// pattern}` roots re-exported so the module's imports resolve unchanged —
// the behaviour under test is the production file, not a copy.
mod bounds {}
mod pattern {
    pub use catherine_analysis::pattern::*;
}
#[path = "../src/pattern/memento.rs"]
mod memento;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LawKind, LensCall, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::EpistemicLabel;
use catherine_core::migrate::MigrationFindingKind;
use catherine_core::model::Model;
use expectations::{Expectation, EXPECTATIONS};

/// The `memento` rows, in table order.
fn memento_rows() -> Vec<&'static Expectation> {
    EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "memento")
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

/// One migration finding as the migrate surface's stable `(kind, label)`.
fn migration_key(
    finding: &catherine_core::migrate::MigrationFinding,
) -> (&'static str, &'static str) {
    let kind = match &finding.kind {
        MigrationFindingKind::LostRow { .. } => "lost_row",
        MigrationFindingKind::LostCell { .. } => "lost_cell",
        MigrationFindingKind::Weakened { .. } => "weakened",
    };
    let label = match finding.label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    };
    (kind, label)
}

/// The lost-cell witnesses as `(arrow, row)`, in engine order.
fn lost_cells(outcome: &LensOutcome) -> Vec<(String, usize)> {
    let LensOutcome::Findings(findings) = outcome else {
        panic!("the recorded row must complete: {outcome:?}");
    };
    assert!(
        findings.checks.is_empty(),
        "the Δ lens fills only the migrations channel"
    );
    findings
        .migrations
        .iter()
        .map(|finding| match &finding.kind {
            MigrationFindingKind::LostCell { arrow, row } => (arrow.0.clone(), row.0),
            MigrationFindingKind::LostRow { object, row } => {
                panic!("unexpected lost_row: {}#{}", object.0, row.0)
            }
            MigrationFindingKind::Weakened { constraint, .. } => {
                panic!("unexpected weakened: {constraint}")
            }
        })
        .collect()
}

/// Run the lens directly over one row's fixture.
fn run_row(row: &Expectation) -> LensOutcome {
    let model = fixture(row.fixture);
    let values: Vec<String> = row
        .values
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    let input = LensInput {
        model: &model,
        values: &values,
    };
    memento::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    })
}

/// Assert one recorded row: the lens completes, its migrations match the
/// recorded multiset, and its checks channel is empty.
fn assert_row(row: &Expectation) {
    assert_eq!(row.class, "finding", "{} is a completed row", row.fault);
    let outcome = run_row(row);
    let LensOutcome::Findings(findings) = &outcome else {
        panic!("{} ({}) must evaluate, got UNKNOWN", row.fixture, row.fault);
    };
    assert!(
        findings.checks.is_empty(),
        "{} ({}) has no checks surface",
        row.fixture,
        row.fault
    );
    let mut actual: Vec<(&'static str, &'static str)> =
        findings.migrations.iter().map(migration_key).collect();
    actual.sort_unstable();
    let mut expected = row.findings.to_vec();
    expected.sort_unstable();
    assert_eq!(
        actual, expected,
        "{} ({}, parity={}) recorded multiset",
        row.fixture, row.fault, row.parity
    );
}

#[test]
fn the_metadata_is_the_memento_disposition_row() {
    let meta = memento::Lens.meta();
    assert_eq!(meta.id, "memento");
    assert_eq!(meta.name, "Memento");
    assert_eq!(meta.abstraction, "delta save/restore");
    assert_eq!(meta.law, LawKind::Delta);
    assert_eq!(meta.law.as_str(), "delta");
    assert_eq!(
        meta.scope,
        "lost-cell enumeration and non-invertibility witnesses; control self-negates"
    );
}

#[test]
fn the_table_has_the_recorded_memento_rows() {
    let rows = memento_rows();
    assert_eq!(rows.len(), 5);
    let faults: Vec<&str> = rows.iter().map(|row| row.fault).collect();
    assert_eq!(
        faults,
        [
            "memento-identity",
            "dropped-field",
            "lossy-optional",
            "version-skew",
            "non-invertibility"
        ]
    );
}

#[test]
fn every_recorded_row_matches_the_ledger_multiset() {
    let rows = memento_rows();
    assert!(
        !rows.is_empty(),
        "the ledger records at least one memento row"
    );
    for row in rows {
        assert_row(row);
    }
}

#[test]
fn the_identity_control_self_negates() {
    let row = memento_rows()
        .into_iter()
        .find(|row| row.fault == "memento-identity")
        .expect("the control row is recorded");
    let outcome = run_row(row);
    let LensOutcome::Findings(findings) = outcome else {
        panic!("the identity control must complete");
    };
    assert!(
        findings.checks.is_empty() && findings.migrations.is_empty(),
        "a faithful restore reports no false finding: {findings:?}"
    );
}

#[test]
fn the_dropped_field_is_named_at_every_valued_cell() {
    let row = memento_rows()
        .into_iter()
        .find(|row| row.fault == "dropped-field")
        .expect("the dropped-field row is recorded");
    let cells = lost_cells(&run_row(row));
    assert_eq!(
        cells,
        vec![
            ("snap_word_count".to_string(), 0),
            ("snap_word_count".to_string(), 1)
        ]
    );
}

#[test]
fn the_lossy_optional_field_is_named_only_where_valued() {
    let row = memento_rows()
        .into_iter()
        .find(|row| row.fault == "lossy-optional")
        .expect("the lossy-optional row is recorded");
    let cells = lost_cells(&run_row(row));
    assert_eq!(
        cells,
        vec![("page_note".to_string(), 1), ("page_note".to_string(), 3)]
    );
}

#[test]
fn the_version_skew_collapse_names_every_page_level_cell() {
    let row = memento_rows()
        .into_iter()
        .find(|row| row.fault == "version-skew")
        .expect("the version-skew row is recorded");
    let cells = lost_cells(&run_row(row));
    assert_eq!(
        cells,
        vec![
            ("page_next".to_string(), 0),
            ("page_note".to_string(), 1),
            ("page_text".to_string(), 0),
            ("page_text".to_string(), 1),
            ("page_text".to_string(), 2),
        ]
    );
}

#[test]
fn the_non_invertibility_row_reports_the_same_lost_cells() {
    let row = memento_rows()
        .into_iter()
        .find(|row| row.fault == "non-invertibility")
        .expect("the non-invertibility row is recorded");
    assert_row(row);
    assert_eq!(lost_cells(&run_row(row)).len(), 5);
}

#[test]
fn a_model_without_a_schema_map_is_unknown_not_clean() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match memento::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("schema_map")),
        LensOutcome::Findings(findings) => {
            panic!("an absent save/restore surface must not be clean: {findings:?}")
        }
    }
}
