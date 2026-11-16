//! Purpose: the `catherine check` acceptance surface for coproducts
//! (openspec change `add-coproduct-exhaustiveness`): a declared
//! `schema.coproducts` section executes coverage and disjointness, and
//! both witness shapes reach the human and `--json` renderings with the
//! violation exit code.
//!
//! Responsibilities: probe the real binary over the hermetic fixtures —
//! the rogue `voucher` row (coverage: one executed violation naming the
//! uncovered row, no injection pair), the dual-covered `shared` row
//! (disjointness: the row and both injections), the clean closed table
//! (zero findings, exit 0), and the exit-3 class on every violation.
//!
//! Rationale: the finding JSON is the kernel's encoding printed verbatim
//! (no CLI reshaping), so assertions are structural on `data.findings`;
//! the human assertions pin only the witness facts the §5 block must
//! carry (row spelling, `uncovered`, both injection names). The web
//! panel's coproduct union branch is out of scope for this binary — it
//! is the wasm boundary's concern and is noted for GREEN, not tested
//! here.

mod common;

use common::{code, findings, parse_json, run, stderr, stdout};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/coproduct/");

fn fixture(name: &str) -> String {
    format!("{FIXTURES}{name}")
}

/// Scenario "Rogue variant is a violation with a witness": one executed
/// violation names the uncovered `Method` row and carries no injection
/// pair; the run exits with the violation class code.
#[test]
fn rogue_variant_json_names_the_uncovered_row() {
    let out = run(&["check", &fixture("rogue-voucher.ct.json"), "--json"]);
    assert_eq!(
        code(&out),
        3,
        "a coverage violation exits 3: {}",
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the check envelope");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "exactly one finding: {found:?}");
    assert_eq!(found[0]["kind"], "violation");
    assert_eq!(found[0]["label"], "executed");
    let witness = &found[0]["witness"];
    assert_eq!(witness["coproduct"], 0, "the declaration's index");
    assert_eq!(witness["row"], 3, "`voucher` is Method row 3");
    assert!(
        witness.get("injections").is_none(),
        "a coverage witness carries no injection pair: {witness}"
    );
}

/// The human rendering names the uncovered row and marks it uncovered.
#[test]
fn rogue_variant_human_names_the_uncovered_row() {
    let out = run(&["check", &fixture("rogue-voucher.ct.json"), "--human"]);
    assert_eq!(
        code(&out),
        3,
        "the human run keeps the class: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("violation:"),
        "the head is a violation: {text}"
    );
    assert!(text.contains("Method#3"), "the uncovered row: {text}");
    assert!(text.contains("uncovered"), "the defect is named: {text}");
}

/// Scenario "Dual coverage is a violation naming both injections": one
/// executed violation names the `shared` row and both covering arrows.
#[test]
fn dual_coverage_json_names_the_row_and_both_injections() {
    let out = run(&["check", &fixture("dual-covered.ct.json"), "--json"]);
    assert_eq!(
        code(&out),
        3,
        "a disjointness violation exits 3: {}",
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the check envelope");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "exactly one finding: {found:?}");
    assert_eq!(found[0]["kind"], "violation");
    assert_eq!(found[0]["label"], "executed");
    let witness = &found[0]["witness"];
    assert_eq!(witness["coproduct"], 0, "the declaration's index");
    assert_eq!(witness["row"], 2, "`shared` is Method row 2");
    assert_eq!(
        witness["injections"],
        serde_json::json!(["card_inj", "transfer_inj"]),
        "both injections covering the row"
    );
}

/// The human rendering names the row and both injections.
#[test]
fn dual_coverage_human_names_the_row_and_both_injections() {
    let out = run(&["check", &fixture("dual-covered.ct.json"), "--human"]);
    assert_eq!(
        code(&out),
        3,
        "the human run keeps the class: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(text.contains("Method#2"), "the doubly covered row: {text}");
    assert!(
        text.contains("card_inj") && text.contains("transfer_inj"),
        "both colliding injections: {text}"
    );
}

/// Scenario "Clean closed table is silent": a correct declaration emits
/// zero findings and exits 0.
#[test]
fn clean_closed_table_is_silent() {
    let out = run(&["check", &fixture("clean.ct.json"), "--json"]);
    assert_eq!(code(&out), 0, "a clean table exits 0: {}", stderr(&out));
    assert_eq!(stderr(&out), "", "a clean run writes no diagnostics");
    let envelope = parse_json(&stdout(&out), "the check envelope");
    assert!(
        findings(&envelope).is_empty(),
        "a closed table with exact coverage emits nothing"
    );
}

/// A proposed coproduct's coverage failure is a question: the same
/// witness shape, labeled `hypothesized`, still exits through the
/// violation class.
#[test]
fn proposed_coproduct_is_hypothesized() {
    let out = run(&["check", &fixture("proposed-voucher.ct.json"), "--json"]);
    assert_eq!(code(&out), 3, "a hypothesized violation keeps the class");
    let envelope = parse_json(&stdout(&out), "the check envelope");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "exactly one finding: {found:?}");
    assert_eq!(found[0]["label"], "hypothesized");
}
