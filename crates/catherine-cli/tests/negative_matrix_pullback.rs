//! Purpose: the pullback surface of the negative-input matrix
//! (catherine-ih5, from the rule-of-5 review of `add-pullback-checks`),
//! probing every malformed `schema.pullbacks` claim against the real
//! binary and asserting the honest labeled error per row.
//!
//! Responsibilities: the malformed-pullback fixture paths; the `Expect`
//! vocabulary (labeled error or usage error) and the row table over it;
//! the recurrence gate that every row surfaces its malformed input from
//! stderr — never silence — with a message fragment naming the defect.
//!
//! Rationale: the parse discipline mirrors `invalid_coproduct`, so an
//! undeclared object or arrow, a leg not sourced at the apex, a wrong
//! leg/cospan length, disagreeing cospan targets, a repeated arrow, and
//! the composability cause (a leg target that is not its paired cospan
//! arrow's source, both positions) are one stable `invalid_pullback` at
//! the offending node; an absent `legs` field is the positioned serde
//! syntax error and a bad status reuses the shared `invalid_status`.
//! A bad flag is clap's usage error. Position assertions stay at the
//! message level (the envelope carries the kernel's positioned message,
//! never separate line/column fields), and message fragments are asserted
//! in place of snapshots.

use std::process::{Command, Output};

const UNDECLARED_OBJECT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-undeclared-object.ct.json"
);
const UNDECLARED_LEG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-undeclared-leg-arrow.ct.json"
);
const LEG_NOT_AT_APEX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-leg-not-at-apex.ct.json"
);
const ONE_LEG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-one-leg.ct.json"
);
const THREE_LEGS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-three-legs.ct.json"
);
const UNDECLARED_COSPAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-undeclared-cospan-arrow.ct.json"
);
const ONE_COSPAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-one-cospan.ct.json"
);
const THREE_COSPAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-three-cospan.ct.json"
);
const COSPAN_TARGETS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-cospan-targets.ct.json"
);
const DUPLICATE_ARROW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-duplicate-arrow.ct.json"
);
const LEG0_NOT_COMPOSABLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-leg0-not-composable.ct.json"
);
const LEG1_NOT_COMPOSABLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-leg1-not-composable.ct.json"
);
const BAD_STATUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-status.ct.json"
);
const MISSING_LEGS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/malformed-missing-legs.ct.json"
);
const CLEAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pullback/clean.ct.json"
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
    /// code, a positioned message, and a fragment naming the defect;
    /// stdout stays empty.
    Error {
        code: &'static str,
        fragments: &'static [&'static str],
    },
    /// Usage error: clap's own exit 2, its error text on stderr.
    Usage,
}

struct Row {
    /// `surface × class` — printed in every failure message.
    name: &'static str,
    args: &'static [&'static str],
    expect: Expect,
}

/// One row per malformed pullback claim; `--json` so the envelope
/// channels are assertable per row.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    Row { name: "pullback × undeclared apex object (the apex is not a schema object)",
          args: &["check", UNDECLARED_OBJECT, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["undeclared object"] } },
    Row { name: "pullback × undeclared leg arrow (a leg names no declared arrow)",
          args: &["check", UNDECLARED_LEG, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["undeclared arrow"] } },
    Row { name: "pullback × leg not sourced at the apex (f: A -> C as a leg)",
          args: &["check", LEG_NOT_AT_APEX, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["not at the apex"] } },
    Row { name: "pullback × one leg (a square needs exactly two)",
          args: &["check", ONE_LEG, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["exactly two legs"] } },
    Row { name: "pullback × three legs (a square needs exactly two)",
          args: &["check", THREE_LEGS, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["exactly two legs"] } },
    Row { name: "pullback × undeclared cospan arrow (a cospan names no declared arrow)",
          args: &["check", UNDECLARED_COSPAN, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["undeclared arrow"] } },
    Row { name: "pullback × one cospan arrow (a square needs exactly two)",
          args: &["check", ONE_COSPAN, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["exactly two cospan arrows"] } },
    Row { name: "pullback × three cospan arrows (a square needs exactly two)",
          args: &["check", THREE_COSPAN, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["exactly two cospan arrows"] } },
    Row { name: "pullback × cospan targets differ (the cospan must share one target)",
          args: &["check", COSPAN_TARGETS, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["target different objects"] } },
    Row { name: "pullback × repeated arrow name (the same cospan arrow twice)",
          args: &["check", DUPLICATE_ARROW, "--json"],
          expect: Expect::Error { code: "invalid_pullback", fragments: &["duplicate arrow"] } },
    Row { name: "pullback × leg0 target is not its cospan arrow's source (p1 to A, f from B)",
          args: &["check", LEG0_NOT_COMPOSABLE, "--json"],
          expect: Expect::Error { code: "invalid_pullback",
                                   fragments: &["pullback leg `p1` targets", "not the source",
                                                "cospan arrow `f`"] } },
    Row { name: "pullback × leg1 target is not its cospan arrow's source (p2 to B, g from A)",
          args: &["check", LEG1_NOT_COMPOSABLE, "--json"],
          expect: Expect::Error { code: "invalid_pullback",
                                   fragments: &["pullback leg `p2` targets", "not the source",
                                                "cospan arrow `g`"] } },
    Row { name: "pullback × bad status (the shared accepted/proposed enum)",
          args: &["check", BAD_STATUS, "--json"],
          expect: Expect::Error { code: "invalid_status", fragments: &["pullback status"] } },
    Row { name: "pullback × absent legs field (the required leg list)",
          args: &["check", MISSING_LEGS, "--json"],
          expect: Expect::Error { code: "syntax", fragments: &["missing field `legs`"] } },
    Row { name: "pullback × bad flag syntax (unknown flag)",
          args: &["check", CLEAN, "--bogus", "--json"],
          expect: Expect::Usage },
];

/// The recurrence gate: every row surfaces its malformed input honestly —
/// a labeled error or a usage error, never silence.
#[test]
fn pullback_negative_matrix_rows_surface_honestly() {
    for Row { name, args, expect } in MATRIX {
        let out = run(args);
        match expect {
            Expect::Error {
                code: expected,
                fragments,
            } => {
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
                for fragment in *fragments {
                    assert!(
                        message.contains(*fragment),
                        "[{name}] the message names the defect (`{fragment}`): {message}"
                    );
                }
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
