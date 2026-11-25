//! Purpose: the `composite` lens Meter (beads `catherine-mx1.3`) — holds
//! the shipped Identity lens to the sealed kata ledger's recorded
//! per-fixture `(kind, label)` multisets from
//! `tests/pattern_expectations.rs`.
//!
//! Responsibilities: run the lens directly (not through the registry),
//! compare each recorded `composite` row's checks-plus-migrations multiset
//! exactly, pin the near-floor control's two executed elements by their
//! witnesses, pin the flatten fault's seven-cell migrate channel, pin the
//! clean above-floor model's silence, and pin a `--values` request on a
//! lens with no rule table as `Unknown`, never as clean.
//!
//! Rationale: the production module is compiled here by `#[path]` because
//! the registry line belongs to the orchestrator's wiring ticket; the
//! root-level `bounds`/`pattern` aliases satisfy the module's
//! `crate::`-rooted imports in this second compilation. Parity rows (the
//! floor control and its two recorded faults) are asserted as plain
//! multisets — no unique-detection or detection-rate claim is made here,
//! and the expectation table is read, never edited.

use catherine_analysis::bounds::{self, ScopeCaps};
use catherine_analysis::pattern::{self, LensFindings, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::Finding;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, FindingKind, ViolationWitness,
};
use catherine_core::migrate::MigrationFinding;
use catherine_core::migrate::MigrationFindingKind;
use catherine_core::model::{Model, RowId};
use catherine_core::parse::parse;

#[path = "pattern_expectations.rs"]
mod expectations;

#[path = "../src/pattern/composite.rs"]
mod composite;

use expectations::{FindingKey, EXPECTATIONS};

/// A repository-relative fixture path.
fn fixture_path(relative: &str) -> String {
    format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"))
}

/// Load one fixture model.
fn fixture(relative: &str) -> Model {
    let text = std::fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"));
    parse(&text).unwrap_or_else(|error| panic!("{relative} parses: {error:?}"))
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

/// The recorded multiset over both channels, sorted for order-insensitivity.
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

/// Run the lens on one fixture, requiring a completed outcome.
fn run_on(relative: &str) -> LensFindings {
    let model = fixture(relative);
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match composite::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => {
            panic!("composite on {relative} reported unknown: {scope}")
        }
    }
}

/// Every recorded `composite` row: the lens's checks plus migrations must
/// be exactly the recorded multiset.
#[test]
fn every_composite_row_matches_the_recorded_multiset() {
    let mut replayed = 0;
    let caps = ScopeCaps::default();
    for row in EXPECTATIONS.iter().filter(|row| row.pattern == "composite") {
        replayed += 1;
        let text = std::fs::read_to_string(fixture_path(row.fixture))
            .unwrap_or_else(|error| panic!("{} is readable: {error}", row.fixture));
        if row.class == "construction" {
            assert!(parse(&text).is_err(), "{} is a refusal", row.fixture);
            continue;
        }
        let model =
            parse(&text).unwrap_or_else(|error| panic!("{} parses: {error:?}", row.fixture));
        let values: Vec<String> = row.values.iter().map(|spec| spec.to_string()).collect();
        let input = LensInput {
            model: &model,
            values: &values,
        };
        let findings = match composite::Lens.run(&input, &caps) {
            LensOutcome::Findings(findings) => findings,
            LensOutcome::Unknown { scope } => {
                panic!("composite on {} reported unknown: {scope}", row.fixture)
            }
        };
        let mut expected = row.findings.to_vec();
        expected.sort_unstable();
        assert_eq!(
            multiset(&findings),
            expected,
            "composite on {} (fault `{}`, parity {})",
            row.fixture,
            row.fault,
            row.parity
        );
    }
    assert_eq!(replayed, 6, "the table records six composite rows");
}

/// The near-floor control: one dangling parent cell and the L1 violation
/// the nil row's real parent produces, named by their witnesses.
#[test]
fn the_floor_control_names_its_two_executed_elements() {
    let findings = run_on("fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json");
    assert_eq!(findings.checks.len(), 2, "{:?}", findings.checks);
    let mut dangling = 0;
    let mut violations = 0;
    for finding in &findings.checks {
        assert_eq!(finding.label, EpistemicLabel::Executed);
        match &finding.kind {
            FindingKind::Dangling(DanglingWitness { arrow, src }) => {
                assert_eq!(arrow.0, "parent");
                assert_eq!(*src, RowId(0));
                dangling += 1;
            }
            FindingKind::Violation(witness) => {
                // nil's parent resolves to `root` (row 2), but `top(nil)`
                // is `nil` (row 1): the L1 equation fails on the nil row.
                assert_eq!(
                    witness,
                    &ViolationWitness::Equation(EquationWitness {
                        equation: 0,
                        row: RowId(1),
                        lhs: RowId(2),
                        rhs: RowId(1),
                    })
                );
                violations += 1;
            }
            other => panic!("unexpected control finding: {other:?}"),
        }
    }
    assert_eq!((dangling, violations), (1, 1));
    assert!(
        findings.migrations.is_empty(),
        "the control has no Δ surface"
    );
}

/// The clean above-floor and one-level models have nothing to report: their
/// silence is real success, not an unevaluated declaration.
#[test]
fn the_clean_composite_models_stay_silent() {
    for relative in [
        "fixtures/katas/composite-null-object/composite-null-object.ct.json",
        "fixtures/katas/composite-null-object/floor/floor-composite-null-object.ct.json",
    ] {
        assert_eq!(run_on(relative), LensFindings::default(), "{relative}");
    }
}

/// The flatten fault is a migrate surface: seven lost parent cells, no
/// check finding, each cell reported on the migrations channel only.
#[test]
fn the_flatten_fault_loses_every_valued_parent_cell() {
    let findings = run_on("fixtures/katas/composite-null-object/fault-lost-child-migrate.ct.json");
    assert!(
        findings.checks.is_empty(),
        "the source model's equations hold: {:?}",
        findings.checks
    );
    assert_eq!(findings.migrations.len(), 7);
    for (index, finding) in findings.migrations.iter().enumerate() {
        assert_eq!(finding.label, EpistemicLabel::Executed);
        match &finding.kind {
            MigrationFindingKind::LostCell { arrow, row } => {
                assert_eq!(arrow.0, "parent");
                assert_eq!(*row, RowId(index));
            }
            other => panic!("expected a lost cell, got {other:?}"),
        }
    }
}

/// A `--values` request on a lens with no rule table is the unevaluable
/// request: `Unknown` naming the mismatch, never a silently ignored flag.
#[test]
fn values_on_a_lens_without_a_rule_table_are_unknown() {
    let model = fixture("fixtures/katas/composite-null-object/composite-null-object.ct.json");
    let values = vec!["Node=c1".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match composite::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => {
            assert!(scope.contains("--values"), "names the flag: {scope}");
        }
        LensOutcome::Findings(findings) => {
            panic!("an unbindable request is not clean: {findings:?}")
        }
    }
}

/// The disposition row's metadata: id, name, identity law, and the honesty
/// boundary the kata verdict recorded.
#[test]
fn metadata_is_the_composite_disposition_row() {
    let meta = composite::Lens.meta();
    assert_eq!(meta.id, "composite");
    assert_eq!(meta.name, "Composite");
    assert_eq!(meta.abstraction, "monoid (identity)");
    assert_eq!(meta.law, pattern::LawKind::Identity);
    assert_eq!(meta.law.as_str(), "identity");
    assert_eq!(
        meta.scope,
        "identity/totality equations; associativity unexpressible"
    );
}
