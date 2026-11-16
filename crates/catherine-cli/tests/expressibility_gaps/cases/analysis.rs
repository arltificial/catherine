//! Purpose: the analysis-surface rows — what `distinguish`, associativity,
//! naturality and coproduct probes can and cannot say.
//! Responsibilities: one observation per row. Each row returns `Limitation`
//! only for its exact claimed signature and `Cleared` only for its exact
//! fixed signature; every other outcome panics.
//! Rationale: grouped out of `cases.rs` at the file-line gate; the scope
//! probe keeps every comparison and panic, with the branch decision moved
//! into the verdict helpers so the observation stays under the cyclomatic
//! gate.

use super::probe::{self, envelope, findings, warnings};
use super::Observation;
use crate::common;
use serde_json::Value;

/// The three `distinguish` invocations behind the scope row: the bare form
/// (no observation set) and the `f`-only and `g`-only forms.
struct ScopeRuns {
    bare_exit: i32,
    bare_missing_observe: bool,
    bare_outcome: String,
    narrow_exit: i32,
    narrow_outcome: String,
    wide_exit: i32,
    wide_outcome: String,
}

impl ScopeRuns {
    /// The claimed limitation: the bare form cannot pick the observation set,
    /// while `f` alone sees equivalence and `g` alone distinguishes.
    fn shows_limitation(&self) -> bool {
        self.bare_is_missing_observe() && self.narrow_matches() && self.wide_matches()
    }

    /// The fix: the bare form now enumerates the scope and sees the `g`
    /// difference, or the narrow run now sees past its hand-picked
    /// observation.
    fn shows_fix(&self) -> bool {
        self.bare_sees_difference() || self.narrow_sees_difference()
    }

    fn bare_is_missing_observe(&self) -> bool {
        self.bare_exit == 2 && self.bare_missing_observe
    }

    fn bare_sees_difference(&self) -> bool {
        self.bare_exit == 0 && self.bare_outcome == "distinguishable"
    }

    fn narrow_matches(&self) -> bool {
        self.narrow_exit == 0 && self.narrow_outcome == "equivalent-in-scope"
    }

    fn narrow_sees_difference(&self) -> bool {
        self.narrow_exit == 0 && self.narrow_outcome == "distinguishable"
    }

    fn wide_matches(&self) -> bool {
        self.wide_exit == 0 && self.wide_outcome == "distinguishable"
    }
}

