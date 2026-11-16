//! Purpose: the `catherine naturality` acceptance surface (openspec change
//! `add-instance-morphism`): a source document's declared `instance_map`
//! is executed against a target instance of the identical schema, and a
//! commutation break is one `violation`/`executed` finding with the
//! arrow, the source row, and both composed values in its witness.
//!
//! Responsibilities: probe the real binary over the committed
//! crm/warehouse fixtures — the commuting map, the same document twice,
//! the `a3→w3` break and its human witness, a square undefined on both
//! sides (exit 0, `unresolved_naturality_square` warning naming arrow and
//! row), a one-sided undefined square (violation with a null composed
//! value), the reused `schema_mismatch` refusal, the construction-time
//! `unknown_map_target` refusal, and the additive-only envelope shape.
//!
//! Rationale: design Decisions 2/3/4 — breaks are violations with a
//! witness (no new finding kinds), malformed claims are labeled errors
//! before any check runs, and identical schemas are required. Assertions
//! are structural (`serde_json::Value`) and numeric (exit codes), never
//! snapshots; row spellings follow the transcript convention (`Object#n`,
//! dense row ids), the only spelling a schema-only renderer can produce.

mod common;

use common::{code, findings, parse_json, run, stderr, stdout};
use serde_json::Value;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/naturality/");

/// The CRM instance with its declared, commuting map to the warehouse.
const CRM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm.ct.json"
);
/// The warehouse instance every square of the map commutes with.
const WAREHOUSE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/warehouse.ct.json"
);
/// The same warehouse whose `w3` row sits under the wrong owner: exactly
/// the `a3→w3` square breaks.
const WAREHOUSE_BREAK: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/warehouse_break.ct.json"
);
/// A document whose declared map is the identity: the same document twice
/// is clean by construction.
const IDENTITY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/identity_map.ct.json"
);
/// A parseable document with a different schema (the arrow is reversed).
const SCHEMA_MISMATCH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/schema_mismatch.ct.json"
);
/// A total map naming a target row (`w9`) the target instance lacks.
const UNKNOWN_TARGET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_unknown_target.ct.json"
);
/// A source instance whose `account_owner` cell at `a2` dangles while the
/// map stays total.
const CRM_UNDEFINED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/crm_undefined.ct.json"
);
/// The target whose matching `account_owner` cell at `w2` also dangles:
/// exactly the `a2→w2` square is undefined on both sides.
const WAREHOUSE_UNDEFINED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/naturality/warehouse_undefined.ct.json"
);

