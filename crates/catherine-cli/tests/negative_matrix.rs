//! The negative-input matrix (catherine-348, PREVENTION-A): the
//! recurrence gate for class-A fail-open boundaries. Bugs
//! catherine-18t / 51v / 0kw shared one shape — malformed or
//! context-mismatched input at a layer seam was silently dropped or
//! reinterpreted instead of surfacing as a labeled error or an envelope
//! warning. One table-driven test walks every row below and asserts the
//! honest outcome per row: a labeled error (exit 1, error envelope on
//! stderr, stable code, non-empty message) or a usage error (exit 2,
//! clap's error on stderr) or an envelope warning (success envelope on
//! stdout, `warnings` naming the ignored input) — NEVER silence (no
//! exit-0-without-warning, no silently-ignored input).
//!
//! Surfaces × classes: every CLI surface (check, findings, search,
//! redundant, underconstraint, distinguish, query, migrate, rules) gets
//! a row for every malformed-input class that applies to it — undeclared
//! object/arrow names in flags, second-doc schema mismatch, out-of-range
//! `format_version`, unknown seed, uncomposable observation, partial or
//! absent schema map, absent rule presentation, unknown predicate
//! column, invalid policy, garbage rules, garbage instance rows, bad flag
//! syntax (unknown flag → clap exit 2; malformed flag value →
//! `bad_argument` exit 1).
//! Behavior is probed live against the real binary
//! (`CARGO_BIN_EXE_catherine`); assertions are structural
//! (`serde_json::Value`) and numeric (exit codes), never snapshots.
//!
//! `deny_unknown_fields` audit (catherine-348 item 2), adjudicated
//! keep-and-document per the catherine-51v precedent: the parse-layer
//! raw structs (`crates/catherine-core/src/parse/schema.rs`,
//! `RawDoc`/`RawSchema`/`RawArrow`/`RawEquation`/`RawInstance`) are
//! serde `Deserialize` WITHOUT `deny_unknown_fields` (zero occurrences
//! workspace-wide), so unknown JSON keys are silently ignored — and
//! that is the deliberate additive-minor policy, documented user-facing
//! in `crates/catherine-core/README.md` ("Unknown top-level or section
//! keys are ignored: additive minors of v1 may introduce optional
//! fields without breaking this parser" + the `format_version` policy
//! section: additive minors above `KNOWN_MINOR` are *rejected* with
//! `unsupported_version`, never guessed). Unknown keys are therefore
//! not a matrix row: ignoring them is documented policy (b), not a
//! fail-open seam; the version gate pins the semantics-bearing half of
//! that policy (rows below, `unsupported_version`).
//!
//! Review rule (ro5u checklist): a new flag or surface without a
//! negative-matrix row is incomplete.

use std::process::{Command, Output};

const BAD_VERSION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bad_version.ct.json"
);
const ANALYSIS_CONTRADICTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_contradiction.ct.json"
);
const ANALYSIS_UNDERCONSTRAINT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_underconstraint.ct.json"
);
const ANALYSIS_REDUNDANT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_redundant.ct.json"
);
const ANALYSIS_DIST_A: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_dist_a.ct.json"
);
const ANALYSIS_DIST_B: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_dist_b.ct.json"
);
const ANALYSIS_DIST_REVSCHEMA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_dist_revschema.ct.json"
);
const ANALYSIS_QUERY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_query.ct.json"
);
/// The migration fixtures: the committed RED fixture with lost data
/// (`fixtures/migration/`, shared with the kernel suites) and the two
/// migrate-only malformed-input fixtures.
const MIGRATION_DROP_OBJECT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/migration/drop_object.ct.json"
);
const MIGRATE_PARTIAL_MAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/migrate_partial_map.ct.json"
);
const MIGRATE_GARBAGE_ROWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/migrate_garbage_rows.ct.json"
);
/// A pre-map document (parses; carries no `schema_map`).
const MIGRATE_NO_MAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bowling.ct.json"
);
/// The rule lens's clean fixture (`fixtures/rule_lens/`) and the
/// rules-only malformed-input fixtures.
const RULE_LENS_GAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/rule_lens/gap.ct.json"
);
const RULES_CLEAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rules_clean.ct.json"
);
const RULES_UNKNOWN_COLUMN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rules_unknown_column.ct.json"
);
const RULES_BAD_POLICY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rules_bad_policy.ct.json"
);
const RULES_GARBAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rules_garbage.ct.json"
);
/// A pre-1.2 document (parses; carries no `rule_presentation`).
const RULES_NO_SECTION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bowling.ct.json"
);

