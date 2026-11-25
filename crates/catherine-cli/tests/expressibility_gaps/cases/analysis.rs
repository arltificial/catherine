//! Purpose: the analysis-surface rows — what `distinguish`, associativity,
//! naturality and coproduct probes can and cannot say.
//! Responsibilities: one observation per row. Each row returns `Limitation`
//! only for its exact claimed signature and `Cleared` only for its exact
//! fixed signature; every other outcome panics.
//! Rationale: grouped out of `cases.rs` at the file-line gate; the scope
//! probe keeps every comparison and panic, with the branch decision moved
//! into the verdict helpers so the observation stays under the cyclomatic
//! gate.

use super::probe::{self, assert_diagnostic_names, envelope, findings, warnings};
use super::Observation;
use crate::common;
use serde_json::Value;
use std::process::Output;

/// The three `distinguish` invocations behind the scope row: the bare form
/// (no observation set) and the `f`-only and `g`-only forms. Each run keeps
/// its exit code and reported outcome; the bare run's outcome stays empty
/// when the invocation fails for lack of `--observe`.
struct ScopeRuns {
    bare: Run,
    bare_missing_observe: bool,
    narrow: Run,
    wide: Run,
}

/// One `distinguish` invocation's exit code and reported outcome.
struct Run {
    exit: i32,
    outcome: String,
}

impl Run {
    /// Exit 0 reporting `outcome`.
    fn reports(&self, outcome: &str) -> bool {
        self.exit == 0 && self.outcome == outcome
    }

    fn of(out: &Output, value: Value) -> Self {
        Self {
            exit: common::code(out),
            outcome: outcome_of(&value),
        }
    }
}

