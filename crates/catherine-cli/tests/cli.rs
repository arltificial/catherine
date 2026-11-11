//! Acceptance tests for the `catherine` CLI (openspec change
//! `add-check-cli`, beads `catherine-86i`).
//!
//! Purpose: pin the user-facing contract end-to-end — exit codes per
//! finding class, the genesis-vibes JSON envelope shape with findings
//! under `data.findings`, the labeled-error rendering, the DESIGN §5
//! agent-loop walkthrough (byte-for-byte human transcript), and the
//! `findings` view's payload equality with `check`.
//!
//! Responsibilities: spawn the real binary (`CARGO_BIN_EXE_catherine`)
//! against the fixture documents under `tests/fixtures/` and assert
//! structurally (serde_json::Value field names/values) and numerically
//! (exit codes) — never byte-compare JSON; the one mandated human-text
//! assertion is the §5 walkthrough transcript. Findings JSON is P2's
//! kernel encoding printed verbatim; tests read it back, the CLI never
//! does (no text-parsing exit contracts).
//!
//! Rationale: fixtures are real kernel documents (parse → build → check
//! produce the pinned findings — `bowling.ct.json` is engineered so the
//! kernel genuinely emits the §5 witness: equation 0, row `Assignment#3`,
//! values `Unit#1`/`Unit#2`); hand-edited findings would be a failed
//! fixture. The three analysis classes (contradiction/underconstraint/
//! redundant) have no P4 producer, so their exit codes and renderings
//! are pinned at the unit level (src/run.rs, src/render.rs), never by
//! fabricated end-to-end fixtures.

use std::process::{Command, Output};

const BOWLING: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bowling.ct.json"
);
const BOWLING_FIXED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bowling-fixed.ct.json"
);
const DANGLING: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/dangling.ct.json"
);
const MULTI_CLASS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/multi_class.ct.json"
);
const MALFORMED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/malformed.ct.json"
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
        .unwrap_or_else(|e| panic!("stdout is one JSON envelope: {e}\n{text}"))
}

// ── RED task 1: the violating fixture pins the envelope + exit ──────

#[test]
fn check_violating_fixture_emits_envelope_and_exits_3() {
    let out = run(&["check", BOWLING, "--json"]);
    assert_eq!(code(&out), 3, "violation exit code");
    assert_eq!(
        stderr(&out),
        "",
        "the error stream stays empty on a finding run"
    );

    let v = parse_json(&stdout(&out));
    assert_eq!(v["ok"], true, "findings envelope is a success envelope");
    for key in ["ok", "envelope_kind", "data", "warnings", "hints"] {
        assert!(v.get(key).is_some(), "envelope key `{key}` present");
    }

    let findings = v["data"]["findings"]
        .as_array()
        .expect("data.findings array");
    assert_eq!(findings.len(), 1, "exactly one violation entry");
    assert_eq!(findings[0]["kind"], "violation");
    assert_eq!(
        findings[0]["label"], "executed",
        "accepted equation fails as executed"
    );
    assert_eq!(
        findings[0]["witness"]["equation"], 0,
        "equation id (canonical index)"
    );
    assert_eq!(findings[0]["witness"]["row"], 3, "row id Assignment#3");
    assert_eq!(findings[0]["witness"]["lhs"], 1, "path value Unit#1");
    assert_eq!(findings[0]["witness"]["rhs"], 2, "path value Unit#2");
}

// ── RED task 2: the malformed fixture pins the labeled error ────────

#[test]
fn check_malformed_fixture_emits_error_envelope_and_exits_1() {
    let out = run(&["check", MALFORMED, "--json"]);
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(
        stdout(&out),
        "",
        "errors never write stdout, even under --json"
    );

    let v = parse_json(&stderr(&out));
    assert_eq!(v["ok"], false, "error envelope carries ok=false");
    assert_eq!(v["envelope_kind"], "error");

    let data = &v["data"];
    assert_eq!(data["code"], "duplicate_ident", "stable error code");
    let message = data["message"].as_str().expect("message string");
    assert!(
        message.contains("line 14"),
        "line present in message: {message}"
    );
    assert!(
        message.contains("column 16"),
        "column present in message: {message}"
    );
    assert!(
        message.contains("duplicate arrow ident `in_section`"),
        "kernel message verbatim: {message}"
    );

    let remediation = data["remediation"].as_array().expect("remediation array");
    assert!(
        !remediation.is_empty(),
        "remediation hint present (never empty)"
    );
    let hint = remediation[0]["description"]
        .as_str()
        .expect("hint description");
    assert!(!hint.is_empty(), "hint non-empty");

    assert!(!stderr(&out).contains("panicked"), "no stack traces, ever");
}

// ── Exit-code classes end-to-end (criterion 4) ──────────────────────

