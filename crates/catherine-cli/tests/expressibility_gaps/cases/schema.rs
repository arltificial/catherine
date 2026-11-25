//! Purpose: the schema-surface rows — declarations the checker accepts
//! without enforcing anything.
//! Responsibilities: one observation per row about identity equations,
//! optional references, unknown fields, declared cones, and the now
//! declarable pullback squares. Each row returns `Limitation` only for its
//! exact claimed signature and `Cleared` only for its exact fixed
//! signature; every other outcome panics.
//! Rationale: grouped out of `cases.rs` at the file-line gate; the strict
//! signatures are what stop a broken fixture from laundering a limitation
//! into a `closed` row.

use super::probe::{self, assert_diagnostic_names, check_run, envelope, findings};
use super::Observation;
use crate::common;
use serde_json::Value;

pub(super) fn identity_empty_path_equations() -> Observation {
    let runs = [
        check_run("identity_involution.ct.json"),
        check_run("identity_memento_roundtrip.ct.json"),
    ];
    identity_disposition(&runs).unwrap_or_else(|| {
        panic!(
            "identity-empty-path-equations: neither the empty-path rejection nor the \
             executed identity equation: {}\n{}",
            runs[0].text(),
            runs[1].text()
        )
    })
}

fn identity_disposition(runs: &[probe::CheckRun]) -> Option<Observation> {
    if runs.iter().all(is_identity_rejection) {
        return Some(Observation::Limitation);
    }
    if runs.iter().all(is_identity_evaluation) {
        return Some(Observation::Cleared);
    }
    None
}

/// The claimed signature: `error[syntax]` for the empty equation side.
fn is_identity_rejection(run: &probe::CheckRun) -> bool {
    let text = run.text();
    text.starts_with("syntax:") && text.contains("equation path must be non-empty")
}

/// The fixed signature: the identity equation parsed and executed, so its
/// violation finds are present with the equation witness.
fn is_identity_evaluation(run: &probe::CheckRun) -> bool {
    let Some(found) = run.findings() else {
        return false;
    };
    !found.is_empty()
        && found.iter().all(|finding| {
            finding["kind"] == "violation"
                && finding["label"] == "executed"
                && finding["witness"]["equation"] == 0
        })
}

pub(super) fn optional_references() -> Observation {
    let path = probe::fixture("optional_reference.ct.json");
    let out = common::run(&["check", &path, "--json"]);
    match common::code(&out) {
        4 => {
            let value = envelope(&out, "optional reference");
            let found = findings(&value);
            assert_eq!(
                found.len(),
                1,
                "one dangling finding for the absent optional value"
            );
            assert_eq!(found[0]["kind"], "dangling");
            assert_eq!(found[0]["witness"]["arrow"], "address");
            assert_eq!(found[0]["witness"]["src"], 1);
            Observation::Limitation
        }
        0 => {
            let value = envelope(&out, "optional reference");
            assert!(
                findings(&value).is_empty(),
                "a cleared optional-reference run is clean of findings"
            );
            assert!(
                probe::warnings(&value).is_empty(),
                "a cleared optional-reference run is clean of warnings"
            );
            Observation::Cleared
        }
        other => panic!(
            "optional reference: unexpected exit {other}: {}",
            common::stderr(&out)
        ),
    }
}

pub(super) fn unknown_top_level_fields() -> Observation {
    let run = check_run("unknown_top_level_fields.ct.json");
    if run.is_silent() {
        return Observation::Limitation;
    }
    assert!(
        is_labeled_or_warning(&run),
        "unknown-top-level-fields: the fix is a labeled error or envelope warning, got {}",
        run.text()
    );
    let text = run.text();
    assert_diagnostic_names!("unknown-top-level-fields", &text, &["unknown"]);
    Observation::Cleared
}

/// The unknown-field fix's allowed channels: a labeled error or a warning.
fn is_labeled_or_warning(run: &probe::CheckRun) -> bool {
    run.is_labeled() || run.is_warning()
}

pub(super) fn declared_cones_ignored() -> Observation {
    let run = check_run("declared_cone.ct.json");
    if run.is_silent() {
        return Observation::Limitation;
    }
    let text = run.text();
    assert_diagnostic_names!(
        "declared-cones-ignored",
        &text,
        &["cones", "coproduct", "limit"],
    );
    Observation::Cleared
}

/// The claimed signature is a silent run: a pullback-shaped claim was
/// unstatable, so nothing could execute. The fixed signature is the
/// declared square's failing row as an executed violation naming
/// declaration `0` and both composed values.
pub(super) fn pullback_absence() -> Observation {
    let run = check_run("pullback_claims.ct.json");
    if run.is_silent() {
        return Observation::Limitation;
    }
    let text = run.text();
    let found = run
        .findings()
        .unwrap_or_else(|| panic!("pullback-absence: the claim must execute, got {text}"));
    assert_pullback_commutes_witness(found);
    assert_diagnostic_names!("pullback-absence", &text, &["pullback"]);
    Observation::Cleared
}

/// The commutes witness (not covers or unique): an executed violation whose
/// witness names pullback `0` and two unequal composed values, so the
/// square was actually evaluated rather than silently dropped.
fn assert_pullback_commutes_witness(found: &[Value]) {
    let witness = found
        .iter()
        .find(|finding| finding["kind"] == "violation" && finding["witness"]["pullback"] == 0)
        .unwrap_or_else(|| panic!("pullback-absence: no violation names pullback 0: {found:?}"));
    assert_eq!(
        witness["label"], "executed",
        "a failing square is an executed violation: {witness}"
    );
    assert_ne!(
        witness["witness"]["lhs"], witness["witness"]["rhs"],
        "the commutes witness names two unequal composed values: {witness}"
    );
}
