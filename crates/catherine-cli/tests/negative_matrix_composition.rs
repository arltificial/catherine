//! Purpose: the composition surface of the negative-input matrix
//! (catherine-0qz, openspec change `add-model-composition`), probing every
//! malformed or conflicting joint invocation against the real binary and
//! asserting the honest outcome per row.
//! Responsibilities:
//!   - The composition fixture paths: the sharing gap-suite billing
//!     document, the disagreeing-endpoint pair, the disagreeing-row pair,
//!     the dangling cross-document seam claim, the disjoint pair, and a
//!     fixture that shares nothing with the billing document.
//!   - The `Expect` vocabulary shared by the matrix (labeled error, usage
//!     error, envelope warning) and the row tables over it.
//!   - The recurrence gate: every error row surfaces its malformed input
//!     from stderr — never silence — and every warning row carries its
//!     labeled rule on the envelope.
//!
//! Rationale: split into its own binary like `negative_matrix_projection`
//! and `negative_matrix_naturality` so the composition rows stay inside
//! the file-size gate and never weaken another surface's rows. Design
//! Decision 2 makes conflicts labeled errors (never a last-writer-wins
//! union), so each row pins the stable code and exit 1; the arity itself
//! must keep surfacing usage errors (exit 2) for bad flags.

use serde_json::Value;
use std::process::{Command, Output};

const BILLING: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gap_suite/composition_billing.ct.json"
);
const ENDPOINTS_A: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/endpoints_a.ct.json"
);
const ENDPOINTS_B: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/endpoints_b.ct.json"
);
const ROWS_A: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/rows_a.ct.json"
);
const ROWS_B: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/rows_b.ct.json"
);
const DANGLING: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/dangling_seam.ct.json"
);
const DISJOINT_A: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/disjoint_a.ct.json"
);
const DISJOINT_B: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/composition/disjoint_b.ct.json"
);
/// A parseable document whose schema is malformed (duplicate arrow ident).
const MALFORMED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/malformed.ct.json"
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

fn parse_json(text: &str, row: &str) -> Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("[{row}] stderr is one JSON envelope: {e}\n{text}"))
}

/// The honest outcome a row must assert — never silence.
#[derive(Debug)]
enum Expect {
    /// Labeled error: exit 1, error envelope on stderr with a stable code
    /// and a non-empty message; stdout stays empty.
    Error { code: &'static str },
    /// Usage error: clap's own exit 2, its error text on stderr, stdout
    /// stays empty.
    Usage,
}

/// A warning-class row: the run proceeds (exit 0) but the envelope's
/// warnings channel must carry the labeled rule — never a signal-free
/// clean exit.
struct WarnRow {
    /// `surface × scenario` — printed in every failure message.
    name: &'static str,
    args: &'static [&'static str],
    rule: &'static str,
}

struct Row {
    /// `surface × class` — printed in every failure message.
    name: &'static str,
    args: &'static [&'static str],
    expect: Expect,
}

/// One row per conflicting or malformed joint invocation; `--json` so the
/// error envelope is assertable per row.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    Row { name: "composition × disagreeing arrow endpoints (a name declared twice with different structure)",
          args: &["check", ENDPOINTS_A, ENDPOINTS_B, "--json"],
          expect: Expect::Error { code: "schema_conflict" } },
    Row { name: "composition × disagreeing row values (a shared row valued differently in a common arrow)",
          args: &["check", ROWS_A, ROWS_B, "--json"],
          expect: Expect::Error { code: "row_conflict" } },
    Row { name: "composition × dangling cross-document reference (a seam equation names no input's arrow)",
          args: &["check", BILLING, DANGLING, "--json"],
          expect: Expect::Error { code: "unknown_arrow" } },
    Row { name: "composition × malformed first document (the first input fails its document parse)",
          args: &["check", MALFORMED, BILLING, "--json"],
          expect: Expect::Error { code: "duplicate_ident" } },
    Row { name: "composition × malformed second document (the second input fails its document parse)",
          args: &["check", BILLING, MALFORMED, "--json"],
          expect: Expect::Error { code: "duplicate_ident" } },
    Row { name: "composition × unknown flag (arity support never weakens clap's usage errors)",
          args: &["check", BILLING, ENDPOINTS_A, "--bogus", "--json"],
          expect: Expect::Usage },
    Row { name: "composition × underconstraint malformed second document (the joint analysis surface parses every input)",
          args: &["underconstraint", BILLING, MALFORMED, "--json"],
          expect: Expect::Error { code: "duplicate_ident" } },
    Row { name: "composition × underconstraint disagreeing arrow endpoints (the joint analysis surface refuses a bad overlap)",
          args: &["underconstraint", ENDPOINTS_A, ENDPOINTS_B, "--json"],
          expect: Expect::Error { code: "schema_conflict" } },
];

/// The envelope-warning class: a joint run that proceeds must still label
/// its weakened reading. `findings` shares the `check` execution, so the
/// variadic form is exercised on both names.
#[rustfmt::skip]
const WARNING_MATRIX: &[WarnRow] = &[
    WarnRow { name: "composition × disjoint documents (check) — the warning class is labeled, never a signal-free clean exit",
              args: &["check", DISJOINT_A, DISJOINT_B, "--json"],
              rule: "empty_composition_interface" },
    WarnRow { name: "composition × disjoint documents (findings) — the variadic findings surface carries the same label",
              args: &["findings", BILLING, DISJOINT_A, "--json"],
              rule: "empty_composition_interface" },
    WarnRow { name: "composition × repeated document path (check) — the duplicate input is named, never silently deduplicated",
              args: &["check", DISJOINT_A, DISJOINT_A, "--json"],
              rule: "duplicate_document_input" },
];

/// The recurrence gate: every row must surface its malformed input
/// honestly — labeled error or usage error — never silence.
#[test]
fn composition_negative_matrix_rows_surface_honestly() {
    for Row { name, args, expect } in MATRIX {
        let out = run(args);
        match expect {
            Expect::Error { code: expected } => {
                assert_eq!(code(&out), 1, "[{name}] labeled errors exit 1");
                assert_eq!(stdout(&out), "", "[{name}] errors never write stdout");
                let value = parse_json(&stderr(&out), name);
                assert_eq!(value["ok"], false, "[{name}] error envelope is a failure");
                assert_eq!(value["envelope_kind"], "error", "[{name}] envelope kind");
                assert_eq!(
                    value["data"]["code"], *expected,
                    "[{name}] stable conflict code"
                );
                assert!(
                    !value["data"]["message"]
                        .as_str()
                        .unwrap_or_default()
                        .is_empty(),
                    "[{name}] a labeled error carries a message"
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

/// The warning rows: a disjoint union (or a variadic findings run over
/// disjoint documents) exits clean but names the `empty_composition_interface`
/// rule on the envelope — the honest outcome, never silence.
#[test]
fn composition_warning_rows_are_labeled_on_the_envelope() {
    for WarnRow { name, args, rule } in WARNING_MATRIX {
        let out = run(args);
        assert_eq!(code(&out), 0, "[{name}] a warned run is not an error");
        assert_eq!(
            stderr(&out),
            "",
            "[{name}] JSON warnings live on the envelope, not stderr"
        );
        let value = parse_json(&stdout(&out), name);
        assert_eq!(value["ok"], true, "[{name}] the envelope is a success");
        let rules: Vec<&str> = value["warnings"]
            .as_array()
            .map(|warnings| {
                warnings
                    .iter()
                    .filter_map(|warning| warning["rule_name"].as_str())
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            rules.contains(rule),
            "[{name}] the labeled warning is present: {rules:?}"
        );
    }
}