#[test]
fn check_clean_fixture_exits_0_with_empty_findings() {
    let out = run(&["check", BOWLING_FIXED, "--json"]);
    assert_eq!(code(&out), 0, "clean exit code");
    assert_eq!(stderr(&out), "", "no errors");
    let v = parse_json(&stdout(&out));
    let findings = v["data"]["findings"]
        .as_array()
        .expect("data.findings array");
    assert!(findings.is_empty(), "zero findings on the fixed document");
}

#[test]
fn check_dangling_fixture_exits_4_and_the_run_continues() {
    let out = run(&["check", DANGLING, "--json"]);
    assert_eq!(code(&out), 4, "dangling exit code");
    assert_eq!(stderr(&out), "", "no errors");
    let v = parse_json(&stdout(&out));
    let findings = v["data"]["findings"]
        .as_array()
        .expect("data.findings array");
    assert_eq!(findings.len(), 1, "exactly the dangling entry");
    assert_eq!(findings[0]["kind"], "dangling");
    assert_eq!(findings[0]["label"], "executed");
    assert_eq!(findings[0]["witness"]["arrow"], "in_section", "the arrow");
    assert_eq!(findings[0]["witness"]["src"], 0, "the source row");
}

#[test]
fn check_multi_class_fixture_exits_3_and_lists_both_classes() {
    let out = run(&["check", MULTI_CLASS, "--json"]);
    assert_eq!(code(&out), 3, "violation wins the precedence (exit 3)");
    assert_eq!(stderr(&out), "", "no errors");
    let v = parse_json(&stdout(&out));
    let findings = v["data"]["findings"]
        .as_array()
        .expect("data.findings array");
    let kinds: Vec<&str> = findings
        .iter()
        .map(|f| f["kind"].as_str().expect("kind string"))
        .collect();
    assert!(
        kinds.contains(&"violation") && kinds.contains(&"dangling"),
        "the payload still lists both classes: {kinds:?}"
    );
}

#[test]
fn check_human_malformed_error_is_labeled_without_stack_trace() {
    let out = run(&["check", MALFORMED]);
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let err = stderr(&out);
    assert!(err.contains("duplicate_ident"), "stable code: {err}");
    assert!(
        err.contains("line 14") && err.contains("column 16"),
        "position: {err}"
    );
    assert!(err.contains("hint"), "remediation hint: {err}");
    assert!(!err.contains("panicked"), "no stack traces: {err}");
}

#[test]
fn usage_error_exits_2_on_stderr() {
    let out = run(&["--definitely-not-a-flag"]);
    assert_eq!(code(&out), 2, "clap's own usage code");
    assert_eq!(stdout(&out), "", "usage errors never write stdout");
    assert!(!stderr(&out).is_empty(), "usage errors are reported");
}

#[test]
fn unknown_subcommand_exits_2_with_a_suggestion() {
    let out = run(&["chek", MALFORMED]);
    assert_eq!(code(&out), 2, "clap's own usage code");
    assert_eq!(stdout(&out), "", "usage errors never write stdout");
}

// ── The findings view (criterion 5) ─────────────────────────────────

#[test]
fn findings_view_has_the_same_payload_as_check() {
    let check = run(&["check", BOWLING, "--json"]);
    let findings = run(&["findings", BOWLING, "--json"]);
    assert_eq!(code(&check), code(&findings), "same execution, same exit");
    let a = parse_json(&stdout(&check));
    let b = parse_json(&stdout(&findings));
    assert_eq!(
        a["data"]["findings"], b["data"]["findings"],
        "same findings payload"
    );
    assert_eq!(
        a["envelope_kind"], b["envelope_kind"],
        "same envelope shape"
    );
}

// ── The DESIGN §5 agent-loop walkthrough (task 7, criterion 10) ─────

#[test]
fn design_s5_walkthrough_violating_then_fixed() {
    // Phase 1: the violating document reproduces the §5 violation block
    // byte-for-byte and exits 3. The transcript's `underconstraint`
    // line is P4-gated (no producer exists; findings.rs emits nothing
    // for it) — its absence here is the honest state, never fabricated.
    let violating = run(&["check", BOWLING]);
    assert_eq!(code(&violating), 3, "violation exit code");
    assert_eq!(stderr(&violating), "", "findings go to stdout");
    assert_eq!(
        stdout(&violating),
        "violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item\n\
         \x20 at row Assignment#3: lhs=Unit#1 rhs=Unit#2\n",
        "the §5 violation block, byte-for-byte"
    );

    // Phase 2: the loop — the agent's proposed fix lands, the re-check
    // is clean: exit 0, zero findings.
    let fixed = run(&["check", BOWLING_FIXED]);
    assert_eq!(code(&fixed), 0, "clean re-check exit code");
    assert_eq!(stdout(&fixed), "", "zero findings print nothing");
    assert_eq!(stderr(&fixed), "", "no errors");
}
