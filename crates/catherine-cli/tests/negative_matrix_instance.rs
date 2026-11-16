//! The instance-shape extension of the negative-input matrix
//! (catherine-3en, arc-review MEDIUM-005): the same recurrence gate, probing
//! the legacy per-object instance spelling against the real binary. Split out
//! of `negative_matrix.rs` so both files stay inside the file-size gate; the
//! row is unweakened and the shared surfaces keep their rows there.
//!
//! Rows: `check` on a document whose `instance` object carries neither
//! `rows` nor `tables` — only the unrecognized key `N` with per-object row
//! lists (the legacy spelling). The parser must refuse the shape as a
//! labeled `syntax` error (exit 1, error envelope on stderr, non-empty
//! message naming the missing keys and the key it saw) — never read it as
//! an empty instance and check clean.
//!
//! Rationale: unknown keys are deliberately ignored (additive minors add
//! optional fields), but that tolerance presupposes recognizable content
//! alongside them. An instance object whose whole content is unrecognized
//! is a shape this build cannot read; reporting it as empty is a class-A
//! silent reinterpretation, not tolerance. The message half is pinned
//! separately below so the diagnosis, not just the channel, is a contract.

use std::process::{Command, Output};

/// The gap-suite fixture for the legacy per-object instance spelling
/// (`instance.N: [{id, f}]`, no `rows`/`tables`): a shape this build cannot
/// read, refused as `syntax`, never an empty instance.
const WRONG_SHAPE_INSTANCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gap_suite/wrong_shape_instance.ct.json"
);

/// Spawn the `catherine` binary with the given arguments (same helper as
/// the other CLI suites).
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

fn parse_json(text: &str, row: &str) -> serde_json::Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("[{row}] stdout/stderr is one JSON envelope: {e}\n{text}"))
}

struct Row {
    /// `surface × class` — printed in every failure message.
    name: &'static str,
    args: &'static [&'static str],
    /// The stable labeled-error code the row must surface.
    code: &'static str,
}

/// One row per unreadable instance invocation; `--json` so the error
/// envelope is assertable.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    Row { name: "check × unreadable instance shape (legacy per-object keys, no `rows`/`tables`)",
          args: &["check", WRONG_SHAPE_INSTANCE, "--json"],
          code: "syntax" },
];

/// The recurrence gate: every row must surface its malformed input
/// honestly — a labeled error, never silence.
#[test]
fn instance_shape_negative_matrix_rows_surface_honestly() {
    for Row {
        name,
        args,
        code: expected,
    } in MATRIX
    {
        let out = run(args);
        assert_eq!(code(&out), 1, "[{name}] labeled errors exit 1");
        assert_eq!(stdout(&out), "", "[{name}] errors never write stdout");
        let v = parse_json(&stderr(&out), name);
        assert_eq!(v["ok"], false, "[{name}] error envelope is a failure");
        assert_eq!(v["envelope_kind"], "error", "[{name}] envelope kind");
        assert_eq!(v["data"]["code"], *expected, "[{name}] stable error code");
        assert!(
            !v["data"]["message"].as_str().unwrap_or_default().is_empty(),
            "[{name}] a labeled error carries a message"
        );
    }
}

/// The shape row's message half: the rejection must name the missing
/// `rows`/`tables` keys *and* the key it saw instead, so a reader can tell
/// the shape was refused rather than the document being unreadable JSON.
/// The row above pins exit/code/channel; this pins the diagnosis.
#[test]
fn unreadable_instance_shape_names_the_rejected_keys() {
    let out = run(&["check", WRONG_SHAPE_INSTANCE, "--json"]);
    assert_eq!(code(&out), 1, "an unreadable instance shape exits 1");
    let v = parse_json(&stderr(&out), "wrong-shape instance");
    assert_eq!(v["data"]["code"], "syntax");
    let message = v["data"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("no `rows` or `tables`") && message.contains("unrecognized keys: `N`"),
        "the rejection names the shape and its keys: {message}"
    );
}
