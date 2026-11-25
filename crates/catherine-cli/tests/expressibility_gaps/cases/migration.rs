//! Purpose: the migration rows — what `migrate` loses or accepts without
//! checking.
//! Responsibilities: one observation per row about non-functorial schema maps,
//! dropped equations, dropped pullback support arrows, and vacuous clean
//! instances. Each row returns `Limitation` only for its exact claimed
//! signature and `Cleared` only for its exact fixed signature; every other
//! outcome panics.
//! Rationale: grouped out of `cases.rs` at the file-line gate; the narrow
//! exit mapping stops an unrelated exit 3/8 from reading as a fix.

use super::probe::{self, assert_diagnostic_names, check_run, envelope};
use super::Observation;
use crate::common;

/// One `migrate --json` run over a named fixture: the fixture name, the
/// raw process output and its parsed JSON envelope. The shared context the
/// row's three outcomes (lossless, refused, loss-reporting) read.
struct MigrateRun<'a> {
    name: &'a str,
    out: std::process::Output,
    value: serde_json::Value,
}

impl<'a> MigrateRun<'a> {
    fn of(name: &'a str) -> Self {
        let path = probe::fixture(name);
        let out = common::run(&["migrate", &path, "--json"]);
        let value = envelope(&out, name);
        Self { name, out, value }
    }
}

fn migrate_without_loss(name: &str, expected_objects: &[&str]) -> Observation {
    let run = MigrateRun::of(name);
    match common::code(&run.out) {
        0 => {
            assert_lossless(&run, expected_objects);
            Observation::Limitation
        }
        1 => rejected_map(&run),
        _ => migrated_loss(&run),
    }
}

/// The exit-1 shape: the map was checked and refused as a labeled error
/// naming the map-checking fix — never a merely broken document.
fn rejected_map(run: &MigrateRun<'_>) -> Observation {
    let name = run.name;
    assert_eq!(
        run.value["ok"], false,
        "{name}: exit 1 is the error envelope"
    );
    let code = run.value["data"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: a rejected map is a labeled error"));
    assert!(
        code != "syntax" && code != "no_schema_map",
        "{name}: a broken document is not the map-checking fix (code `{code}`)"
    );
    let text = format!(
        "{code}: {}",
        run.value["data"]["message"].as_str().unwrap_or_default()
    );
    assert_diagnostic_names!(
        name,
        &text,
        &["map", "functor", "equation", "pullback", "endpoint"],
    );
    Observation::Cleared
}

/// The fixed signature on the finding-bearing exit: the map now reports the
/// lost data it previously swallowed. Any other exit panics.
fn migrated_loss(run: &MigrateRun<'_>) -> Observation {
    let name = run.name;
    if common::code(&run.out) == 8 {
        let lost = run.value["data"]["lost"].as_array().expect("lost array");
        assert!(
            !lost.is_empty(),
            "{name}: the fixed migration reports lost data"
        );
        return Observation::Cleared;
    }
    panic!(
        "{name}: unexpected migrate exit {}: {}",
        common::code(&run.out),
        common::stderr(&run.out)
    )
}

/// The claimed signature: lossless reindexing covers every expected object.
fn assert_lossless(run: &MigrateRun<'_>, expected_objects: &[&str]) {
    let name = run.name;
    let rows = run.value["data"]["rows"]
        .as_object()
        .unwrap_or_else(|| panic!("{name}: migration ran"));
    for object in expected_objects {
        assert!(
            rows.contains_key(*object),
            "{name}: migration covered object {object}"
        );
    }
    let lost = run.value["data"]["lost"].as_array().expect("lost array");
    assert!(
        lost.is_empty(),
        "{name}: the claimed behavior is zero lost data"
    );
}

pub(super) fn non_functorial_schema_map() -> Observation {
    nonfunctorial_refusal("nonfunctorial_schema_map.ct.json")
}

/// The fixed signature for the non-functorial map (catherine-kpn.7):
/// `migrate` refuses to run the ill-typed map — exit 1, the labeled
/// `partial_schema_map` error naming the offending arrow and objects —
/// instead of silently migrating through the wrong-endpoint path. Any
/// other exit panics.
fn nonfunctorial_refusal(name: &str) -> Observation {
    let path = probe::fixture(name);
    let out = common::run(&["migrate", &path, "--json"]);
    assert_eq!(common::code(&out), 1, "{name}: the refusal is exit 1");
    let value = envelope(&out, name);
    assert_eq!(value["ok"], false, "{name}: exit 1 is the error envelope");
    let code = value["data"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: the refusal carries its error code"));
    assert_eq!(
        code, "partial_schema_map",
        "{name}: the parser's schema-map typing check is the fix"
    );
    let text = format!(
        "{code}: {}",
        value["data"]["message"].as_str().unwrap_or_default()
    );
    assert_diagnostic_names!(name, &text, &["f", "A", "B", "X", "Y", "endpoint"]);
    Observation::Cleared
}

pub(super) fn dropped_equations_on_migrate() -> Observation {
    migrate_without_loss("dropped_equation_migration.ct.json", &["A2", "B2", "C2"])
}

/// The same lossless signature over a dropped pullback support arrow: the
/// map assigns the accepted square's `p1` leg to `null` while every `p1`
/// cell is already null, so no lost cell exists to report and the
/// declaration vanishes silently (`Limitation`). A migration finding for
/// the lost declaration is the fix (`Cleared`).
pub(super) fn dropped_pullbacks_on_migrate() -> Observation {
    migrate_without_loss(
        "dropped_pullback_migration.ct.json",
        &["P2", "A2", "B2", "C2"],
    )
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
    assert_diagnostic_names!(
        "vacuous-clean-instances",
        &zero.text(),
        &["vacuous", "zero", "empty", "no row"],
    );
    Observation::Cleared
}
