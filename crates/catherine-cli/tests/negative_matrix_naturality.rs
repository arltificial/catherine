//! Purpose: the `naturality` surface of the negative-input matrix
//! (catherine-348, PREVENTION-A; openspec change `add-instance-morphism`),
//! probing every malformed or absent instance-map claim against the real
//! binary and asserting the honest outcome per row.
//! Responsibilities:
//!   - The naturality fixture paths: the commuting crm → warehouse map, the
//!     target document, and the malformed-claim variants (an incomplete map,
//!     a duplicate entry, an undeclared object, an ill-formed target, a
//!     source document that does not parse at all, and a square undefined on
//!     both sides).
//!   - The `Expect` vocabulary shared by the matrix (labeled error, usage
//!     error, envelope warning) and the row table over it.
//!   - The recurrence gate: every row surfaces its malformed input from
//!     stderr or `warnings` — never silence.
//!
//! Rationale: split out of `negative_matrix.rs` (following
//! `negative_matrix_projection.rs`) because the naturality rows plus
//! their fixtures pushed that binary past the 400-line file-size gate; no
//! row is weakened and the other surfaces keep their rows there. The
//! source document's `instance_map` is the claim being executed, so a
//! malformed claim is a labeled error before any check (design
//! Decision 2) and an absent claim warns instead of returning a silent
//! clean.

use std::process::{Command, Output};

const BAD_VERSION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bad_version.ct.json"
);
/// The naturality fixtures (openspec change `add-instance-morphism`):
/// the crm source with its commuting warehouse map, the warehouse target,
/// and the malformed-claim variants.
const NATURALITY_CRM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm.ct.json"
);
const NATURALITY_WAREHOUSE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/warehouse.ct.json"
);
const NATURALITY_INCOMPLETE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_incomplete_map.ct.json"
);
const NATURALITY_UNKNOWN_TARGET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_unknown_target.ct.json"
);
const NATURALITY_UNDECLARED_OBJECT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_undeclared_object.ct.json"
);
const NATURALITY_BAD_TARGET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_bad_target.ct.json"
);
const NATURALITY_DUPLICATE_ENTRY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_duplicate_entry.ct.json"
);
const NATURALITY_NO_MAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_no_map.ct.json"
);
const NATURALITY_SCHEMA_MISMATCH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/schema_mismatch.ct.json"
);
/// A pair whose `account_owner` square at `a2→w2` dangles on both sides:
/// the operation can verify nothing there and must warn.
const NATURALITY_UNDEFINED_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_undefined.ct.json"
);
const NATURALITY_UNDEFINED_TARGET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/warehouse_undefined.ct.json"
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

fn parse_json(text: &str, row: &str) -> serde_json::Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("[{row}] stdout/stderr is one JSON envelope: {e}\n{text}"))
}

/// The honest outcome a row must assert — never silence.
#[derive(Debug)]
enum Expect {
    /// Labeled error: pinned exit (1), error envelope on stderr with a
    /// stable code and a non-empty message; stdout stays empty.
    Error { code: &'static str },
    /// Usage error: clap's own exit 2, its error text on stderr,
    /// stdout stays empty.
    Usage,
    /// Envelope warning: success envelope on stdout whose `warnings`
    /// name the ignored input (`contains`), pinned exit (the warning
    /// never changes exit semantics), stderr stays empty.
    Warning { exit: i32, contains: &'static str },
}

struct Row {
    /// `surface × class` — printed in every failure message.
    name: &'static str,
    args: &'static [&'static str],
    expect: Expect,
}

/// One row per malformed naturality invocation; `--json` so envelope
/// channels are assertable per row.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    // The source document's `instance_map` is the claim being executed;
    // a malformed claim is a labeled error before any check (design
    // Decision 2), and an absent claim warns instead of returning a
    // silent clean (the matrix's never-silence rule).
    Row { name: "naturality × incomplete instance_map (an uncovered row is a malformed claim)",
          args: &["naturality", NATURALITY_INCOMPLETE, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Error { code: "incomplete_instance_map" } },
    Row { name: "naturality × duplicate map entry (one object mapped twice)",
          args: &["naturality", NATURALITY_DUPLICATE_ENTRY, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Error { code: "duplicate_ident" } },
    Row { name: "naturality × undeclared map object (an entry names no schema object)",
          args: &["naturality", NATURALITY_UNDECLARED_OBJECT, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Error { code: "invalid_ident" } },
    Row { name: "naturality × ill-formed map target (a cell's target is not a row ident)",
          args: &["naturality", NATURALITY_BAD_TARGET, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Error { code: "invalid_ident" } },
    Row { name: "naturality × malformed source document (the claim's own document fails to parse)",
          args: &["naturality", MALFORMED, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Error { code: "duplicate_ident" } },
    Row { name: "naturality × unknown map target (a cell names a target row that does not exist)",
          args: &["naturality", NATURALITY_UNKNOWN_TARGET, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Error { code: "unknown_map_target" } },
    Row { name: "naturality × source document without an instance_map (no silent empty verification)",
          args: &["naturality", NATURALITY_NO_MAP, NATURALITY_WAREHOUSE, "--json"],
          expect: Expect::Warning { exit: 0, contains: "instance_map" } },
    Row { name: "naturality × square undefined on both sides (a non-answer warns, never a silent clean)",
          args: &["naturality", NATURALITY_UNDEFINED_SOURCE, NATURALITY_UNDEFINED_TARGET, "--json"],
          expect: Expect::Warning { exit: 0, contains: "undefined on both sides" } },
    Row { name: "naturality × second-doc schema mismatch (the distinguish check, reused)",
          args: &["naturality", NATURALITY_CRM, NATURALITY_SCHEMA_MISMATCH, "--json"],
          expect: Expect::Error { code: "schema_mismatch" } },
    Row { name: "naturality × malformed document (duplicate arrow ident in the second doc)",
          args: &["naturality", NATURALITY_CRM, MALFORMED, "--json"],
          expect: Expect::Error { code: "duplicate_ident" } },
    Row { name: "naturality × second-doc out-of-range format_version",
          args: &["naturality", NATURALITY_CRM, BAD_VERSION, "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "naturality × bad flag syntax (unknown flag)",
          args: &["naturality", NATURALITY_CRM, NATURALITY_WAREHOUSE, "--bogus", "--json"],
          expect: Expect::Usage },
];

/// The recurrence gate: every row must surface its malformed input
/// honestly — labeled error, usage error, or envelope warning — never
/// silence.
#[test]
fn naturality_negative_matrix_rows_surface_honestly() {
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
                assert!(
                    !v["data"]["message"].as_str().unwrap_or_default().is_empty(),
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
            Expect::Warning {
                exit: expected,
                contains,
            } => {
                assert_eq!(
                    code(&out),
                    *expected,
                    "[{name}] a warning never changes exit semantics"
                );
                assert_eq!(stderr(&out), "", "[{name}] the warning rides stdout");
                let v = parse_json(&stdout(&out), name);
                assert_eq!(v["ok"], true, "[{name}] warning run is a success envelope");
                let warnings = v["warnings"].as_array().unwrap_or_else(|| {
                    panic!("[{name}] warnings array present, got: {}", stdout(&out))
                });
                assert!(
                    warnings.iter().any(|w| w["message"]
                        .as_str()
                        .map(|m| m.contains(contains))
                        .unwrap_or(false)),
                    "[{name}] a warning names the ignored input `{contains}`: {warnings:?}"
                );
            }
        }
    }
}
