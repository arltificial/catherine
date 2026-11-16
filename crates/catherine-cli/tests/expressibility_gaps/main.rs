//! Purpose: the executable disposition harness for known expressibility
//! limitations (`openspec/changes/add-expressibility-gap-suite`, beads
//! `catherine-j6g`) — one test per spec scenario, each dispatching over the
//! table at `docs/ref/expressibility-dispositions.md` and the per-row cases in
//! [`cases`].
//!
//! Responsibilities: declare the shared modules, expose the gap-suite fixture
//! root, and call the disposition runner (GREEN: `runner.rs`) with the
//! registered cases, then assert on the returned report — the rows in scope,
//! the number actually visited, and an empty flipped set — so a runner or
//! table parser that checks nothing cannot pass. Every test function below is
//! bound by a scenario contract under `.espectacular/gap-suite/`.
//!
//! Rationale: the case observations are RED work and live in [`cases`]; the
//! table→verdict runner is the one production item GREEN adds, so this binary
//! fails to compile naming `runner` until then — the intended RED state, never
//! a stub that could pass vacuously. `common` is compiled per integration
//! binary (the `catherine-core/tests/common` precedent).

mod cases;
#[path = "../common/mod.rs"]
mod common;
mod evidence;
mod runner;

/// Absolute path to one gap-suite fixture document.
pub fn fixture(name: &str) -> String {
    format!(
        "{}/tests/fixtures/gap_suite/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Absolute path to the gap-suite fixture directory.
pub fn fixture_dir() -> String {
    format!("{}/tests/fixtures/gap_suite", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn fixed_limitation_must_flip_its_row() {
    let report = runner::gap_rows_still_show_the_limitation(&cases::all());
    assert!(
        report.rows > 0,
        "the disposition table parsed no rows for the gap direction"
    );
    assert_eq!(
        report.checked, report.rows,
        "the gap direction must visit every row in scope"
    );
    assert!(
        report.flipped.is_empty(),
        "rows contradicting their `gap` disposition: {:?}",
        report.flipped
    );
}

#[test]
fn regressed_limitation_must_flip_its_row_back() {
    let report = runner::closed_rows_still_show_the_fix(&cases::all());
    // Zero `closed` rows is honest at registration (every row starts `gap`);
    // the direction becomes non-vacuous the moment the first row flips, and
    // the gap direction still requires rows > 0.
    assert_eq!(
        report.checked, report.rows,
        "the closed direction must visit every row in scope"
    );
    assert!(
        report.flipped.is_empty(),
        "rows contradicting their `closed` disposition: {:?}",
        report.flipped
    );
}

#[test]
fn stale_claim_registers_closed_not_as_a_gap() {
    let report = runner::closed_rows_cite_fixing_evidence(&cases::all());
    assert!(
        report.rows > 0,
        "the evidence check parsed no disposition rows"
    );
    assert_eq!(
        report.checked, report.rows,
        "the evidence check must visit every row"
    );
    assert!(
        report.flipped.is_empty(),
        "rows whose evidence contradicts their disposition: {:?}",
        report.flipped
    );
}

#[test]
fn unregistered_case_fails_the_suite() {
    let report = runner::every_case_is_registered(&cases::all());
    assert!(report.rows > 0, "registration parsed no disposition rows");
    assert_eq!(
        report.checked, report.rows,
        "registration must visit every row"
    );
    assert!(
        report.flipped.is_empty(),
        "rows whose registration contradicts the table: {:?}",
        report.flipped
    );
}

#[test]
fn claim_without_a_row_fails_the_gate() {
    let report = runner::limitation_claims_name_a_disposition_row(&cases::all());
    assert!(report.rows > 0, "the claim gate parsed no disposition rows");
    assert_eq!(
        report.checked, report.rows,
        "the claim gate must consult every row"
    );
    assert!(
        report.flipped.is_empty(),
        "rows whose claim contradicts the table: {:?}",
        report.flipped
    );
}

#[test]
fn gate_runs_in_ci() {
    let report = runner::gate_is_wired_into_ci(&cases::all());
    assert!(
        report.rows > 0,
        "the CI wiring check parsed no disposition rows"
    );
    assert_eq!(
        report.checked, report.rows,
        "the CI wiring check must consult every row"
    );
    assert!(
        report.flipped.is_empty(),
        "rows the CI wiring fails to guard: {:?}",
        report.flipped
    );
}