fn scope_runs() -> ScopeRuns {
    let left = probe::fixture("distinguish_left.ct.json");
    let right = probe::fixture("distinguish_right.ct.json");

    let bare = common::run(&["distinguish", "--json", &left, &right]);
    let narrow = common::run(&["distinguish", "--observe", "f@a#0", "--json", &left, &right]);
    let wide = common::run(&["distinguish", "--observe", "g@a#0", "--json", &left, &right]);

    let narrow_value = envelope(&narrow, "narrow observation");
    let wide_value = envelope(&wide, "wide observation");
    let bare_value = if common::code(&bare) == 0 {
        envelope(&bare, "bare observation")
    } else {
        Value::Null
    };
    ScopeRuns {
        bare_exit: common::code(&bare),
        bare_missing_observe: common::stderr(&bare).contains("--observe"),
        bare_outcome: bare_value["data"]["outcome"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        narrow_exit: common::code(&narrow),
        narrow_outcome: narrow_value["data"]["outcome"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        wide_exit: common::code(&wide),
        wide_outcome: wide_value["data"]["outcome"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    }
}

pub(super) fn distinguish_observation_scope() -> Observation {
    let runs = scope_runs();
    if runs.shows_limitation() {
        return Observation::Limitation;
    }
    if runs.shows_fix() {
        return Observation::Cleared;
    }
    panic!(
        "distinguish: unexpected behavior — bare exit {} (missing --observe: {}), \
         f-only `{}`, g-only `{}`",
        runs.bare_exit, runs.bare_missing_observe, runs.narrow_outcome, runs.wide_outcome
    )
}

pub(super) fn associativity_scaffolding() -> Observation {
    let full = probe::fixture("associativity_scaffolded.ct.json");
    let full_out = common::run(&["check", &full, "--json"]);
    let full_value = envelope(&full_out, "associativity scaffold");

    let omitted = probe::fixture("associativity_omitted_triples.ct.json");
    let omitted_out = common::run(&["check", &omitted, "--json"]);
    let omitted_value = envelope(&omitted_out, "omitted triples");

    if associativity_limitation(&full_out, &full_value, &omitted_out, &omitted_value) {
        return Observation::Limitation;
    }
    if associativity_fix(&full_out, &full_value, &omitted_out, &omitted_value) {
        return Observation::Cleared;
    }
    panic!(
        "associativity-scaffolding: neither the scaffold cost nor a declarative fix — \
         scaffolded exit {}, omitted exit {}",
        common::code(&full_out),
        common::code(&omitted_out)
    )
}

fn associativity_limitation(
    full_out: &std::process::Output,
    full_value: &Value,
    omitted_out: &std::process::Output,
    omitted_value: &Value,
) -> bool {
    scaffold_cost(full_out, full_value) && omission_is_clean(omitted_out, omitted_value)
}

fn associativity_fix(
    full_out: &std::process::Output,
    full_value: &Value,
    omitted_out: &std::process::Output,
    omitted_value: &Value,
) -> bool {
    declarative_surface(full_out, full_value) || carrier_completeness(omitted_out, omitted_value)
}

/// The claimed signature: 18 violations at equation 7 over the hand-built
/// Triple rows.
fn scaffold_cost(out: &std::process::Output, value: &Value) -> bool {
    if common::code(out) != 3 {
        return false;
    }
    let found = findings(value);
    if found.len() != 18 {
        return false;
    }
    found.iter().all(|finding| {
        finding["kind"] == "violation" && finding["witness"]["equation"] == 7
    })
}

/// The claimed signature: omitting the failing triples makes the model clean.
fn omission_is_clean(out: &std::process::Output, value: &Value) -> bool {
    if common::code(out) != 0 {
        return false;
    }
    if !findings(value).is_empty() {
        return false;
    }
    warnings(value).is_empty()
}

/// The fix's clean shape: the scaffold cost is gone and the model checks
/// clean (the declarative operation surface replaced the hand-built rows).
fn declarative_surface(out: &std::process::Output, value: &Value) -> bool {
    if common::code(out) != 0 {
        return false;
    }
    if !findings(value).is_empty() {
        return false;
    }
    warnings(value).is_empty()
}

/// The fix's finding shape: the omitted triples are now caught as a
/// carrier-completeness violation.
fn carrier_completeness(out: &std::process::Output, value: &Value) -> bool {
    if common::code(out) != 3 {
        return false;
    }
    let found = findings(value);
    if found.is_empty() {
        return false;
    }
    found.iter().all(|finding| finding["kind"] == "violation")
}

pub(super) fn naturality_absence() -> Observation {
    let crm = probe::fixture("naturality_crm.ct.json");
    let warehouse = probe::fixture("naturality_warehouse.ct.json");

    let check = common::run(&["check", &crm, "--json"]);
    let check_value = envelope(&check, "claimed instance map");
    let map_silent = check_was_silent(&check, &check_value);

    let operation = common::run(&["naturality", &crm, &warehouse, "--json"]);
    let operation_exit = common::code(&operation);
    if operation_exit == 2 {
        assert!(
            common::stderr(&operation).contains("unrecognized subcommand"),
            "naturality: exit 2 for a reason other than the missing subcommand: {}",
            common::stderr(&operation)
        );
        assert!(
            map_silent,
            "naturality-absence: the subcommand is absent but the instance map is no longer \
             silently dropped"
        );
        return Observation::Limitation;
    }
    let value = envelope(&operation, "naturality");
    naturality_fix(operation_exit, &value).unwrap_or_else(|| {
        panic!(
            "naturality: unexpected exit {operation_exit}: {}",
            common::stderr(&operation)
        )
    })
}

fn check_was_silent(out: &std::process::Output, value: &Value) -> bool {
    if common::code(out) != 0 {
        return false;
    }
    if !findings(value).is_empty() {
        return false;
    }
    warnings(value).is_empty()
}

/// The fixed signature: a clean commuting run, or the violation findings
/// with the naturality witness `{arrow, source_row, lhs, rhs}`.
fn naturality_fix(exit: i32, value: &Value) -> Option<Observation> {
    match exit {
        0 => clean_fix(value),
        3 => violation_fix(value),
        _ => None,
    }
}

fn clean_fix(value: &Value) -> Option<Observation> {
    if findings(value).is_empty() {
        if warnings(value).is_empty() {
            return Some(Observation::Cleared);
        }
    }
    None
}

fn violation_fix(value: &Value) -> Option<Observation> {
    let found = findings(value);
    if found.is_empty() {
        return None;
    }
    for finding in found {
        assert_eq!(finding["kind"], "violation", "naturality finding kind");
        assert_eq!(finding["label"], "executed", "naturality finding label");
        let witness = &finding["witness"];
        assert!(witness["arrow"].is_string(), "naturality witness arrow");
        assert!(witness["source_row"].is_number(), "naturality witness source_row");
        assert!(
            witness["lhs"].is_number() || witness["lhs"].is_null(),
            "naturality witness lhs"
        );
        assert!(
            witness["rhs"].is_number() || witness["rhs"].is_null(),
            "naturality witness rhs"
        );
    }
    Some(Observation::Cleared)
}

pub(super) fn coproduct_absence() -> Observation {
    let path = probe::fixture("coproduct_payments.ct.json");
    let out = common::run(&["check", &path, "--json"]);
    match common::code(&out) {
        0 => {
            let value = envelope(&out, "coproduct declaration");
            assert!(
                findings(&value).is_empty(),
                "the claimed signature has no coproduct findings"
            );
            assert!(
                warnings(&value).is_empty(),
                "the claimed signature has no coproduct warnings"
            );
            Observation::Limitation
        }
        3 => coproduct_fix(&envelope(&out, "coproduct declaration")),
        other => panic!(
            "coproduct declaration: unexpected exit {other}: {}",
            common::stderr(&out)
        ),
    }
}

/// The fixed signature: the uncovered `voucher` row and the doubly-covered
/// `shared` row are `violation`/`executed` findings, and the disjointness
/// witness names both colliding injections.
fn coproduct_fix(value: &Value) -> Observation {
    let found = findings(value);
    assert!(!found.is_empty(), "coproduct coverage is a violation");
    assert!(
        found.len() >= 2,
        "coverage (voucher) and disjointness (shared) each produce a finding"
    );
    for finding in found {
        assert_eq!(finding["kind"], "violation", "coproduct finding kind");
        assert_eq!(finding["label"], "executed", "coproduct finding label");
    }
    let text = found
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("card_inj") && text.contains("transfer_inj"),
        "the disjointness witness names both colliding injections: {text}"
    );
    Observation::Cleared
}