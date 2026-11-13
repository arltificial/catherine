//! The `rules --project` extension of the negative-input matrix
//! (catherine-348, PREVENTION-A): the same recurrence gate, probing every
//! malformed projection invocation against the real binary. Split out of
//! `negative_matrix.rs` so both files stay inside the file-size gate; no
//! row is weakened and the shared surfaces keep their rows there.
//!
//! Rows: `--project` on a document that already carries a
//! `rule_presentation` (two presentation sources), an undeclared object, an
//! arrow not sourced at the object, a missing `--columns`, `--columns`
//! without `--project`, a null projected cell, and an empty projected
//! object. Each must surface a labeled error (exit 1, error envelope on
//! stderr, stable code, non-empty message) — never silence.

use std::process::{Command, Output};

/// The projection surface's fixtures: instance rows to project, the same
/// rows plus a `rule_presentation` (two presentation sources), a null
/// projected cell, and an object with no rows.
const PROJECTION_ROWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/projection_rows.ct.json"
);
const PROJECTION_WITH_RULES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/projection_with_rules.ct.json"
);
const PROJECTION_NULL_CELL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/projection_null_cell.ct.json"
);
const PROJECTION_EMPTY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/projection_empty.ct.json"
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

/// One row per malformed `rules --project` invocation; `--json` so the
/// error envelope is assertable.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    Row { name: "rules × --project on a document carrying rule_presentation (two sources)",
          args: &["rules", PROJECTION_WITH_RULES, "--project", "capture",
                  "--columns", "capture_source", "--json"],
          code: "bad_argument" },
    Row { name: "rules × --project names an undeclared object",
          args: &["rules", PROJECTION_ROWS, "--project", "ghost",
                  "--columns", "capture_source", "--json"],
          code: "bad_argument" },
    Row { name: "rules × --columns arrow not sourced at the projected object",
          args: &["rules", PROJECTION_ROWS, "--project", "capture",
                  "--columns", "source_edition", "--json"],
          code: "bad_argument" },
    Row { name: "rules × --project without --columns",
          args: &["rules", PROJECTION_ROWS, "--project", "capture", "--json"],
          code: "bad_argument" },
    Row { name: "rules × --columns without --project",
          args: &["rules", PROJECTION_ROWS, "--columns", "capture_source", "--json"],
          code: "bad_argument" },
    Row { name: "rules × null projected cell (no silent row drop)",
          args: &["rules", PROJECTION_NULL_CELL, "--project", "capture",
                  "--columns", "capture_source", "--json"],
          code: "projection_incomplete" },
    Row { name: "rules × empty projected object (no vacuous clean run)",
          args: &["rules", PROJECTION_EMPTY, "--project", "capture",
                  "--columns", "capture_source", "--json"],
          code: "empty_projection" },
];

/// The recurrence gate: every row must surface its malformed input
/// honestly — a labeled error, never silence.
#[test]
fn projection_negative_matrix_rows_surface_honestly() {
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
