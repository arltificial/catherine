//! Purpose: the Meter for the Chain of Responsibility lens (beads
//! `catherine-mx1.13`) — runs the shipped `Lens` directly (not through the
//! registry, which the orchestrator wires afterwards) over every recorded
//! `chain-of-responsibility` row and compares the exact `(kind, label)`
//! multiset the kata ledger recorded.
//!
//! Responsibilities: include the checked-in expectation table, bind and
//! execute each row's fixture, and assert the metadata, the completed
//! outcome, and the bounded non-answer on an absent or unbound surface.
//! Rows whose recorded multiset is empty — the clean tables, the
//! near-floor control, and the behaviour-only strategy-selection faults —
//! must stay silent; parity rows keep their recorded findings without a
//! uniqueness claim.
//!
//! Rationale: the production module is deliberately not declared in
//! `src/pattern/mod.rs` yet (the lead owns that one line), so the test
//! includes it by path and mirrors the crate's module names locally. That
//! keeps the lens's own imports unchanged between the test build and the
//! eventual registry wiring. The comparison sorts both sides, so a row is
//! matched as a multiset; the harness's separate replay Meter re-checks
//! the same rows once the lens is registered.

#[path = "pattern_expectations.rs"]
mod expectations;

/// The crate modules the by-path lens imports; this is the test-local
/// mirror of `catherine_analysis`'s own module tree, not a second copy of
/// any implementation.
mod bounds {}
mod pattern {
    pub use catherine_analysis::pattern::*;
}
mod rule_lens {
    pub use catherine_analysis::rule_lens::*;
}

#[path = "../src/pattern/chain_of_responsibility.rs"]
mod chain_of_responsibility;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LensCall, LensFindings, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::{EpistemicLabel, Finding, FindingKind};
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::Model;
use catherine_core::parse::parse;
use chain_of_responsibility::{Lens, COR_META};
use expectations::{Expectation, FindingKey, EXPECTATIONS};

/// The registry key the expectation table and replay Meter use.
const ID: &str = "chain-of-responsibility";

#[test]
fn metadata_names_the_chain_of_responsibility_row() {
    let meta = Lens.meta();
    assert_eq!(meta.id, ID);
    assert_eq!(meta.name, "Chain of Responsibility");
    assert_eq!(meta.law.as_str(), "order");
    assert!(meta.abstraction.contains("handler-order"));
    assert!(meta.scope.contains("rule-lens regions"));
    assert!(!meta.scope.is_empty());
    assert_eq!(&COR_META, meta);
}

#[test]
fn every_recorded_row_matches_the_lens() {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == ID)
        .collect();
    assert_eq!(rows.len(), 13, "the table records 13 CoR rows");
    for row in rows {
        let findings = run_row(row);
        assert_eq!(
            multiset(&findings),
            row.findings,
            "{} (fault `{}`, parity {}): expected {:?}",
            row.fixture,
            row.fault,
            row.parity,
            row.findings
        );
        assert!(
            findings.migrations.is_empty(),
            "{}: the CoR lens has no migration surface",
            row.fixture
        );
    }
}

#[test]
fn empty_rows_stay_silent() {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == ID && row.findings.is_empty())
        .collect();
    assert!(!rows.is_empty(), "the table records silent rows");
    for row in rows {
        let findings = run_row(row);
        assert!(
            findings.checks.is_empty() && findings.migrations.is_empty(),
            "{} (fault `{}`) must stay silent, got {:?}",
            row.fixture,
            row.fault,
            multiset(&findings)
        );
    }
}

#[test]
fn a_model_without_a_rule_presentation_is_unknown_not_clean() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("rule_presentation")),
        LensOutcome::Findings(findings) => {
            panic!("an absent surface must not be clean: {findings:?}")
        }
    }
}

#[test]
fn unbound_values_are_an_unknown_scope() {
    let text = std::fs::read_to_string(fixture_path(
        "fixtures/katas/cor-strategy-rules/dispatch.ct.json",
    ))
    .expect("fixture is readable");
    let model = parse(&text).expect("fixture parses");
    let values = vec!["kind=fraud,refund".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => {
            assert!(
                scope.contains("amount"),
                "scope names the missing column: {scope}"
            )
        }
        LensOutcome::Findings(_) => panic!("an unbound domain must not be evaluated"),
    }
}

/// Execute one recorded row against the shipped lens.
fn run_row(row: &Expectation) -> LensFindings {
    let text = std::fs::read_to_string(fixture_path(row.fixture))
        .unwrap_or_else(|error| panic!("{} is readable: {error}", row.fixture));
    let model = parse(&text).unwrap_or_else(|error| panic!("{} parses: {error:?}", row.fixture));
    let values: Vec<String> = row.values.iter().map(|spec| spec.to_string()).collect();
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => {
            panic!(
                "{} (fault `{}`) reported unknown: {scope}",
                row.fixture, row.fault
            )
        }
    }
}

/// A repository-relative fixture path.
fn fixture_path(relative: &str) -> String {
    format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"))
}

/// The recorded multiset over both channels, sorted so order does not
/// matter.
fn multiset(findings: &LensFindings) -> Vec<FindingKey> {
    let mut keys: Vec<FindingKey> = findings
        .checks
        .iter()
        .map(check_key)
        .chain(findings.migrations.iter().map(migration_key))
        .collect();
    keys.sort_unstable();
    keys
}

/// One check finding as the taxonomy's stable `(kind, label)` spelling.
fn check_key(finding: &Finding) -> FindingKey {
    let kind = match &finding.kind {
        FindingKind::Violation(_) => "violation",
        FindingKind::Dangling(_) => "dangling",
        FindingKind::Contradiction(_) => "contradiction",
        FindingKind::Underconstraint(_) => "underconstraint",
        FindingKind::Redundant(_) => "redundant",
        FindingKind::Granularity(_) => "granularity",
    };
    (kind, label_name(finding.label))
}

/// One migration finding as the migrate surface's stable spelling.
fn migration_key(finding: &MigrationFinding) -> FindingKey {
    let kind = match &finding.kind {
        MigrationFindingKind::LostRow { .. } => "lost_row",
        MigrationFindingKind::LostCell { .. } => "lost_cell",
        MigrationFindingKind::Weakened { .. } => "weakened",
    };
    (kind, label_name(finding.label))
}

/// The epistemic label's stable spelling.
fn label_name(label: EpistemicLabel) -> &'static str {
    match label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    }
}
