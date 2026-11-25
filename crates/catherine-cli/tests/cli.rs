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
//! fabricated end-to-end fixtures. The spawn/parse plumbing lives in
//! `tests/common` (`try_run`/`envelope`/`findings`), spelled for `?` so
//! each test propagates instead of unwrapping.

mod common;

#[path = "cli/walkthrough.rs"]
mod walkthrough;

use common::{code, envelope, findings, stderr, stdout, try_run};
use std::error::Error;

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
const BAD_VERSION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bad_version.ct.json"
);
const MULTI_CLASS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/multi_class.ct.json"
);
const MALFORMED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/malformed.ct.json"
);
const ZERO_ROWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gap_suite/zero_rows_instance.ct.json"
);
const CONSTRAINT_FREE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/scope_constraint_free.ct.json"
);

// ── RED task 1: the violating fixture pins the envelope + exit ──────

/// The §5 violation finding's pins: the executed label and the
/// equation/row/lhs/rhs witness of the bowling fixture.
fn assert_violation_witness(finding: &serde_json::Value) {
    assert_eq!(
        finding["label"], "executed",
        "accepted equation fails as executed"
    );
    assert_eq!(
        finding["witness"]["equation"], 0,
        "equation id (canonical index)"
    );
    assert_eq!(finding["witness"]["row"], 3, "row id Assignment#3");
    assert_eq!(finding["witness"]["lhs"], 1, "path value Unit#1");
    assert_eq!(finding["witness"]["rhs"], 2, "path value Unit#2");
}

#[test]
fn check_violating_fixture_emits_envelope_and_exits_3() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", BOWLING, "--json"])?;
    assert_eq!(code(&out), 3, "violation exit code");
    assert_eq!(
        stderr(&out),
        "",
        "the error stream stays empty on a finding run"
    );

    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["ok"], true, "findings envelope is a success envelope");
    for key in ["ok", "envelope_kind", "data", "warnings", "hints"] {
        assert!(v.get(key).is_some(), "envelope key `{key}` present");
    }

    let found = findings(&v);
    assert_eq!(found.len(), 1, "exactly one violation entry");
    assert_eq!(found[0]["kind"], "violation");
    assert_violation_witness(&found[0]);
    Ok(())
}

// ── RED task 2: the malformed fixture pins the labeled error ────────

/// The malformed fixture's error envelope shape: `ok=false`, the error
/// kind, and the stable `duplicate_ident` code.
fn assert_malformed_envelope(v: &serde_json::Value) {
    assert_eq!(v["ok"], false, "error envelope carries ok=false");
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "duplicate_ident", "stable error code");
}

/// The malformed fixture's labeled message: line/column positions and
/// the kernel message verbatim, plus a non-empty remediation hint.
fn assert_malformed_message(v: &serde_json::Value) {
    let message = v["data"]["message"].as_str().expect("message string");
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
    let remediation = v["data"]["remediation"]
        .as_array()
        .expect("remediation array");
    assert!(
        !remediation.is_empty(),
        "remediation hint present (never empty)"
    );
    let hint = remediation[0]["description"]
        .as_str()
        .expect("hint description");
    assert!(!hint.is_empty(), "hint non-empty");
}

#[test]
fn check_malformed_fixture_emits_error_envelope_and_exits_1() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", MALFORMED, "--json"])?;
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(
        stdout(&out),
        "",
        "errors never write stdout, even under --json"
    );

    let v = envelope("stderr", &stderr(&out))?;
    assert_malformed_envelope(&v);
    assert_malformed_message(&v);
    assert!(!stderr(&out).contains("panicked"), "no stack traces, ever");
    Ok(())
}

// ── Exit-code classes end-to-end (criterion 4) ──────────────────────

#[test]
fn check_clean_fixture_exits_0_with_empty_findings() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", BOWLING_FIXED, "--json"])?;
    assert_eq!(code(&out), 0, "clean exit code");
    assert_eq!(stderr(&out), "", "no errors");
    let v = envelope("stdout", &stdout(&out))?;
    assert!(
        findings(&v).is_empty(),
        "zero findings on the fixed document"
    );
    Ok(())
}

#[test]
fn check_dangling_fixture_exits_4_and_the_run_continues() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", DANGLING, "--json"])?;
    assert_eq!(code(&out), 4, "dangling exit code");
    assert_eq!(stderr(&out), "", "no errors");
    let v = envelope("stdout", &stdout(&out))?;
    let found = findings(&v);
    assert_eq!(found.len(), 1, "exactly the dangling entry");
    assert_eq!(found[0]["kind"], "dangling");
    assert_eq!(found[0]["label"], "executed");
    assert_eq!(found[0]["witness"]["arrow"], "in_section", "the arrow");
    assert_eq!(found[0]["witness"]["src"], 0, "the source row");
    Ok(())
}

#[test]
fn check_multi_class_fixture_exits_3_and_lists_both_classes() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", MULTI_CLASS, "--json"])?;
    assert_eq!(code(&out), 3, "violation wins the precedence (exit 3)");
    assert_eq!(stderr(&out), "", "no errors");
    let v = envelope("stdout", &stdout(&out))?;
    let kinds: Vec<&str> = findings(&v)
        .iter()
        .map(|f| f["kind"].as_str().expect("kind string"))
        .collect();
    assert!(
        kinds.contains(&"violation") && kinds.contains(&"dangling"),
        "the payload still lists both classes: {kinds:?}"
    );
    Ok(())
}

