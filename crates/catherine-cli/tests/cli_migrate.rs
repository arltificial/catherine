//! Acceptance tests for the `catherine migrate` subcommand (openspec
//! change `add-delta-migration`, beads `catherine-alr` phase C, task 7).
//!
//! Purpose: pin the user-facing contract end-to-end — the success
//! envelope shape (`data.rows` target-row summary, `data.lost` as the
//! kernel `MigrationFinding` encoding verbatim, `data.row_map` as the
//! kernel correspondence verbatim), the lost-data exit (8) and the
//! lossless exit (0), the labeled errors (`no_schema_map`,
//! `partial_schema_map`, `syntax`), the human rendering, and the
//! bogart rehearsal fixture's documented expectations (task 8).
//!
//! Responsibilities: spawn the real binary (`CARGO_BIN_EXE_catherine`)
//! against the committed migration fixtures (`fixtures/migration/`,
//! shared with the kernel suites) and the migrate-only fixtures under
//! `tests/fixtures/`; assert structurally (`serde_json::Value` field
//! names/values) and numerically (exit codes) — never byte-compare
//! JSON. The lost findings are the kernel's serialization printed
//! verbatim; tests read it back, the CLI never does.
//!
//! Rationale: fixtures are real kernel documents whose schema maps are
//! total (the `partial_schema_map` fixture is the one deliberate
//! exception — it pins the labeled parse error through the CLI). Exit
//! codes follow the CLI's exit table: lost data is a finding-bearing
//! run (8), a lossless migration is clean (0), parse/build and
//! input errors exit 1 through the labeled-error path.

use std::process::{Command, Output};

const DROP_OBJECT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/migration/drop_object.ct.json"
);
const NON_INVERTIBLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/migration/non_invertible_round_trip.ct.json"
);
const BOGART: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/migration/bogart_rehearsal.ct.json"
);
const PARTIAL_MAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/migrate_partial_map.ct.json"
);
const GARBAGE_ROWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/migrate_garbage_rows.ct.json"
);
/// The dropped-equation fixture (shared with the expressibility-gap
/// suite): the target schema renames the vocabulary and forgets the
/// accepted triangle equation, while the data migrates losslessly.
const DROPPED_EQUATION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gap_suite/dropped_equation_migration.ct.json"
);
/// Two accepted constraints — an equation over `P` and a key over `Q` —
/// dropped by one identity-vocabulary target: the both-dropped case the
/// witness-attribution regression pins.
const DROPPED_TWO_CONSTRAINTS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gap_suite/dropped_two_constraints_migration.ct.json"
);
/// A pre-map document: parses, carries no `schema_map`.
const NO_MAP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bowling.ct.json"
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
        .unwrap_or_else(|e| panic!("stdout/stderr is one JSON envelope: {e}\n{text}"))
}

// ── the lost-data run: envelope verbatim + exit 8 ────────────────────

#[test]
fn migrate_lost_data_envelope_is_exit_8_with_kernel_findings_verbatim() {
    let out = run(&["migrate", DROP_OBJECT, "--json"]);
    assert_eq!(code(&out), 8, "lost data is a finding-bearing run");
    assert_eq!(
        stderr(&out),
        "",
        "the error stream stays empty on a finding run"
    );

    let v = parse_json(&stdout(&out));
    assert_eq!(v["ok"], true, "the migration report is a success envelope");
    for key in ["ok", "envelope_kind", "data", "warnings", "hints"] {
        assert!(v.get(key).is_some(), "envelope key `{key}` present");
    }
    assert_eq!(v["envelope_kind"], "check", "the payload carries findings");

    assert_eq!(
        v["data"]["rows"],
        serde_json::json!({"Doc": 2}),
        "target rows per object (the migrated instance summary)"
    );
    assert_eq!(
        v["data"]["row_map"],
        serde_json::json!({"Doc": {"0": 0, "1": 1}}),
        "kept source rows → target rows, the kernel correspondence verbatim"
    );

    // The kernel `MigrationFinding` encoding, verbatim: label, kind tag,
    // witness — sorted (rows first, then the dropped arrow's cells).
    let lost = v["data"]["lost"].as_array().expect("data.lost array");
    assert_eq!(lost.len(), 6, "3 dropped Scratch rows + 3 valued cells");
    assert_eq!(
        lost[0],
        serde_json::json!({
            "label": "executed",
            "kind": "lost_row",
            "witness": {"object": "Scratch", "row": 0}
        }),
        "the kernel's lost-row encoding verbatim"
    );
    assert_eq!(
        lost[5],
        serde_json::json!({
            "label": "executed",
            "kind": "lost_cell",
            "witness": {"arrow": "scratch_note", "row": 2}
        }),
        "the kernel's lost-cell encoding verbatim"
    );
    for finding in lost {
        assert_eq!(finding["label"], "executed", "losses are executed facts");
    }
}

