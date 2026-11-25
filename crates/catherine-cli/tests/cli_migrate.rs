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
//! input errors exit 1 through the labeled-error path. The spawn/parse
//! plumbing is the shared `tests/common` harness
//! (`try_run`/`envelope`/`lost`), spelled for `?`.

mod common;

#[path = "cli_migrate/bogart_rehearsal.rs"]
mod bogart_rehearsal;

use common::{code, envelope, lost, stderr, stdout, try_run};
use serde_json::Value;
use std::error::Error;

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

/// The success-envelope pins shared by every migration report: the
/// genesis keys plus the findings-bearing `check` kind.
fn assert_migration_report_is_a_success_envelope(v: &Value) {
    assert_eq!(v["ok"], true, "the migration report is a success envelope");
    for key in ["ok", "envelope_kind", "data", "warnings", "hints"] {
        assert!(v.get(key).is_some(), "envelope key `{key}` present");
    }
    assert_eq!(v["envelope_kind"], "check", "the payload carries findings");
}

/// The drop-object report's numeric pins: six lost findings, the
/// target-row summary, and the kept-row correspondence verbatim.
fn assert_target_rows_and_correspondence(v: &Value) {
    assert_eq!(lost(v).len(), 6, "3 dropped Scratch rows + 3 valued cells");
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
}

/// The kernel `MigrationFinding` encoding of `data.lost`, verbatim:
/// label, kind tag, witness — sorted (rows first, then the dropped
/// arrow's cells).
fn assert_lost_findings_verbatim(v: &Value) {
    let lost = lost(v);
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

// ── the lost-data run: envelope verbatim + exit 8 ────────────────────

#[test]
fn migrate_lost_data_envelope_is_exit_8_with_kernel_findings_verbatim() -> Result<(), Box<dyn Error>>
{
    let out = try_run(&["migrate", DROP_OBJECT, "--json"])?;
    assert_eq!(code(&out), 8, "lost data is a finding-bearing run");
    assert_eq!(
        stderr(&out),
        "",
        "the error stream stays empty on a finding run"
    );

    let v = envelope("stdout", &stdout(&out))?;
    assert_migration_report_is_a_success_envelope(&v);
    assert_target_rows_and_correspondence(&v);
    assert_lost_findings_verbatim(&v);
    Ok(())
}

#[test]
fn migrate_lossless_run_is_exit_zero_with_empty_lost() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", NON_INVERTIBLE, "--json"])?;
    assert_eq!(code(&out), 0, "a lossless migration is clean");
    assert_eq!(stderr(&out), "", "the error stream stays empty");

    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["ok"], true);
    assert_eq!(lost(&v).len(), 0);
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
    Ok(())
}

// ── labeled errors: exit 1, envelope on stderr, stable code ──────────

#[test]
fn migrate_without_schema_map_is_the_no_schema_map_error() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", NO_MAP, "--json"])?;
    assert_eq!(code(&out), 1, "no migration declared is an input error");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "no_schema_map");
    assert!(
        !v["data"]["remediation"]
            .as_array()
            .unwrap_or_else(|| panic!("remediation"))
            .is_empty(),
        "remediation is non-empty"
    );
    Ok(())
}

#[test]
fn migrate_partial_map_surfaces_the_labeled_parse_error() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", PARTIAL_MAP, "--json"])?;
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["data"]["code"], "partial_schema_map", "the stable code");
    let message = v["data"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("message"));
    assert!(
        message.contains("scratch_note"),
        "the unmapped element is named: {message}"
    );
    Ok(())
}

#[test]
fn migrate_garbage_rows_surfaces_the_labeled_parse_error() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", GARBAGE_ROWS, "--json"])?;
    assert_eq!(code(&out), 1, "parse/build failure exit code");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["data"]["code"], "syntax", "JSON-structural malformation");
    let message = v["data"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("message"));
    assert!(
        message.contains("_9"),
        "the undeclared row reference is named: {message}"
    );
    Ok(())
}

// ── human rendering ──────────────────────────────────────────────────

#[test]
fn migrate_human_lines_name_every_lost_element() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", DROP_OBJECT])?;
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
    Ok(())
}

