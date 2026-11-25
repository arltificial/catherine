//! Purpose: the `decorator` lens Meter (beads `catherine-mx1.7`) — the
//! wrapper-laws evidence read through the stacking transparency equation,
//! against the checked-in expectation rows.
//!
//! Responsibilities: call `decorator::Lens::run` directly (never the
//! registry, which the orchestrator wires) on every `decorator` row of
//! `pattern_expectations.rs` and assert the recorded `(kind, label)`
//! multiset per fixture, plus the metadata and the seam's fail-closed
//! `Unknown` arm for a `--values` list the lens cannot bind.
//!
//! Rationale: the module under test is included from `src/` by `#[path]`
//! because `pattern/mod.rs` is the orchestrator's registry file, not a
//! lens ticket's; the re-export shims below give the module the same
//! `crate::bounds` / `crate::pattern` paths it sees inside the library, so
//! the tested source is the shipped source unchanged. The comparison is a
//! multiset, never a sequence, matching the replay harness; the rows are
//! the sealed ledger's, so a mismatch is a lens bug, never a table edit.

#[path = "../src/pattern/decorator.rs"]
mod decorator;

mod bounds {
    pub use catherine_analysis::bounds::ScopeCaps;
}

mod pattern {
    pub use catherine_analysis::pattern::{
        LawKind, LensFindings, LensInput, LensOutcome, PatternLens, PatternMeta,
    };
}

#[path = "pattern_expectations.rs"]
mod expectations;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LensFindings, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::{EpistemicLabel, FindingKind};
use catherine_core::model::Model;
use catherine_core::parse::parse;
use expectations::{Expectation, FindingKey, EXPECTATIONS};

/// The number of `decorator` rows the sealed table records (the floor's
/// three faults, the clean arm, and the four above-floor faults).
const ROW_COUNT: usize = 8;

#[test]
fn the_decorator_metadata_is_the_disposition_row() {
    let meta = decorator::Lens.meta();
    assert_eq!(meta.id, "decorator");
    assert_eq!(meta.name, "Decorator");
    assert_eq!(meta.abstraction, "wrapper law preservation");
    assert_eq!(meta.law, catherine_analysis::pattern::LawKind::Order);
    assert_eq!(meta.law.as_str(), "order");
    assert!(meta.scope.contains("stacking transparency"));
    assert!(meta.scope.contains("coverage-only"));
}

#[test]
fn every_recorded_decorator_row_replays_exactly() {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "decorator")
        .collect();
    assert_eq!(
        rows.len(),
        ROW_COUNT,
        "the checked-in table covers `decorator`"
    );
    let caps = ScopeCaps::default();
    for row in rows {
        assert_eq!(row.class, "finding", "`{}` is a finding row", row.fault);
        let actual = actual_multiset(row.fixture, &caps);
        let mut expected = row.findings.to_vec();
        expected.sort_unstable();
        assert_eq!(
            actual, expected,
            "`decorator` on {} (fault `{}`, parity {})",
            row.fixture, row.fault, row.parity
        );
    }
}

#[test]
fn the_clean_arm_stays_silent() {
    let clean: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "decorator" && row.findings.is_empty())
        .collect();
    assert_eq!(clean.len(), 1, "exactly one zero-finding decorator row");
    let caps = ScopeCaps::default();
    for row in clean {
        assert!(
            actual_multiset(row.fixture, &caps).is_empty(),
            "the clean wrapper fixture is silent: {}",
            row.fixture
        );
    }
}

#[test]
fn a_rule_table_without_equations_is_genuine_silence() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match decorator::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Findings(findings) => {
            assert!(findings.checks.is_empty());
            assert!(findings.migrations.is_empty());
        }
        LensOutcome::Unknown { scope } => {
            panic!("an empty document is evaluated, not unknown: {scope}")
        }
    }
}

#[test]
fn a_values_list_is_unknown_because_the_lens_has_no_rule_surface() {
    let model = fixture("fixtures/katas/wrapper-laws/wrapper-laws.ct.json");
    let values = vec!["kind=lit".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match decorator::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => {
            assert!(scope.contains("no rule-table surface"), "{scope}");
        }
        LensOutcome::Findings(_) => panic!("an unevaluable flag must not be ignored"),
    }
}

/// Run the lens on one fixture and return its checks-channel multiset (the
/// decorator lens has no migration surface).
fn actual_multiset(relative: &str, caps: &ScopeCaps) -> Vec<FindingKey> {
    let model = fixture(relative);
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match decorator::Lens.run(&input, caps) {
        LensOutcome::Findings(findings) => {
            assert!(
                findings.migrations.is_empty(),
                "`decorator` has no migration surface: {:?}",
                findings.migrations
            );
            multiset(&findings)
        }
        LensOutcome::Unknown { scope } => {
            panic!("`decorator` on {relative} reported unknown: {scope}")
        }
    }
}

/// The recorded multiset for a completed outcome, over the checks channel.
/// Sorting keeps the comparison order-insensitive.
fn multiset(findings: &LensFindings) -> Vec<FindingKey> {
    let mut keys: Vec<FindingKey> = findings.checks.iter().map(check_key).collect();
    keys.sort_unstable();
    keys
}

/// One check finding as the taxonomy's stable `(kind, label)` spelling.
fn check_key(finding: &catherine_core::findings::Finding) -> FindingKey {
    let kind = match &finding.kind {
        FindingKind::Violation(_) => "violation",
        FindingKind::Dangling(_) => "dangling",
        FindingKind::Contradiction(_) => "contradiction",
        FindingKind::Underconstraint(_) => "underconstraint",
        FindingKind::Redundant(_) => "redundant",
        FindingKind::Granularity(_) => "granularity",
    };
    let label = match finding.label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    };
    (kind, label)
}

/// Load one repository-relative fixture model.
fn fixture(relative: &str) -> Model {
    let path = format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"));
    parse(&text).unwrap_or_else(|error| panic!("{relative} parses: {error:?}"))
}
