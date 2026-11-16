//! Purpose: the migration rows — what `migrate` loses or accepts without
//! checking.
//! Responsibilities: one observation per row about non-functorial schema maps,
//! dropped equations, and vacuous clean instances. Each row returns
//! `Limitation` only for its exact claimed signature and `Cleared` only for
//! its exact fixed signature; every other outcome panics.
//! Rationale: grouped out of `cases.rs` at the file-line gate; the narrow
//! exit mapping stops an unrelated exit 3/8 from reading as a fix.

use super::probe::{self, assert_diagnostic_names, check_run, envelope};
use super::Observation;
use crate::common;

fn migrate_without_loss(name: &str, expected_objects: &[&str]) -> Observation {
    let path = probe::fixture(name);
    let out = common::run(&["migrate", &path, "--json"]);
    let value = envelope(&out, name);
    match common::code(&out) {
        0 => {
            assert_lossless(name, &value, expected_objects);
            Observation::Limitation
        }
        1 => {
            assert_eq!(value["ok"], false, "{name}: exit 1 is the error envelope");
            let code = value["data"]["code"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: a rejected map is a labeled error"));
            assert!(
                code != "syntax" && code != "no_schema_map",
                "{name}: a broken document is not the map-checking fix (code `{code}`)"
            );
            let text = format!(
                "{code}: {}",
                value["data"]["message"].as_str().unwrap_or_default()
            );
            assert_diagnostic_names(name, &text, &["map", "functor", "equation", "endpoint"]);
            Observation::Cleared
        }
        _ => migrated_loss(name, &out, &value),
    }
}

/// The fixed signature on the finding-bearing exit: the map now reports the
/// lost data it previously swallowed. Any other exit panics.
fn migrated_loss(name: &str, out: &std::process::Output, value: &serde_json::Value) -> Observation {
    if common::code(out) == 8 {
        let lost = value["data"]["lost"].as_array().expect("lost array");
        assert!(
            !lost.is_empty(),
            "{name}: the fixed migration reports lost data"
        );
        return Observation::Cleared;
    }
    panic!(
        "{name}: unexpected migrate exit {}: {}",
        common::code(out),
        common::stderr(out)
    )
}

/// The claimed signature: lossless reindexing covers every expected object.
fn assert_lossless(name: &str, value: &serde_json::Value, expected_objects: &[&str]) {
    let rows = value["data"]["rows"]
        .as_object()
        .unwrap_or_else(|| panic!("{name}: migration ran"));
    for object in expected_objects {
        assert!(
            rows.contains_key(*object),
            "{name}: migration covered object {object}"
        );
    }
    let lost = value["data"]["lost"].as_array().expect("lost array");
    assert!(
        lost.is_empty(),
        "{name}: the claimed behavior is zero lost data"
    );
}

pub(super) fn non_functorial_schema_map() -> Observation {
    migrate_without_loss("nonfunctorial_schema_map.ct.json", &["X", "Y"])
}

pub(super) fn dropped_equations_on_migrate() -> Observation {
    migrate_without_loss("dropped_equation_migration.ct.json", &["A2", "B2", "C2"])
}

pub(super) fn vacuous_clean_instances() -> Observation {
    let zero = check_run("zero_rows_instance.ct.json");
    if zero.is_silent() {
        return Observation::Limitation;
    }
    vacuity_fix(&zero)
}

/// The fixed signature: a vacuity diagnostic on the zero-row instance. The
/// legacy wrong-shaped instance this row used to share is no longer part of
/// it — that shape is refused as `syntax` (catherine-3en, pinned by
/// `negative_matrix.rs`), while the zero-row half still checks clean.
fn vacuity_fix(zero: &probe::CheckRun) -> Observation {
    assert!(
        zero.is_warning() || zero.findings().is_some(),
        "zero_rows_instance: the fix is a vacuity diagnostic, got {}",
        zero.text()
    );
    assert_diagnostic_names(
        "vacuous-clean-instances",
        &zero.text(),
        &["vacuous", "zero", "empty", "no row"],
    );
    Observation::Cleared
}