/// The constraint-level signal (catherine-kpn.8 option (a)): migrate
/// runs the shared diff classifier over the embedded target schema, so
/// a target that forgets an accepted constraint is a finding-bearing
/// run — one `weakened` finding naming the dropped constraint with its
/// separating witness row — while the rows and cells still migrate
/// exactly as before (no lost-row/lost-cell findings here).
#[test]
fn migrate_target_dropping_an_accepted_constraint_emits_the_weakened_finding(
) -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", DROPPED_EQUATION, "--json"])?;
    assert_eq!(
        code(&out),
        8,
        "a weakened migration is a finding-bearing run"
    );
    assert_eq!(stderr(&out), "", "the error stream stays empty");
    let v = envelope("stdout", &stdout(&out))?;
    assert_data_migration_is_lossless(lost(&v));
    let weakened = weakened_findings(&v);
    assert_eq!(
        weakened.len(),
        1,
        "exactly one constraint-level finding: {:?}",
        lost(&v)
    );
    assert_weakened_names_the_dropped_triangle(weakened[0]);
    Ok(())
}

/// The data migration itself is lossless: no lost-row/lost-cell finding
/// among the losses (constraint-level drift only).
fn assert_data_migration_is_lossless(lost: &[Value]) {
    assert!(
        !lost
            .iter()
            .any(|f| f["kind"] == "lost_row" || f["kind"] == "lost_cell"),
        "the data migration itself is lossless: {lost:?}"
    );
}

/// The weakened findings of one migration envelope, in payload order.
fn weakened_findings(value: &Value) -> Vec<&Value> {
    lost(value)
        .iter()
        .filter(|finding| finding["kind"] == "weakened")
        .collect()
}

/// The dropped-equation migration's single weakened finding: the
/// executed drift names the dropped triangle on its domain object.
fn assert_weakened_names_the_dropped_triangle(finding: &Value) {
    assert_eq!(finding["label"], "executed", "drift is executed fact");
    let witness = &finding["witness"];
    assert_eq!(
        witness["object"], "A",
        "the witness row sits on the dropped equation's domain object"
    );
    let constraint = witness["constraint"]
        .as_str()
        .unwrap_or_else(|| panic!("the finding names the dropped constraint"));
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
fn migrate_dropping_two_constraints_gives_each_its_own_witness_row() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", DROPPED_TWO_CONSTRAINTS, "--json"])?;
    assert_eq!(
        code(&out),
        8,
        "a weakened migration is a finding-bearing run"
    );
    let v = envelope("stdout", &stdout(&out))?;
    assert_data_migration_is_lossless(lost(&v));
    let weakened = weakened_findings(&v);
    assert_eq!(
        weakened.len(),
        2,
        "both dropped constraints are named: {:?}",
        lost(&v)
    );
    // `f = g`: the scan's first resolved candidate (every cell at row
    // 0) keeps the equation equal — vacuous there — so its scan
    // continues to the next candidate, which first parts f from g at
    // row 1. A shared first-violation shortcut would report row 0.
    let equation = weakening_by_constraint(&weakened, "f = g");
    // `key A(g)`: the first resolved candidate already maps both rows
    // to row 0 — the key's own colliding pair starts at row 0.
    let key = weakening_by_constraint(&weakened, "key A(g)");
    assert_each_constraint_names_its_own_witness_row(equation, key);
    Ok(())
}

/// The weakened finding whose witness names `needle`.
fn weakening_by_constraint<'a>(weakened: &[&'a Value], needle: &str) -> &'a Value {
    weakened
        .iter()
        .find(|finding| {
            finding["witness"]["constraint"]
                .as_str()
                .is_some_and(|constraint| constraint.contains(needle))
        })
        .unwrap_or_else(|| panic!("the constraint is named"))
}

/// Each dropped constraint gets its own separating witness: `f = g`
/// separates at row 1 (the scan's first resolved candidate — every cell
/// at row 0 — keeps the equation equal, vacuous there, so its scan
/// continues; a shared first-violation shortcut would report row 0),
/// and the key `A(g)` collides at its own opening candidate's row 0.
fn assert_each_constraint_names_its_own_witness_row(equation: &Value, key: &Value) {
    assert_eq!(
        equation["witness"]["object"], "A",
        "the witness row sits on the equation's domain object"
    );
    assert_eq!(
        equation["witness"]["row"], 1,
        "the equation's separating row is its own"
    );
    assert_eq!(key["witness"]["object"], "A");
    assert_eq!(
        key["witness"]["row"], 0,
        "the key's colliding pair starts at row 0"
    );
}

// ── task 8: the bogart rehearsal fixture, end to end ─────────────────
// Pinned in the `bogart_rehearsal` submodule.
