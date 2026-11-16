//! Purpose: the cross-capability integration surface (ro5u finding
//! DRAFT-001) — one committed binary that drives a format-1.5 document
//! carrying `schema.keys`, `schema.coproducts`, and `instance_map` plus
//! equations through the real `catherine` binary, and glues it with a
//! seam document.
//!
//! Responsibilities: probe the combined fixture through `check` (clean),
//! its rogue-variant variant (one coproduct coverage violation), and
//! `naturality` against an identical-schema target (commuting and
//! breaking); probe the joint run with the seam document (one witnessed
//! seam violation whose `source` names both declaring documents) and the
//! seam document alone (the `unknown_arrow` refusal — a cross-document
//! claim is only standalone-parseable when every arrow it names is local).
//!
//! Rationale: each capability has its own binary, so this one ratchets
//! the combined section set as a single executed document and at a seam.
//! Assertions are structural (exit codes, finding kinds, witness and
//! `source` fields) and never snapshots: the additive sections must not
//! perturb one another's execution, and the seam violation must stay
//! attributable to the documents that declared it.

mod common;

use common::{code, findings, parse_json, run, stderr, stdout};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/integration/");

fn fixture(name: &str) -> String {
    format!("{FIXTURES}{name}")
}

/// Scenario "Clean combined document is silent": all three additive
/// sections execute together and produce zero findings, exit 0.
#[test]
fn clean_combined_document_checks_clean() {
    let out = run(&["check", &fixture("all_sections.ct.json"), "--json"]);
    assert_eq!(
        code(&out),
        0,
        "the combined document is clean: {}",
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "a clean run writes no diagnostics");
    let envelope = parse_json(&stdout(&out), "the combined check envelope");
    assert!(
        findings(&envelope).is_empty(),
        "keys, coproducts, and the equation all hold: {:?}",
        findings(&envelope)
    );
}

/// Scenario "Rogue variant is a coverage violation": the same combined
/// schema with one uncovered `Method` row executes the declared coproduct
/// and exits with the violation class.
#[test]
fn rogue_variant_is_a_coproduct_violation() {
    let out = run(&["check", &fixture("rogue_variant.ct.json"), "--json"]);
    assert_eq!(
        code(&out),
        3,
        "a coverage violation exits 3: {}",
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the rogue-variant envelope");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "exactly one finding: {found:?}");
    assert_eq!(found[0]["kind"], "violation", "the finding kind");
    assert_eq!(found[0]["label"], "executed", "an accepted coproduct");
    let witness = &found[0]["witness"];
    assert_eq!(witness["coproduct"], 0, "the only declaration");
    assert_eq!(witness["row"], 2, "`voucher` is the uncovered row");
    assert!(
        witness.get("injections").is_none(),
        "coverage carries no injection pair: {witness}"
    );
}

/// Scenario "Naturality commutes against an identical-schema target":
/// the combined document's map is total and every square holds.
#[test]
fn naturality_against_identical_schema_is_clean() {
    let out = run(&[
        "naturality",
        &fixture("all_sections.ct.json"),
        &fixture("warehouse.ct.json"),
        "--json",
    ]);
    assert_eq!(
        code(&out),
        0,
        "the commuting map is clean: {}",
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "a clean run writes no diagnostics");
    let envelope = parse_json(&stdout(&out), "the naturality envelope");
    assert!(
        findings(&envelope).is_empty(),
        "every square commutes: {:?}",
        findings(&envelope)
    );
}

/// Scenario "A disagreeing identical-schema target is a witnessed break":
/// the target's `region_tier` disagrees on the `north` row only.
#[test]
fn naturality_break_is_a_witnessed_violation() {
    let out = run(&[
        "naturality",
        &fixture("all_sections.ct.json"),
        &fixture("warehouse_break.ct.json"),
        "--json",
    ]);
    assert_eq!(
        code(&out),
        3,
        "one commutation break exits 3: {}",
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the break envelope");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "exactly one square breaks: {found:?}");
    assert_eq!(found[0]["kind"], "violation", "a break is a violation");
    let witness = &found[0]["witness"];
    assert_eq!(witness["arrow"], "region_tier", "the broken arrow");
    assert_eq!(witness["source_row"], 0, "the `north` row");
    assert_eq!(witness["lhs"], 1, "`map ∘ region_tier` lands on `pro`");
    assert_eq!(witness["rhs"], 0, "`region_tier ∘ map` lands on `basic`");
}

/// Scenario "Joint check executes the seam claim": the seam equation names
/// an arrow only the combined document declares, so it is deferred alone
/// and executed after gluing; the violation's `source` names both
/// declaring documents.
#[test]
fn joint_check_attributes_the_seam_violation() {
    let out = run(&[
        "check",
        &fixture("all_sections.ct.json"),
        &fixture("seam.ct.json"),
        "--json",
    ]);
    assert_eq!(
        code(&out),
        3,
        "the seam violation exits 3: {}",
        stderr(&out)
    );
    let envelope = parse_json(&stdout(&out), "the joint check envelope");
    let found = findings(&envelope);
    assert_eq!(found.len(), 1, "only the seam claim fails: {found:?}");
    assert_eq!(found[0]["kind"], "violation", "the seam finding kind");
    let sources: Vec<&str> = found[0]["source"]
        .as_array()
        .expect("joint findings carry source attribution")
        .iter()
        .map(|document| document.as_str().expect("document name"))
        .collect();
    assert!(
        sources[0].ends_with("seam.ct.json"),
        "the declaring document is attributed first: {sources:?}"
    );
    assert!(
        sources.iter().any(|s| s.ends_with("all_sections.ct.json")),
        "the path's declaring document is attributed: {sources:?}"
    );
}

/// Scenario "A seam document alone is refused": the claim names an arrow
/// it does not declare, so a standalone `check` is the parse-time
/// `unknown_arrow` labeled error, exit 1.
#[test]
fn seam_document_alone_is_unknown_arrow() {
    let out = run(&["check", &fixture("seam.ct.json"), "--json"]);
    assert_eq!(code(&out), 1, "a labeled parse error exits 1");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let envelope = parse_json(&stderr(&out), "the seam refusal");
    assert_eq!(
        envelope["data"]["code"], "unknown_arrow",
        "stable error code"
    );
    let message = envelope["data"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("`dispatch_region`"),
        "the undeclared arrow is named: {message}"
    );
}

/// Canonical stability: two joint runs of the same inputs emit identical
/// findings (the taxonomy order is a contract, not walk order).
#[test]
fn repeated_joint_check_is_findings_stable() {
    let args = [
        "check",
        &fixture("all_sections.ct.json"),
        &fixture("seam.ct.json"),
        "--json",
    ];
    let first = run(&args);
    let second = run(&args);
    assert_eq!(code(&first), 3, "the joint run stays finding-bearing");
    assert_eq!(code(&second), 3, "the repeat run keeps the class");
    let first = parse_json(&stdout(&first), "first joint envelope");
    let second = parse_json(&stdout(&second), "second joint envelope");
    assert_eq!(
        findings(&first),
        findings(&second),
        "repeated runs are diff-stable"
    );
}