#[test]
fn migrate_lossless_run_is_exit_zero_with_empty_lost() {
    let out = run(&["migrate", NON_INVERTIBLE, "--json"]);
    assert_eq!(code(&out), 0, "a lossless migration is clean");
    assert_eq!(stderr(&out), "", "the error stream stays empty");

    let v = parse_json(&stdout(&out));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["lost"].as_array().expect("lost").len(), 0);
    assert_eq!(
        v["data"]["rows"],
        serde_json::json!({"Narrow": 1}),
        "both Wide rows collapse onto the one Narrow row"
    );
    assert_eq!(
        v["data"]["row_map"]["Narrow"],
        serde_json::json!({"0": 0, "1": 0}),
        "the collapse is visible in the correspondence, not silent"
    );
}

// ── labeled errors: exit 1, envelope on stderr, stable code ──────────

#[test]
fn migrate_without_schema_map_is_the_no_schema_map_error() {
    let out = run(&["migrate", NO_MAP, "--json"]);
    assert_eq!(code(&out), 1, "no migration declared is an input error");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = parse_json(&stderr(&out));
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "no_schema_map");
    assert!(
        !v["data"]["remediation"]
            .as_array()
            .expect("remediation")
            .is_empty(),
        "remediation is non-empty"
    );
}

#[test]
fn migrate_partial_map_surfaces_the_labeled_parse_error() {
    let out = run(&["migrate", PARTIAL_MAP, "--json"]);
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = parse_json(&stderr(&out));
    assert_eq!(v["data"]["code"], "partial_schema_map", "the stable code");
    let message = v["data"]["message"].as_str().expect("message");
    assert!(
        message.contains("scratch_note"),
        "the unmapped element is named: {message}"
    );
}

#[test]
fn migrate_garbage_rows_surfaces_the_labeled_parse_error() {
    let out = run(&["migrate", GARBAGE_ROWS, "--json"]);
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = parse_json(&stderr(&out));
    assert_eq!(v["data"]["code"], "syntax", "JSON-structural malformation");
    let message = v["data"]["message"].as_str().expect("message");
    assert!(
        message.contains("_9"),
        "the undeclared row reference is named: {message}"
    );
}

// ── human rendering ──────────────────────────────────────────────────

#[test]
fn migrate_human_lines_name_every_lost_element() {
    let out = run(&["migrate", DROP_OBJECT]);
    assert_eq!(code(&out), 8, "the exit decision is mode-independent");
    assert_eq!(stderr(&out), "");
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 7, "one summary head + one line per finding");
    assert!(
        lines[0].starts_with("migrate: 2 target row(s) across 1 object(s), 6 lost data finding(s)"),
        "the summary head counts honestly: {first}",
        first = lines[0]
    );
    assert_eq!(
        lines[1], "lost-row: Scratch#0",
        "witness lines are countable"
    );
    assert_eq!(lines[4], "lost-cell: scratch_note at Scratch#0");
}

/// The constraint-level signal (catherine-kpn.8 option (a)): migrate
/// runs the shared diff classifier over the embedded target schema, so
/// a target that forgets an accepted constraint is a finding-bearing
/// run — one `weakened` finding naming the dropped constraint with its
/// separating witness row — while the rows and cells still migrate
/// exactly as before (no lost-row/lost-cell findings here).
#[test]
fn migrate_target_dropping_an_accepted_constraint_emits_the_weakened_finding() {
    let out = run(&["migrate", DROPPED_EQUATION, "--json"]);
    assert_eq!(
        code(&out),
        8,
        "a weakened migration is a finding-bearing run"
    );
    assert_eq!(stderr(&out), "", "the error stream stays empty");
    let v = parse_json(&stdout(&out));
    let lost = v["data"]["lost"].as_array().expect("data.lost array");
    assert!(
        !lost
            .iter()
            .any(|f| f["kind"] == "lost_row" || f["kind"] == "lost_cell"),
        "the data migration itself is lossless: {lost:?}"
    );
    let weakened: Vec<_> = lost.iter().filter(|f| f["kind"] == "weakened").collect();
    assert_eq!(
        weakened.len(),
        1,
        "exactly one constraint-level finding: {lost:?}"
    );
    assert_eq!(weakened[0]["label"], "executed", "drift is executed fact");
    let witness = &weakened[0]["witness"];
    assert_eq!(
        witness["object"], "A",
        "the witness row sits on the dropped equation's domain object"
    );
    let constraint = witness["constraint"]
        .as_str()
        .expect("the finding names the dropped constraint");
    assert!(
        constraint.contains("f") && constraint.contains("g") && constraint.contains("k"),
        "the finding names the dropped triangle `{constraint}`"
    );
}