/// Scenario "Commuting map checks clean": zero findings, exit 0, the
/// envelope's findings array empty.
#[test]
fn naturality_commuting_map_checks_clean() {
    let out = run(&["naturality", CRM, WAREHOUSE, "--json"]);
    assert_eq!(
        code(&out),
        0,
        "a commuting map is clean: stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "a clean run writes no diagnostics");
    let envelope = parse_json(&stdout(&out), "the naturality envelope");
    assert_eq!(envelope["ok"], true, "a clean run is a success envelope");
    assert!(
        findings(&envelope).is_empty(),
        "a commuting map produces zero findings: {:?}",
        findings(&envelope)
    );
}

/// Scenario "Same document twice is trivially clean": the degenerate case
/// is accepted, zero findings, exit 0 — never an error.
#[test]
fn naturality_same_document_twice_is_clean() {
    let out = run(&["naturality", IDENTITY, IDENTITY, "--json"]);
    assert_eq!(
        code(&out),
        0,
        "the same document twice is accepted: stderr={}",
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the naturality envelope");
    assert!(
        findings(&envelope).is_empty(),
        "every square holds by construction: {:?}",
        findings(&envelope)
    );
}

/// Scenario "Naturality break is a violation with a witness": the
/// `a3→w3` square breaks on `account_owner`; one `violation` labeled
/// `executed` carries the arrow, the source row, and both composed values
/// (`lhs` = `map(a(x))` = `owner#1`, `rhs` = `a(map(x))` = `owner#0`), and
/// the run exits with the violation class code (3).
#[test]
fn naturality_break_is_a_violation_with_a_witness() {
    let out = run(&["naturality", CRM, WAREHOUSE_BREAK, "--json"]);
    assert_eq!(
        code(&out),
        3,
        "one violation exits with the violation class code: stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the naturality envelope");
    let findings = findings(&envelope);
    assert_eq!(findings.len(), 1, "exactly one square breaks: {findings:?}");
    let finding = &findings[0];
    assert_eq!(finding["kind"], "violation", "breaks are violations");
    assert_eq!(finding["label"], "executed", "the check was executed");
    let witness = &finding["witness"];
    assert_eq!(witness["arrow"], "account_owner", "the broken arrow");
    assert_eq!(
        witness["source_row"], 2,
        "the source row is `a3`, dense id 2 of `account`"
    );
    assert_eq!(
        witness["lhs"], 1,
        "`map ∘ account_owner` lands on `p2`, dense id 1 of `owner`"
    );
    assert_eq!(
        witness["rhs"], 0,
        "`account_owner ∘ map` lands on `p1`, dense id 0 of `owner`"
    );
}

/// The human rendering names the arrow, the source row, and both composed
/// values in the transcript's `Object#n` spelling (the §5 violation
/// block's `at row …: lhs=… rhs=…` line).
#[test]
fn naturality_break_renders_arrow_row_and_composed_values() {
    let out = run(&["naturality", CRM, WAREHOUSE_BREAK]);
    assert_eq!(
        code(&out),
        3,
        "the human run keeps the exit class: stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("account_owner"),
        "the witness names the arrow: {text}"
    );
    assert!(
        text.contains("account#2"),
        "the witness names the source row: {text}"
    );
    assert!(
        text.contains("owner#1") && text.contains("owner#0"),
        "the witness names both composed values: {text}"
    );
}

/// Scenario "Undefined square is a non-answer": when both `map ∘ arrow`
/// and `arrow ∘ map` are undefined for a square, the run exits 0 with no
/// findings but a `unresolved_naturality_square` warning naming the arrow
/// and source row — never a silent, warning-free clean.
#[test]
fn naturality_undefined_square_warns_instead_of_silent_clean() {
    let out = run(&["naturality", CRM_UNDEFINED, WAREHOUSE_UNDEFINED, "--json"]);
    assert_eq!(
        code(&out),
        0,
        "an undefined square is a warning, never a violation: stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "the warning rides the envelope");
    let envelope = parse_json(&stdout(&out), "the naturality envelope");
    assert_eq!(envelope["ok"], true, "a warning run is a success envelope");
    assert!(
        findings(&envelope).is_empty(),
        "an undefined square is not a witnessed break: {:?}",
        findings(&envelope)
    );
    let warnings = envelope["warnings"].as_array();
    assert_eq!(warnings.map(Vec::len), Some(1), "one undefined square");
    let warning = &warnings.expect("warnings array")[0];
    assert_eq!(
        warning["rule_name"], "unresolved_naturality_square",
        "the stable warning rule name"
    );
    let message = warning["message"]
        .as_str()
        .expect("a warning carries a message");
    assert!(
        message.contains("account_owner")
            && message.contains("source row 1")
            && message.contains("undefined on both sides")
            && message.contains("cannot be verified"),
        "the warning names the arrow, source row, and non-answer: {message}"
    );
}

/// The human surface: the same undefined square prints one `warning:`
/// line on stderr (stdout stays empty, since there is no finding), and
/// the exit stays 0.
#[test]
fn naturality_undefined_square_warns_on_stderr() {
    let out = run(&["naturality", CRM_UNDEFINED, WAREHOUSE_UNDEFINED]);
    assert_eq!(
        code(&out),
        0,
        "the human run keeps the warning class: stderr={}",
        stderr(&out)
    );
    assert_eq!(stdout(&out), "", "there is no finding to print");
    let text = stderr(&out);
    assert!(
        text.contains("warning:") && text.contains("account_owner"),
        "the warning names the arrow on stderr: {text}"
    );
    assert!(
        text.contains("source row 1") && text.contains("undefined on both sides"),
        "the warning names the source row and the non-answer: {text}"
    );
}

/// A one-sided undefined square is still witnessed: the source cell
/// resolves while the target cell dangles, so `lhs` is the composed
/// target row and `rhs` is null — a `violation`/`executed` finding,
/// exit 3, no warning.
#[test]
fn naturality_one_sided_undefined_is_a_violation_with_a_null_side() {
    let out = run(&["naturality", CRM, WAREHOUSE_UNDEFINED, "--json"]);
    assert_eq!(
        code(&out),
        3,
        "a one-sided undefined square breaks the equation: stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the naturality envelope");
    assert_eq!(
        envelope["warnings"].as_array().map(Vec::len),
        Some(0),
        "a witnessed break is not a warning"
    );
    let findings = findings(&envelope);
    assert_eq!(findings.len(), 1, "exactly one square breaks: {findings:?}");
    let witness = &findings[0]["witness"];
    assert_eq!(witness["arrow"], "account_owner", "the broken arrow");
    assert_eq!(
        witness["source_row"], 1,
        "the source row is `a2`, dense id 1 of `account`"
    );
    assert_eq!(
        witness["lhs"], 1,
        "`map ∘ account_owner` lands on `p2`, dense id 1 of `owner`"
    );
    assert!(
        witness["rhs"].is_null(),
        "`account_owner ∘ map` traverses the target's dangling `w2` cell"
    );
}

/// Scenario "Schemas must be identical": the existing `schema_mismatch`
/// labeled error is reused, exit 1, before any finding is emitted.
#[test]
fn naturality_schemas_must_be_identical() {
    let out = run(&["naturality", CRM, SCHEMA_MISMATCH, "--json"]);
    assert_eq!(
        code(&out),
        1,
        "a differing schema is a labeled error: stderr={}",
        stderr(&out)
    );
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let envelope = parse_json(&stderr(&out), "the error envelope");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["envelope_kind"], "error");
    assert_eq!(
        envelope["data"]["code"], "schema_mismatch",
        "the distinguish schema check is reused, never a guessed alignment"
    );
}

/// Scenario "Unknown target is a construction-time error": the map cell
/// names a target row the target instance lacks; the operation fails with
/// `unknown_map_target` naming the object, the source row, and the cell —
/// before any findings are produced.
#[test]
fn naturality_unknown_target_is_a_construction_time_error() {
    let out = run(&["naturality", UNKNOWN_TARGET, WAREHOUSE, "--json"]);
    assert_eq!(
        code(&out),
        1,
        "an unresolvable cell is a labeled error: stderr={}",
        stderr(&out)
    );
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let envelope = parse_json(&stderr(&out), "the error envelope");
    assert_eq!(envelope["ok"], false);
    assert_eq!(
        envelope["data"]["code"], "unknown_map_target",
        "the stable code for an unresolved target row"
    );
    let message = envelope["data"]["message"]
        .as_str()
        .expect("a labeled error carries a message");
    assert!(
        message.contains("account") && message.contains("a2") && message.contains("w9"),
        "the rejection names the object, the mapped source row, and the missing target row: {message}"
    );
}

/// The envelope shape stays additive: every existing top-level key
/// remains, and the payload is the unchanged findings array. New keys may
/// appear; none of these may move or change type.
#[test]
fn naturality_envelope_shape_is_additive_only() {
    let out = run(&["naturality", CRM, WAREHOUSE_BREAK, "--json"]);
    assert_eq!(
        code(&out),
        3,
        "the violation run exits 3: stderr={}",
        stderr(&out)
    );
    let envelope: Value = parse_json(&stdout(&out), "the naturality envelope");
    for key in [
        "ok",
        "envelope_version",
        "cli_version",
        "envelope_kind",
        "data",
        "warnings",
        "hints",
        "meta",
    ] {
        assert!(
            envelope.get(key).is_some(),
            "the genesis envelope key `{key}` is unchanged: {envelope}"
        );
    }
    assert!(
        envelope["envelope_kind"]
            .as_str()
            .is_some_and(|kind| !kind.is_empty()),
        "the envelope kind stays a non-empty string"
    );
    assert!(
        envelope["data"]["findings"].is_array(),
        "the findings payload is the unchanged array"
    );
    assert!(
        envelope["warnings"].is_array() && envelope["hints"].is_array(),
        "warnings and hints stay arrays"
    );
}

/// The fixture paths exist where the consts say they do (a missing
/// fixture must fail loudly, not as a `read_error` under test).
#[test]
fn naturality_fixtures_are_present() {
    for name in [
        "crm.ct.json",
        "warehouse.ct.json",
        "warehouse_break.ct.json",
        "identity_map.ct.json",
        "schema_mismatch.ct.json",
        "crm_unknown_target.ct.json",
        "crm_undefined.ct.json",
        "warehouse_undefined.ct.json",
        "crm_incomplete_map.ct.json",
        "crm_no_map.ct.json",
    ] {
        let path = format!("{FIXTURES}{name}");
        assert!(
            std::path::Path::new(&path).is_file(),
            "fixture `{path}` exists"
        );
    }
}
