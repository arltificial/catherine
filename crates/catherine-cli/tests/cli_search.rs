//! Acceptance tests for the `catherine search` subcommand — the
//! search-outcome pins (exit 5 contradiction, UNKNOWN scope payloads,
//! witness payload, labeled `--rows` errors), split out of `cli.rs` for
//! the file-size gate; the spawn/parse plumbing is the shared
//! `tests/common` harness (`try_run`/`envelope`), spelled for `?`.

mod common;

use common::{code, envelope, findings, stderr, stdout, try_run};
use serde_json::Value;
use std::error::Error;

const ANALYSIS_CONTRADICTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_contradiction.ct.json"
);

/// The contradiction run's payload: the check envelope, the restated
/// bound, and the executed contradiction finding with its witness scope.
fn assert_contradiction_payload(v: &Value) {
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "check");
    assert_eq!(v["data"]["outcome"], "contradiction");
    assert_eq!(
        v["data"]["bound"]["max_rows"]["b"], 0,
        "the bound is restated"
    );
    let found = findings(v);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["kind"], "contradiction");
    assert_eq!(found[0]["label"], "executed");
    assert_eq!(
        found[0]["witness"]["scope"],
        serde_json::json!([0, 1]),
        "the accepted equations, canonical indices in order"
    );
}

/// The UNKNOWN payload: the named epistemic state, the exact explored
/// scope, the answerability hint, and no finding of any kind.
fn assert_unknown_payload_names_the_explored_scope(v: &Value) {
    assert_eq!(
        v["data"]["outcome"], "unknown",
        "the epistemic state is named"
    );
    assert_eq!(
        v["data"]["scope"]["candidates_explored"], 2,
        "the scope records exactly what was explored"
    );
    assert_eq!(
        v["data"]["scope"]["rows"]["a"], 1,
        "the scope names the row bounds"
    );
    assert_eq!(
        v["data"]["hint"],
        "bound exhausted; pass --rows OBJECT=N or --max-candidates=N to explore further",
        "the unknown payload names the flags that make it answerable"
    );
    assert!(
        findings(v).is_empty(),
        "an UNKNOWN carries no finding — never a naked or implied answer"
    );
}

/// The zero-size payload: UNKNOWN with the empty scope, and no findings.
fn assert_zero_size_unknown_is_the_empty_scope(v: &Value) {
    assert_eq!(v["data"]["outcome"], "unknown");
    let rows = v["data"]["scope"]["rows"]
        .as_object()
        .unwrap_or_else(|| panic!("scope rows object"));
    assert!(rows.is_empty(), "the empty scope is the empty row map");
    assert_eq!(v["data"]["scope"]["candidates_explored"], 0);
    assert_eq!(
        v["data"]["hint"], "no rows bound; pass --rows OBJECT=N",
        "the empty-scope unknown names the --rows flag"
    );
    assert!(
        findings(v).is_empty(),
        "zero-size bounds never report a contradiction"
    );
}

/// `search` over the over-constrained fixture with `b` empty: the space
/// exhausts within the budget, so the run is findings-bearing — exit 5
/// through the existing decision, the contradiction finding with its
/// witness payload.
#[test]
fn analysis_search_contradiction_exits_5_with_finding_payload() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "a=1",
        "--rows",
        "b=0",
        "--rows",
        "c=1",
        "--max-candidates",
        "64",
    ])?;
    assert_eq!(code(&out), 5, "contradiction exit code");
    assert_eq!(stderr(&out), "", "no errors");
    let v = envelope("stdout", &stdout(&out))?;
    assert_contradiction_payload(&v);
    Ok(())
}

/// The same fixture under a budget smaller than the space: exit 0 (a
/// completed bounded run is a successful tool execution) with an
/// explicit UNKNOWN payload — `outcome: "unknown"` + the scope naming
/// the explored bounds — and never a contradiction claim.
#[test]
fn analysis_search_unknown_payload_names_the_explored_scope() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "a=1",
        "--rows",
        "b=0",
        "--rows",
        "c=1",
        "--max-candidates",
        "2",
    ])?;
    assert_eq!(code(&out), 0, "the run completed within its bounds");
    let v = envelope("stdout", &stdout(&out))?;
    assert_unknown_payload_names_the_explored_scope(&v);
    Ok(())
}

/// Zero-size bounds: UNKNOWN with the empty scope — never a
/// contradiction claim (spec scenario, ticket anti_goal).
#[test]
fn analysis_search_zero_size_bound_yields_unknown_empty_scope() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "a=0",
        "--rows",
        "b=0",
        "--rows",
        "c=0",
    ])?;
    assert_eq!(code(&out), 0);
    let v = envelope("stdout", &stdout(&out))?;
    assert_zero_size_unknown_is_the_empty_scope(&v);
    Ok(())
}

/// Freeing `b`: the smallest total instance witnesses within bounds —
/// exit 0 with the witness payload.
#[test]
fn analysis_search_witness_exits_0_with_instance_payload() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "a=1",
        "--rows",
        "b=1",
        "--rows",
        "c=1",
    ])?;
    assert_eq!(code(&out), 0);
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["data"]["outcome"], "witness");
    assert_eq!(v["data"]["witness"]["rows"]["b"], serde_json::json!([0]));
    Ok(())
}

/// Malformed `--rows` values are CLI-level labeled errors, exit 1 on
/// stderr — not usage errors, never panics.
#[test]
fn analysis_search_bad_bound_is_a_labeled_error() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["search", ANALYSIS_CONTRADICTION, "--rows", "a1", "--json"])?;
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["data"]["code"], "bad_argument");
    assert!(!stderr(&out).contains("panicked"));
    Ok(())
}

/// catherine-0kw: an `--rows` spec naming an undeclared object is
/// ignored for the bound, but named on the warnings channel — never
/// dropped without a word.
#[test]
fn analysis_search_warns_on_undeclared_rows_object() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "Objet=4",
        "--rows",
        "a=1",
        "--rows",
        "b=0",
        "--rows",
        "c=1",
        "--max-candidates",
        "64",
    ])?;
    assert_eq!(code(&out), 5, "exit semantics unchanged by the warning");
    let v = envelope("stdout", &stdout(&out))?;
    let warnings = v["warnings"]
        .as_array()
        .unwrap_or_else(|| panic!("warnings array"));
    assert_eq!(warnings.len(), 1, "exactly one warning for the typo");
    assert!(
        warnings[0]["message"]
            .as_str()
            .map(|m| m.contains("Objet"))
            .unwrap_or(false),
        "the warning names the ignored object: {:?}",
        warnings[0]
    );
    Ok(())
}

/// catherine-uh0: the same `--rows` warning reaches stderr in human mode
/// (the default surface) — `warning: <message>`, naming the typo'd
/// object; the JSON envelope keeps its warnings untouched.
#[test]
fn analysis_search_human_mode_prints_rows_warning_to_stderr() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--rows",
        "Objet=2",
        "--rows",
        "a=1",
    ])?;
    assert_eq!(code(&out), 5, "exit semantics unchanged by the warning");
    let err = stderr(&out);
    assert!(
        err.contains("warning: --rows names undeclared object `Objet`"),
        "human-mode stderr carries the warning line: {err:?}"
    );
    Ok(())
}