/// Two accepted constraints — the equation `f = g` and the key `g` —
/// dropped by one identity-vocabulary target: the suite's only
/// both-dropped migration. The classifier scans each dropped
/// constraint in isolation, so every `weakened` finding carries its
/// own separating witness: the equation separates at row 1, the key
/// collides at the opening candidate's row 0. The zero-row instance
/// keeps both scans (four cells) inside the 256-candidate budget.
#[test]
fn migrate_dropping_two_constraints_gives_each_its_own_witness_row() {
    let out = run(&["migrate", DROPPED_TWO_CONSTRAINTS, "--json"]);
    assert_eq!(
        code(&out),
        8,
        "a weakened migration is a finding-bearing run"
    );
    let v = parse_json(&stdout(&out));
    let lost = v["data"]["lost"].as_array().expect("data.lost array");
    assert!(
        !lost
            .iter()
            .any(|f| f["kind"] == "lost_row" || f["kind"] == "lost_cell"),
        "the data migration itself is lossless: {lost:?}"
    );
    let weakened: Vec<_> = lost.iter().filter(|f| f["kind"] == "weakened").collect();
    assert_eq!(
        weakened.len(),
        2,
        "both dropped constraints are named: {lost:?}"
    );
    let by_constraint = |needle: &str| {
        weakened
            .iter()
            .find(|f| {
                f["witness"]["constraint"]
                    .as_str()
                    .is_some_and(|c| c.contains(needle))
            })
            .expect("the constraint is named")
    };
    // `f = g`: the scan's first resolved candidate (every cell at row
    // 0) keeps the equation equal — vacuous there — so its scan
    // continues to the next candidate, which first parts f from g at
    // row 1. A shared first-violation shortcut would report row 0.
    let equation = by_constraint("f = g");
    assert_eq!(
        equation["witness"]["object"], "A",
        "the witness row sits on the equation's domain object"
    );
    assert_eq!(
        equation["witness"]["row"], 1,
        "the equation's separating row is its own"
    );
    // `key A(g)`: the first resolved candidate already maps both rows
    // to row 0 — the key's own colliding pair starts at row 0.
    let key = by_constraint("key A(g)");
    assert_eq!(key["witness"]["object"], "A");
    assert_eq!(
        key["witness"]["row"], 0,
        "the key's colliding pair starts at row 0"
    );
}

// ── task 8: the bogart rehearsal fixture, end to end ─────────────────

#[test]
fn bogart_rehearsal_emits_the_documented_ten_findings_and_exits_8() {
    let out = run(&["migrate", BOGART, "--json"]);
    assert_eq!(code(&out), 8, "the rehearsal drops data by design");

    let v = parse_json(&stdout(&out));
    assert_eq!(
        v["data"]["rows"],
        serde_json::json!({"ServingDocument": 2, "ServingPage": 3}),
        "raw corpus → serving projection"
    );

    let lost = v["data"]["lost"].as_array().expect("data.lost array");
    assert_eq!(lost.len(), 10, "4 dropped rows + 6 valued dropped cells");

    let lost_rows: Vec<_> = lost.iter().filter(|f| f["kind"] == "lost_row").collect();
    assert_eq!(lost_rows.len(), 4, "every RawCapture row is named");
    for (i, finding) in lost_rows.iter().enumerate() {
        assert_eq!(finding["witness"]["object"], "RawCapture");
        assert_eq!(finding["witness"]["row"], i, "rows named in dense order");
    }

    // Sorted order: captured_page (RawCapture rows 0–3), then prev_page
    // (RawPage rows 1–2 — `_0` held no value, so it is not a lost cell).
    assert_eq!(lost[4]["kind"], "lost_cell");
    assert_eq!(lost[4]["witness"]["arrow"], "captured_page");
    assert_eq!(lost[4]["witness"]["row"], 0);
    assert_eq!(lost[7]["witness"]["arrow"], "captured_page");
    assert_eq!(lost[7]["witness"]["row"], 3);
    assert_eq!(lost[8]["witness"]["arrow"], "prev_page");
    assert_eq!(lost[8]["witness"]["row"], 1);
    assert_eq!(lost[9]["witness"]["arrow"], "prev_page");
    assert_eq!(lost[9]["witness"]["row"], 2);

    assert_eq!(
        v["data"]["row_map"]["ServingPage"],
        serde_json::json!({"0": 0, "1": 1, "2": 2}),
        "kept RawPage rows transfer verbatim"
    );
    assert!(
        v["data"]["row_map"].get("RawCapture").is_none(),
        "dropped objects are absent from the correspondence — their loss is in `lost`"
    );
}
