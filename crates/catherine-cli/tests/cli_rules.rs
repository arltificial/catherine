//! Purpose: acceptance tests for the `catherine rules` subcommand
//! (openspec change `add-rule-lens` phase C, task 8) — the rule-lens
//! surface's envelope shape, exit table, human rendering, and labeled
//! errors end to end.
//!
//! Responsibilities: spawn the real binary (`CARGO_BIN_EXE_catherine`)
//! against the committed rule-lens fixtures (`fixtures/rule_lens/`,
//! shared with the kernel suites) and the rules-only fixtures under
//! `tests/fixtures/`; assert the Decision-1 findings verbatim under
//! `data.findings`, the class exits (clean 0, gap 6, redundant 7), the
//! bounded `outcome: "unknown"` payload with its exact scope, the
//! one-line-per-finding human rendering, and the labeled errors
//! (`no_rule_presentation`, `bad_argument`, the kernel's parse codes).
//!
//! Rationale: the domain X is supplied through `--values` — declared
//! data the documents do not carry — so the tests exercise the same
//! bound-in path a user takes. Assertions are structural
//! (`serde_json::Value` fields/values) and numeric (exit codes), never
//! byte-compared snapshots.

use std::process::{Command, Output};

const GAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/rule_lens/gap.ct.json"
);
const SHADOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/rule_lens/shadow.ct.json"
);
const CLEAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rules_clean.ct.json"
);
/// A pre-1.2 document: parses, carries no `rule_presentation`.
const NO_SECTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bowling.ct.json"
);

/// Spawn the `catherine` binary with the given arguments.
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
        .unwrap_or_else(|e| panic!("stdout/stderr is one JSON envelope: {e}\n{text}"))
}

/// A clean partition: no gap, no overlap, no shadow, no derivation.
#[test]
fn a_clean_rule_table_is_exit_zero() {
    let out = run(&["rules", CLEAN, "--values", "n=1,10", "--json"]);
    assert_eq!(code(&out), 0, "no findings is clean");
    assert_eq!(stderr(&out), "", "the error stream stays empty");

    let v = parse_json(&stdout(&out));
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "check");
    assert_eq!(v["data"]["outcome"], "analyzed");
    assert_eq!(v["data"]["findings"].as_array().expect("findings").len(), 0);
    assert!(
        v["data"].get("scope").is_none(),
        "no scope on a completed run"
    );
}

/// The gap fixture maps to `underconstraint`/`hypothesized` with the
/// uncovered class and its dense example index, and exits 6.
#[test]
fn a_gap_is_an_underconstraint_exit_six() {
    let out = run(&["rules", GAP, "--values", "age=10,20,30,70", "--json"]);
    assert_eq!(code(&out), 6, "gap → underconstraint class exit");
    assert_eq!(stderr(&out), "");

    let v = parse_json(&stdout(&out));
    let findings = v["data"]["findings"].as_array().expect("findings");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["label"], "hypothesized");
    assert_eq!(findings[0]["kind"], "underconstraint");
    assert_eq!(findings[0]["witness"]["case"], "uncovered_inputs");
    assert_eq!(findings[0]["witness"]["detail"]["class"], "age = 70");
    assert_eq!(
        findings[0]["witness"]["detail"]["examples"],
        serde_json::json!([3]),
        "the example index is the dense enumeration index"
    );
}

/// The shadow fixture maps to `redundant`/`executed` with the dead rule
/// and its covering rule, and exits 7.
#[test]
fn a_shadowed_rule_is_a_redundancy_exit_seven() {
    let out = run(&["rules", SHADOW, "--values", "n=1,2,3,4,5,6", "--json"]);
    assert_eq!(code(&out), 7, "redundant class exit");
    assert_eq!(stderr(&out), "");

    let v = parse_json(&stdout(&out));
    let findings = v["data"]["findings"].as_array().expect("findings");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["label"], "executed");
    assert_eq!(findings[0]["kind"], "redundant");
    assert_eq!(
        findings[0]["witness"]["derivable"],
        serde_json::json!(["even"])
    );
    assert_eq!(
        findings[0]["witness"]["reducing"],
        serde_json::json!(["all"])
    );
}

/// A budget smaller than the domain is `outcome: "unknown"` with the
/// exact scope — a completed bounded run (exit 0), never collapsed into
/// clean and never a partial finding set.
#[test]
fn a_short_budget_names_the_unknown_scope_and_exits_zero() {
    let out = run(&[
        "rules",
        GAP,
        "--values",
        "age=10,20,30,70",
        "--max-candidates",
        "3",
        "--json",
    ]);
    assert_eq!(code(&out), 0, "a completed bounded run is a tool success");

    let v = parse_json(&stdout(&out));
    assert_eq!(v["data"]["outcome"], "unknown");
    assert_eq!(v["data"]["findings"], serde_json::json!([]));
    assert_eq!(
        v["data"]["scope"]["columns"],
        serde_json::json!({"age": 4}),
        "the scope restates the declared column sizes"
    );
    assert_eq!(v["data"]["scope"]["candidates_explored"], 3);
}

/// Human mode prints exactly one line per finding (kind + witness
/// summary); `--quiet` suppresses it without changing the exit.
#[test]
fn the_human_render_is_one_line_per_finding() {
    let out = run(&["rules", GAP, "--values", "age=10,20,30,70"]);
    assert_eq!(code(&out), 6, "the exit decision is mode-independent");
    assert_eq!(stderr(&out), "");
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    assert_eq!(lines.len(), 1, "one finding → one line");
    assert_eq!(
        lines[0],
        "underconstraint: uncovered inputs age = 70 (examples 3) (hypothesized)"
    );

    let quiet = run(&["rules", GAP, "--values", "age=10,20,30,70", "--quiet"]);
    assert_eq!(code(&quiet), 6, "quiet never changes the exit decision");
    assert_eq!(stdout(&quiet), "", "quiet suppresses the human lines");
}

/// A document without the additive section is the labeled
/// `no_rule_presentation` error, exit 1 — never an empty success.
#[test]
fn a_document_without_the_section_is_a_labeled_error() {
    let out = run(&["rules", NO_SECTION, "--json"]);
    assert_eq!(code(&out), 1, "no rule presentation is an input error");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = parse_json(&stderr(&out));
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "no_rule_presentation");
    assert!(!v["data"]["remediation"]
        .as_array()
        .expect("remediation")
        .is_empty());
}

/// A declared column with no supplied values is a labeled
/// `bad_argument`, never a vacuous clean analysis.
#[test]
fn a_missing_domain_column_is_a_bad_argument() {
    let out = run(&["rules", GAP, "--values", "other=1", "--json"]);
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out), "");
    let v = parse_json(&stderr(&out));
    assert_eq!(v["data"]["code"], "bad_argument");
    let message = v["data"]["message"].as_str().expect("message");
    assert!(
        message.contains("other") || message.contains("age"),
        "the rejected column is named: {message}"
    );
}
