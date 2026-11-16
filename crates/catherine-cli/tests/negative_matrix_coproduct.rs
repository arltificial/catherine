//! Purpose: the coproduct surface of the negative-input matrix
//! (catherine-iib, PREVENTION-A; openspec change
//! `add-coproduct-exhaustiveness`), probing every malformed
//! `schema.coproducts` claim against the real binary and asserting the
//! honest labeled error per row.
//!
//! Responsibilities: the malformed-coproduct fixture paths; the `Expect`
//! vocabulary (labeled error or usage error) and the row table over it;
//! the recurrence gate that every row surfaces its malformed input from
//! stderr — never silence.
//!
//! Rationale: the parse discipline mirrors `invalid_key` (design
//! Decision 1), so an undeclared object or arrow, a non-injection arrow
//! (wrong target, or sourced at the coproduct object itself), an empty
//! arrow list, and a duplicate arrow are one stable code at the offending
//! node, while an absent `arrows` field is the positioned serde syntax
//! error (the keys shape) and a bad status reuses the shared
//! `invalid_status` (the keys precedent). A bad flag is clap's usage
//! error. Position assertions stay at the message level (the envelope
//! carries the kernel's positioned message, never separate line/column
//! fields).

use std::process::{Command, Output};

const UNDECLARED_OBJECT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-undeclared-object.ct.json"
);
const UNDECLARED_ARROW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-undeclared-arrow.ct.json"
);
const WRONG_DIRECTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-wrong-direction.ct.json"
);
const SELF_INJECTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-self-injection.ct.json"
);
const MISSING_ARROWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-missing-arrows.ct.json"
);
const EMPTY_ARROWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-empty-arrows.ct.json"
);
const DUPLICATE_ARROW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-duplicate-arrow.ct.json"
);
const BAD_STATUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/malformed-status.ct.json"
);
const CLEAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/coproduct/clean.ct.json"
);

/// Spawn the `catherine` binary (the matrix helper of the other suites).
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
        .unwrap_or_else(|e| panic!("[{row}] stderr is one JSON envelope: {e}\n{text}"))
}

/// The honest outcome a row must assert — never silence.
#[derive(Debug)]
enum Expect {
    /// Labeled error: exit 1, an error envelope on stderr with a stable
    /// code and a positioned, non-empty message; stdout stays empty.
    Error { code: &'static str },
    /// Usage error: clap's own exit 2, its error text on stderr.
    Usage,
}

struct Row {
    /// `surface × class` — printed in every failure message.
    name: &'static str,
    args: &'static [&'static str],
    expect: Expect,
}

/// One row per malformed coproduct claim; `--json` so the envelope
/// channels are assertable per row.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    Row { name: "coproduct × undeclared object (the coproduct's object is not a schema object)",
          args: &["check", UNDECLARED_OBJECT, "--json"],
          expect: Expect::Error { code: "invalid_coproduct" } },
    Row { name: "coproduct × undeclared arrow (an injection names no declared arrow)",
          args: &["check", UNDECLARED_ARROW, "--json"],
          expect: Expect::Error { code: "invalid_coproduct" } },
    Row { name: "coproduct × wrong-direction arrow (not a V_k -> C injection)",
          args: &["check", WRONG_DIRECTION, "--json"],
          expect: Expect::Error { code: "invalid_coproduct" } },
    Row { name: "coproduct × self-sourced arrow (an injection cannot start at the coproduct object)",
          args: &["check", SELF_INJECTION, "--json"],
          expect: Expect::Error { code: "invalid_coproduct" } },
    Row { name: "coproduct × absent arrows field (the required injection list)",
          args: &["check", MISSING_ARROWS, "--json"],
          expect: Expect::Error { code: "syntax" } },
    Row { name: "coproduct × empty arrow list (a coverage claim needs an injection)",
          args: &["check", EMPTY_ARROWS, "--json"],
          expect: Expect::Error { code: "invalid_coproduct" } },
    Row { name: "coproduct × duplicate arrow name (the same injection twice)",
          args: &["check", DUPLICATE_ARROW, "--json"],
          expect: Expect::Error { code: "invalid_coproduct" } },
    Row { name: "coproduct × bad status (the shared accepted/proposed enum)",
          args: &["check", BAD_STATUS, "--json"],
          expect: Expect::Error { code: "invalid_status" } },
    Row { name: "coproduct × bad flag syntax (unknown flag)",
          args: &["check", CLEAN, "--bogus", "--json"],
          expect: Expect::Usage },
];

/// The recurrence gate: every row surfaces its malformed input honestly —
/// a labeled error or a usage error, never silence.
#[test]
fn coproduct_negative_matrix_rows_surface_honestly() {
    for Row { name, args, expect } in MATRIX {
        let out = run(args);
        match expect {
            Expect::Error { code: expected } => {
                assert_eq!(code(&out), 1, "[{name}] labeled errors exit 1");
                assert_eq!(stdout(&out), "", "[{name}] errors never write stdout");
                let v = parse_json(&stderr(&out), name);
                assert_eq!(v["ok"], false, "[{name}] error envelope is a failure");
                assert_eq!(v["envelope_kind"], "error", "[{name}] envelope kind");
                assert_eq!(v["data"]["code"], *expected, "[{name}] stable error code");
                let message = v["data"]["message"].as_str().unwrap_or_default();
                assert!(
                    message.contains("line ") && message.contains("column "),
                    "[{name}] the error carries its node position: {message}"
                );
            }
            Expect::Usage => {
                assert_eq!(code(&out), 2, "[{name}] usage errors exit 2");
                assert_eq!(stdout(&out), "", "[{name}] usage errors never write stdout");
                let text = stderr(&out);
                assert!(
                    text.contains("error:"),
                    "[{name}] clap's error text is on stderr: {text}"
                );
                assert!(!text.contains("panicked"), "[{name}] never a panic: {text}");
            }
        }
    }
}
