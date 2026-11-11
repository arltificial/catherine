//! Acceptance tests for the `catherine search` subcommand — the
//! search-outcome pins (exit 5 contradiction, UNKNOWN scope payloads,
//! witness payload, labeled `--rows` errors), split out of `cli.rs` for
//! the file-size gate; the prelude (fixture consts, spawn/assert
//! helpers) is carried over verbatim.

use std::process::{Command, Output};

const ANALYSIS_CONTRADICTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_contradiction.ct.json"
);

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
        .expect("spawn catherine binary")
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn parse_json(text: &str) -> serde_json::Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("stdout is one JSON envelope: {e}\n{text}"))
}

/// `search` over the over-constrained fixture with `b` empty: the space
/// exhausts within the budget, so the run is findings-bearing — exit 5
/// through the existing decision, the contradiction finding with its
/// witness payload.
#[test]
fn analysis_search_contradiction_exits_5_with_finding_payload() {
    let out = run(&[
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
    ]);
    assert_eq!(code(&out), 5, "contradiction exit code");
    assert_eq!(stderr(&out), "", "no errors");
    let v = parse_json(&stdout(&out));
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "check");
    assert_eq!(v["data"]["outcome"], "contradiction");
    assert_eq!(
        v["data"]["bound"]["max_rows"]["b"], 0,
        "the bound is restated"
    );
    let findings = v["data"]["findings"].as_array().expect("findings array");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["kind"], "contradiction");
    assert_eq!(findings[0]["label"], "executed");
    assert_eq!(
        findings[0]["witness"]["scope"],
        serde_json::json!([0, 1]),
        "the accepted equations, canonical indices in order"
    );
}

/// The same fixture under a budget smaller than the space: exit 0 (a
/// completed bounded run is a successful tool execution) with an
/// explicit UNKNOWN payload — `outcome: "unknown"` + the scope naming
/// the explored bounds — and never a contradiction claim.
#[test]
fn analysis_search_unknown_payload_names_the_explored_scope() {
    let out = run(&[
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
    ]);
    assert_eq!(code(&out), 0, "the run completed within its bounds");
    let v = parse_json(&stdout(&out));
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
    let findings = v["data"]["findings"].as_array().expect("findings array");
    assert!(
        findings.is_empty(),
        "an UNKNOWN carries no finding — never a naked or implied answer"
    );
}

/// Zero-size bounds: UNKNOWN with the empty scope — never a
/// contradiction claim (spec scenario, ticket anti_goal).
#[test]
fn analysis_search_zero_size_bound_yields_unknown_empty_scope() {
    let out = run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "a=0",
        "--rows",
        "b=0",
        "--rows",
        "c=0",
    ]);
    assert_eq!(code(&out), 0);
    let v = parse_json(&stdout(&out));
    assert_eq!(v["data"]["outcome"], "unknown");
    let rows = v["data"]["scope"]["rows"]
        .as_object()
        .expect("scope rows object");
    assert!(rows.is_empty(), "the empty scope is the empty row map");
    assert_eq!(v["data"]["scope"]["candidates_explored"], 0);
    assert!(
        v["data"]["findings"]
            .as_array()
            .expect("findings")
            .is_empty(),
        "zero-size bounds never report a contradiction"
    );
}

/// Freeing `b`: the smallest total instance witnesses within bounds —
/// exit 0 with the witness payload.
#[test]
fn analysis_search_witness_exits_0_with_instance_payload() {
    let out = run(&[
        "search",
        ANALYSIS_CONTRADICTION,
        "--json",
        "--rows",
        "a=1",
        "--rows",
        "b=1",
        "--rows",
        "c=1",
    ]);
    assert_eq!(code(&out), 0);
    let v = parse_json(&stdout(&out));
    assert_eq!(v["data"]["outcome"], "witness");
    assert_eq!(v["data"]["witness"]["rows"]["b"], serde_json::json!([0]));
}

/// Malformed `--rows` values are CLI-level labeled errors, exit 1 on
/// stderr — not usage errors, never panics.
#[test]
fn analysis_search_bad_bound_is_a_labeled_error() {
    let out = run(&["search", ANALYSIS_CONTRADICTION, "--rows", "a1", "--json"]);
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = parse_json(&stderr(&out));
    assert_eq!(v["data"]["code"], "bad_argument");
    assert!(!stderr(&out).contains("panicked"));
}