/// The run envelope's `data.outcome`, empty when absent.
fn outcome_of(value: &Value) -> String {
    value["data"]["outcome"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

impl ScopeRuns {
    /// The claimed limitation: the bare form cannot pick the observation set,
    /// while `f` alone sees equivalence and `g` alone distinguishes.
    fn shows_limitation(&self) -> bool {
        self.bare.exit == 2
            && self.bare_missing_observe
            && self.narrow.reports("equivalent-in-scope")
            && self.wide.reports("distinguishable")
    }

    /// The fix: the bare form now enumerates the scope and sees the `g`
    /// difference, or the narrow run now sees past its hand-picked
    /// observation.
    fn shows_fix(&self) -> bool {
        self.bare.reports("distinguishable") || self.narrow.reports("distinguishable")
    }
}

fn scope_runs() -> ScopeRuns {
    let left = probe::fixture("distinguish_left.ct.json");
    let right = probe::fixture("distinguish_right.ct.json");

    let bare = common::run(&["distinguish", "--json", &left, &right]);
    let narrow = common::run(&["distinguish", "--observe", "f@a#0", "--json", &left, &right]);
    let wide = common::run(&["distinguish", "--observe", "g@a#0", "--json", &left, &right]);

    let bare_value = if common::code(&bare) == 0 {
        envelope(&bare, "bare observation")
    } else {
        Value::Null
    };
    ScopeRuns {
        bare: Run::of(&bare, bare_value),
        bare_missing_observe: common::stderr(&bare).contains("--observe"),
        narrow: Run::of(&narrow, envelope(&narrow, "narrow observation")),
        wide: Run::of(&wide, envelope(&wide, "wide observation")),
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
        runs.bare.exit, runs.bare_missing_observe, runs.narrow.outcome, runs.wide.outcome
    )
}

pub(super) fn associativity_scaffolding() -> Observation {
    let full = probe::fixture("associativity_scaffolded.ct.json");
    let full_out = common::run(&["check", &full, "--json"]);
    let full_value = envelope(&full_out, "associativity scaffold");

    let omitted = probe::fixture("associativity_omitted_triples.ct.json");
    let omitted_out = common::run(&["check", &omitted, "--json"]);
    let omitted_value = envelope(&omitted_out, "omitted triples");

    if scaffold_cost(&full_out, &full_value) && omission_is_clean(&omitted_out, &omitted_value) {
        return Observation::Limitation;
    }
    if declarative_surface(&full_out, &full_value)
        || carrier_completeness(&omitted_out, &omitted_value)
    {
        return Observation::Cleared;
    }
    panic!(
        "associativity-scaffolding: neither the scaffold cost nor a declarative fix — \
         scaffolded exit {}, omitted exit {}",
        common::code(&full_out),
        common::code(&omitted_out)
    )
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
    found
        .iter()
        .all(|finding| finding["kind"] == "violation" && finding["witness"]["equation"] == 7)
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
    let commuting = naturality_pair(&crm, "naturality_warehouse.ct.json");
    let broken = naturality_pair(&crm, "naturality_warehouse_break.ct.json");
    if absence_signature(&crm, &commuting) {
        return Observation::Limitation;
    }
    if fixed_signature(&commuting, &broken) {
        return Observation::Cleared;
    }
    panic!(
        "naturality-absence: unexpected behavior — commuting exit {}: {}; break exit {}: {}",
        common::code(&commuting),
        common::stderr(&commuting),
        common::code(&broken),
        common::stderr(&broken)
    )
}

fn naturality_pair(crm: &str, target: &str) -> Output {
    common::run(&["naturality", crm, &probe::fixture(target), "--json"])
}

/// The claimed limitation: `check` stays silent about the map *and* the
/// subcommand itself does not exist.
fn absence_signature(crm: &str, operation: &Output) -> bool {
    if common::code(operation) != 2 {
        return false;
    }
    if !common::stderr(operation).contains("unrecognized subcommand") {
        return false;
    }
    check_was_silent(crm)
}

/// The fixed signature: the commuting pair is quiet and the break pair is
/// the single `violation`/`executed` the row quotes.
fn fixed_signature(commuting: &Output, broken: &Output) -> bool {
    if common::code(commuting) != 0 {
        return false;
    }
    quiet(&envelope(commuting, "commuting naturality run")) && break_witness(broken)
}

fn check_was_silent(crm: &str) -> bool {
    let out = common::run(&["check", crm, "--json"]);
    if common::code(&out) != 0 {
        return false;
    }
    quiet(&envelope(&out, "claimed instance map"))
}

/// Exit 0 with no findings and no warnings: the signature is completely
/// quiet, not merely finding-free.
fn quiet(value: &Value) -> bool {
    findings(value).is_empty() && warnings(value).is_empty()
}

/// The break signature: exit 3 with exactly one finding, whose witness is
/// the `account_owner` cell at source row 2 with the two composed values
/// that differ — the row's quoted observation, never a bare exit code.
fn break_witness(broken: &Output) -> bool {
    if common::code(broken) != 3 {
        return false;
    }
    let value = envelope(broken, "break naturality run");
    let found = findings(&value);
    found.len() == 1 && quoted_break(&found[0])
}

fn quoted_break(finding: &Value) -> bool {
    if finding["kind"] != "violation" {
        return false;
    }
    finding["label"] == "executed" && quoted_witness(&finding["witness"])
}

fn quoted_witness(witness: &Value) -> bool {
    if witness["arrow"].as_str() != Some("account_owner") {
        return false;
    }
    witness["source_row"].as_i64() == Some(2) && composed_values(witness)
}

fn composed_values(witness: &Value) -> bool {
    witness["lhs"].as_i64() == Some(1) && witness["rhs"].as_i64() == Some(0)
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

/// The `selector-collision-claims` row: two `Capture` rows sharing one
/// `(partition, digest)` selector tuple — the bogart slice's real defect
/// class (kw9.5, kw9.11) — check silent, because no at-most-one-row-per-
/// selector-tuple claim is expressible over stored rows.
pub(super) fn selector_collision_claims() -> Observation {
    let run = probe::check_run("selector_collision_claims.ct.json");
    if run.is_silent() {
        return Observation::Limitation;
    }
    collision_fix(&run)
}

/// The fixed signature: a `violation`/`executed` (or diagnostic) outcome
/// whose text names the collision class — cardinality, uniqueness or a
/// duplicate selector tuple.
fn collision_fix(run: &probe::CheckRun) -> Observation {
    assert!(
        run.is_warning() || run.findings().is_some(),
        "selector_collision_claims: the fix names the collision, got {}",
        run.text()
    );
    assert_diagnostic_names!(
        "selector-collision-claims",
        &run.text(),
        &[
            "collision",
            "cardinal",
            "unique",
            "duplicate",
            "at most one",
        ],
    );
    Observation::Cleared
}
