//! Purpose: the `catherine check` acceptance surface for pullbacks
//! (openspec change `add-pullback-checks`): a declared
//! `schema.pullbacks` square executes commutation, coverage, and
//! uniqueness, and the commutation witness reaches both renderings with
//! the violation exit code.
//!
//! Responsibilities: probe the real binary over the hermetic fixtures —
//! the clean square (zero findings, exit 0) and the non-commuting square
//! (one executed violation carrying the declaration index, the apex row,
//! and two unequal composed values; exit 3) — and pin the human line's
//! declaration head and composed values.
//!
//! Rationale: the finding JSON is the kernel's encoding printed verbatim
//! (no CLI reshaping), so the `--json` assertions are structural on
//! `data.findings` and the human assertions name only the facts the §5
//! block must carry (`pullback-0`, the apex row, both `Object#n` values).
//! The web panel's pullback union branch is the wasm boundary's concern
//! and is not exercised by this binary.

mod common;

use common::{code, findings, parse_json, run, stderr, stdout};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/");

fn fixture(name: &str) -> String {
    format!("{FIXTURES}{name}")
}

/// A square whose unique apex row commutes: the bijection holds, so the
/// run is evaluated-clean and silent.
#[test]
fn clean_square_is_silent() {
    let out = run(&["check", &fixture("pullback_square.ct.json"), "--json"]);
    assert_eq!(
        code(&out),
        0,
        "a commuting square exits 0: {}",
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "a clean run writes no diagnostics");
    let envelope = parse_json(&stdout(&out), "the check envelope");
    assert_eq!(envelope["ok"], true, "the check envelope is a success");
    assert!(
        findings(&envelope).is_empty(),
        "a satisfied pullback emits nothing"
    );
}

/// The one apex row projects to `A#0`/`B#0`, but `f` and `g` send those
/// to different `C` rows: exactly one executed commutation violation.
#[test]
fn non_commuting_square_json_names_the_row_and_values() {
    let out = run(&["check", &fixture("pullback_violation.ct.json"), "--json"]);
    assert_eq!(
        code(&out),
        3,
        "a commutation violation exits 3: {}",
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "no diagnostics on a finding run");
    let envelope = parse_json(&stdout(&out), "the check envelope");
    assert_eq!(envelope["ok"], true, "the check envelope is a success");
    assert_eq!(envelope["envelope_kind"], "check", "the check payload");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "exactly one finding: {found:?}");
    assert_eq!(found[0]["kind"], "violation");
    assert_eq!(found[0]["label"], "executed");
    let witness = &found[0]["witness"];
    assert_eq!(witness["pullback"], 0, "the declaration's index");
    assert_eq!(witness["row"], 0, "`r1` is the only apex row");
    assert_eq!(witness["lhs"], 0, "`f` sends `A#0` to `C#0`");
    assert_eq!(witness["rhs"], 1, "`g` sends `B#0` to `C#1`");
    assert_ne!(
        witness["lhs"], witness["rhs"],
        "the square fails to commute: {witness}"
    );
}

/// The human rendering names the declaration and both composed values in
/// the transcript's `Object#n` spelling.
#[test]
fn non_commuting_square_human_names_the_declaration_and_values() {
    let out = run(&["check", &fixture("pullback_violation.ct.json"), "--human"]);
    assert_eq!(
        code(&out),
        3,
        "the human run keeps the class: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("violation:"),
        "the head is a violation: {text}"
    );
    assert!(
        text.contains("pullback-0"),
        "the declaration's index: {text}"
    );
    assert!(text.contains("P#0"), "the apex row: {text}");
    assert!(
        text.contains("C#0") && text.contains("C#1"),
        "both composed values: {text}"
    );
    assert!(
        text.contains("lhs=") && text.contains("rhs="),
        "the composed values are labeled: {text}"
    );
}