/// Spawn the `catherine` binary with the given arguments (same helper
/// as the other CLI suites).
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

/// The matrix. One row per surface × applicable malformed-input class;
/// `--json` so envelope channels are assertable per row.
#[rustfmt::skip]
const MATRIX: &[Row] = &[
    // ── check ────────────────────────────────────────────────────────
    Row { name: "check × out-of-range format_version (minor > KNOWN_MINOR)",
          args: &["check", BAD_VERSION, "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "check × bad flag syntax (unknown flag)",
          args: &["check", ANALYSIS_CONTRADICTION, "--bogus", "--json"],
          expect: Expect::Usage },

    // ── findings ─────────────────────────────────────────────────────
    Row { name: "findings × out-of-range format_version (minor > KNOWN_MINOR)",
          args: &["findings", BAD_VERSION, "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "findings × bad flag syntax (unknown flag)",
          args: &["findings", ANALYSIS_CONTRADICTION, "--bogus", "--json"],
          expect: Expect::Usage },

    // ── search ───────────────────────────────────────────────────────
    Row { name: "search × undeclared object in --rows (catherine-0kw: warning, exit unchanged)",
          args: &["search", ANALYSIS_CONTRADICTION, "--json", "--rows", "Objet=4",
                  "--rows", "a=1", "--rows", "b=0", "--rows", "c=1", "--max-candidates", "64"],
          expect: Expect::Warning { exit: 5, contains: "Objet" } },
    Row { name: "search × out-of-range format_version",
          args: &["search", BAD_VERSION, "--rows", "a=1", "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "search × bad flag syntax (malformed --rows value)",
          args: &["search", ANALYSIS_CONTRADICTION, "--rows", "a1", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "search × bad flag syntax (unknown flag)",
          args: &["search", ANALYSIS_CONTRADICTION, "--rows", "a=1", "--bogus", "--json"],
          expect: Expect::Usage },

    // ── redundant ────────────────────────────────────────────────────
    Row { name: "redundant × out-of-range format_version",
          args: &["redundant", BAD_VERSION, "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "redundant × bad flag syntax (unknown flag)",
          args: &["redundant", ANALYSIS_REDUNDANT, "--bogus", "--json"],
          expect: Expect::Usage },

    // ── underconstraint ──────────────────────────────────────────────
    Row { name: "underconstraint × out-of-range format_version",
          args: &["underconstraint", BAD_VERSION, "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "underconstraint × bad flag syntax (unknown flag)",
          args: &["underconstraint", ANALYSIS_UNDERCONSTRAINT, "--bogus", "--json"],
          expect: Expect::Usage },
    Row { name: "underconstraint × bad flag syntax (non-numeric cap value)",
          args: &["underconstraint", ANALYSIS_UNDERCONSTRAINT, "--max-path-length", "x", "--json"],
          expect: Expect::Usage },

    // ── distinguish ──────────────────────────────────────────────────
    Row { name: "distinguish × second-doc schema mismatch (catherine-18t)",
          args: &["distinguish", ANALYSIS_DIST_A, ANALYSIS_DIST_REVSCHEMA,
                  "--observe", "f@a#0", "--json"],
          expect: Expect::Error { code: "schema_mismatch" } },
    Row { name: "distinguish × undeclared arrow in --observe",
          args: &["distinguish", ANALYSIS_DIST_A, ANALYSIS_DIST_B,
                  "--observe", "z@a#0", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "distinguish × uncomposable observation",
          args: &["distinguish", ANALYSIS_DIST_A, ANALYSIS_DIST_B,
                  "--observe", "g@b#0", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "distinguish × second-doc out-of-range format_version",
          args: &["distinguish", ANALYSIS_DIST_A, BAD_VERSION,
                  "--observe", "f@a#0", "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "distinguish × bad flag syntax (malformed --observe spelling)",
          args: &["distinguish", ANALYSIS_DIST_A, ANALYSIS_DIST_B,
                  "--observe", "fa0", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "distinguish × bad flag syntax (unknown flag)",
          args: &["distinguish", ANALYSIS_DIST_A, ANALYSIS_DIST_B,
                  "--observe", "f@a#0", "--bogus", "--json"],
          expect: Expect::Usage },

    // ── query ────────────────────────────────────────────────────────
    Row { name: "query × unknown seed (row outside the object's space)",
          args: &["query", ANALYSIS_QUERY, "--seed", "q#9", "--follow", "g", "--json"],
          expect: Expect::Error { code: "unknown_seed" } },
    Row { name: "query × undeclared seed object (op-level unknown_seed)",
          args: &["query", ANALYSIS_QUERY, "--seed", "zzz#0", "--follow", "f", "--json"],
          expect: Expect::Error { code: "unknown_seed" } },
    Row { name: "query × undeclared arrow in --follow (catherine-348: warning, exit unchanged)",
          args: &["query", ANALYSIS_QUERY, "--seed", "p#0", "--follow", "zzz", "--json"],
          expect: Expect::Warning { exit: 0, contains: "zzz" } },
    Row { name: "query × bad --direction value",
          args: &["query", ANALYSIS_QUERY, "--seed", "p#0", "--follow", "f",
                  "--direction", "sideways", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "query × bad flag syntax (malformed --seed spelling)",
          args: &["query", ANALYSIS_QUERY, "--seed", "p", "--follow", "f", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "query × out-of-range format_version",
          args: &["query", BAD_VERSION, "--seed", "p#0", "--follow", "f", "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "query × bad flag syntax (unknown flag)",
          args: &["query", ANALYSIS_QUERY, "--seed", "p#0", "--follow", "f", "--bogus", "--json"],
          expect: Expect::Usage },

    // ── migrate ─────────────────────────────────────────────────────
    Row { name: "migrate × partial schema map (unassigned arrow named)",
          args: &["migrate", MIGRATE_PARTIAL_MAP, "--json"],
          expect: Expect::Error { code: "partial_schema_map" } },
    Row { name: "migrate × document without a schema_map (no silent empty migration)",
          args: &["migrate", MIGRATE_NO_MAP, "--json"],
          expect: Expect::Error { code: "no_schema_map" } },
    Row { name: "migrate × garbage instance rows (table cell references an undeclared row)",
          args: &["migrate", MIGRATE_GARBAGE_ROWS, "--json"],
          expect: Expect::Error { code: "syntax" } },
    Row { name: "migrate × out-of-range format_version",
          args: &["migrate", BAD_VERSION, "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "migrate × bad flag syntax (unknown flag)",
          args: &["migrate", MIGRATION_DROP_OBJECT, "--bogus", "--json"],
          expect: Expect::Usage },

    // ── rules ──────────────────────────────────────────────────────
    Row { name: "rules × document without a rule_presentation (no silent empty analysis)",
          args: &["rules", RULES_NO_SECTION, "--json"],
          expect: Expect::Error { code: "no_rule_presentation" } },
    Row { name: "rules × unknown column in a predicate",
          args: &["rules", RULES_UNKNOWN_COLUMN, "--values", "n=1", "--json"],
          expect: Expect::Error { code: "unknown_column" } },
    Row { name: "rules × invalid policy spelling",
          args: &["rules", RULES_BAD_POLICY, "--values", "n=1", "--json"],
          expect: Expect::Error { code: "invalid_policy" } },
    Row { name: "rules × garbage rules section (non-object rule)",
          args: &["rules", RULES_GARBAGE, "--values", "n=1", "--json"],
          expect: Expect::Error { code: "syntax" } },
    Row { name: "rules × declared column with no supplied values",
          args: &["rules", RULES_CLEAN, "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "rules × undeclared --values column",
          args: &["rules", RULE_LENS_GAP, "--values", "nope=1", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "rules × malformed --values token (non-int)",
          args: &["rules", RULE_LENS_GAP, "--values", "age=ten", "--json"],
          expect: Expect::Error { code: "bad_argument" } },
    Row { name: "rules × out-of-range format_version",
          args: &["rules", BAD_VERSION, "--values", "n=1", "--json"],
          expect: Expect::Error { code: "unsupported_version" } },
    Row { name: "rules × bad flag syntax (unknown flag)",
          args: &["rules", RULE_LENS_GAP, "--values", "age=1", "--bogus", "--json"],
          expect: Expect::Usage },
];

/// The recurrence gate: every row must surface its malformed input
/// honestly — labeled error, usage error, or envelope warning — never
/// silence.
#[test]
fn negative_matrix_every_row_surfaces_honestly() {
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