#[test]
fn check_human_malformed_error_is_labeled_without_stack_trace() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", MALFORMED])?;
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
    Ok(())
}

#[test]
fn usage_error_exits_2_on_stderr() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["--definitely-not-a-flag"])?;
    assert_eq!(code(&out), 2, "clap's own usage code");
    assert_eq!(stdout(&out), "", "usage errors never write stdout");
    assert!(!stderr(&out).is_empty(), "usage errors are reported");
    Ok(())
}

#[test]
fn unknown_subcommand_exits_2_with_a_suggestion() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["chek", MALFORMED])?;
    assert_eq!(code(&out), 2, "clap's own usage code");
    assert_eq!(stdout(&out), "", "usage errors never write stdout");
    Ok(())
}

// ── The findings view (criterion 5) ─────────────────────────────────

#[test]
fn findings_view_has_the_same_payload_as_check() -> Result<(), Box<dyn Error>> {
    let check = try_run(&["check", BOWLING, "--json"])?;
    let findings_run = try_run(&["findings", BOWLING, "--json"])?;
    assert_eq!(
        code(&check),
        code(&findings_run),
        "same execution, same exit"
    );
    let a = envelope("stdout", &stdout(&check))?;
    let b = envelope("stdout", &stdout(&findings_run))?;
    assert_eq!(
        a["data"]["findings"], b["data"]["findings"],
        "same findings payload"
    );
    assert_eq!(
        a["envelope_kind"], b["envelope_kind"],
        "same envelope shape"
    );
    Ok(())
}

// ── The DESIGN §5 agent-loop walkthrough (task 7, criterion 10) ─────
// Pinned in the `walkthrough` submodule.

// ── The execution scope (openspec change add-execution-scope) ───────

/// The scope object's exact three-key shape, asserted per payload.
fn assert_scope_keys(scope: &serde_json::Value, where_: &str) {
    let mut keys = scope
        .as_object()
        .unwrap_or_else(|| panic!("{where_}: data.scope is an object"))
        .keys()
        .map(|key| key.as_str())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "constraints_checked",
            "equations_over_zero_rows",
            "rows_evaluated"
        ],
        "{where_}: exactly the three snake_case keys"
    );
}

#[test]
fn check_payload_carries_the_execution_scope() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", BOWLING, "--json"])?;
    assert_eq!(code(&out), 3, "exit code unchanged by the scope field");
    let v = envelope("stdout", &stdout(&out))?;
    let scope = &v["data"]["scope"];
    assert_scope_keys(scope, "check");
    assert_eq!(
        scope["constraints_checked"], 1,
        "the document's one accepted equation"
    );
    assert_eq!(
        scope["rows_evaluated"], 14,
        "all instance rows: 4+4+3+3 across the four objects"
    );
    assert_eq!(
        scope["equations_over_zero_rows"], 0,
        "the equation ranges over four Assignment rows"
    );
    Ok(())
}

#[test]
fn check_and_findings_agree_on_the_scope() -> Result<(), Box<dyn Error>> {
    let check = try_run(&["check", BOWLING, "--json"])?;
    let findings_run = try_run(&["findings", BOWLING, "--json"])?;
    assert_eq!(
        envelope("stdout", &stdout(&check))?["data"]["scope"],
        envelope("stdout", &stdout(&findings_run))?["data"]["scope"],
        "one scope computation shared by both subcommands"
    );
    Ok(())
}

#[test]
fn zero_row_instance_reports_its_vacuous_scope_and_keeps_exit_0() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", ZERO_ROWS, "--json"])?;
    assert_eq!(code(&out), 0, "exit code unchanged for the vacuous run");
    assert_eq!(stderr(&out), "", "no errors");
    let v = envelope("stdout", &stdout(&out))?;
    let scope = &v["data"]["scope"];
    assert_scope_keys(scope, "zero-row");
    assert_eq!(
        scope["constraints_checked"], 1,
        "the accepted equation was applicable"
    );
    assert_eq!(scope["rows_evaluated"], 0, "the instance has no rows");
    assert_eq!(
        scope["equations_over_zero_rows"], 1,
        "the equation binds zero rows — the visible vacuity"
    );
    Ok(())
}

#[test]
fn constraint_free_document_reports_zero_constraints_checked() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", CONSTRAINT_FREE, "--json"])?;
    assert_eq!(code(&out), 0, "exit code unchanged");
    assert_eq!(stderr(&out), "", "no errors");
    let v = envelope("stdout", &stdout(&out))?;
    let scope = &v["data"]["scope"];
    assert_scope_keys(scope, "constraint-free");
    assert_eq!(
        scope["constraints_checked"], 0,
        "no constraints — nothing was checked, visibly"
    );
    assert_eq!(scope["rows_evaluated"], 2, "the instance still has rows");
    assert_eq!(scope["equations_over_zero_rows"], 0, "no equations at all");
    Ok(())
}

/// catherine-uh0: the `unsupported_version` hint states the real
/// KNOWN_MINOR policy — the known-version cap — and no longer claims
/// additive minors are accepted.
#[test]
fn unsupported_version_hint_states_known_version_policy() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", BAD_VERSION])?;
    assert_eq!(code(&out), 1, "unsupported version exits 1");
    let err = stderr(&out);
    assert!(
        !err.contains("additive minors"),
        "the stale 'minors accepted' claim is gone: {err:?}"
    );
    assert!(
        err.contains("1.6") && err.contains("KNOWN_MINOR"),
        "the hint states the known version and the KNOWN_MINOR cap: {err:?}"
    );
    Ok(())
}
