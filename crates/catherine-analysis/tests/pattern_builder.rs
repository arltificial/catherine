//! Purpose: the `builder` lens Meter (beads `catherine-mx1.5`) — holds the
//! shipped accumulation lens to the sealed `builder-monoid` kata's
//! per-fixture `(kind, label)` multisets from
//! `tests/pattern_expectations.rs`.
//!
//! Responsibilities: run the lens directly (not through the registry),
//! compare each recorded `builder` row's checks-plus-migrations multiset
//! exactly, pin the floor control's lost trace cell, the near-floor
//! duplicate row's silence, the dropped step's five lost rows plus twenty
//! lost cells, the duplicated law's two `redundant` findings, the splice's
//! one order violation, and the optional part's single lost cell; and pin
//! an unevaluable request (`--values`, no composition law) as `Unknown`,
//! never as clean.
//!
//! Rationale: the production module is compiled here by `#[path]` because
//! the registry line belongs to the orchestrator's wiring ticket; the
//! root-level `bounds`/`pattern`/`rewrite` aliases satisfy the module's
//! `crate::`-rooted imports in this second compilation. Parity rows (the
//! floor control, the duplicated step row) are asserted as plain
//! multisets — no unique-detection or detection-rate claim is made here,
//! and the expectation table is read, never edited.

use catherine_analysis::bounds::{self, ScopeCaps};
use catherine_analysis::pattern::{self, LensFindings, LensInput, LensOutcome, PatternLens};
use catherine_analysis::rewrite;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, RedundantWitness,
    ViolationWitness,
};
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::{Ident, Model, Path, RowId};
use catherine_core::parse::parse;

#[path = "pattern_expectations.rs"]
mod expectations;

#[path = "../src/pattern/builder.rs"]
mod builder;

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
    };
    (kind, label_name(finding.label))
}

/// One migration finding as the migrate surface's stable spelling.
fn migration_key(finding: &MigrationFinding) -> FindingKey {
    let kind = match &finding.kind {
        MigrationFindingKind::LostRow { .. } => "lost_row",
        MigrationFindingKind::LostCell { .. } => "lost_cell",
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
    match builder::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Findings(findings) => findings,
        LensOutcome::Unknown { scope } => panic!("builder on {relative} reported unknown: {scope}"),
    }
}

/// Every recorded `builder` row: the lens's checks plus migrations must be
/// exactly the recorded multiset.
#[test]
fn every_builder_row_matches_the_recorded_multiset() {
    let mut replayed = 0;
    let caps = ScopeCaps::default();
    for row in EXPECTATIONS.iter().filter(|row| row.pattern == "builder") {
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
        let findings = match builder::Lens.run(&input, &caps) {
            LensOutcome::Findings(findings) => findings,
            LensOutcome::Unknown { scope } => {
                panic!("builder on {} reported unknown: {scope}", row.fixture)
            }
        };
        let mut expected = row.findings.to_vec();
        expected.sort_unstable();
        assert_eq!(
            multiset(&findings),
            expected,
            "builder on {} (fault `{}`, parity {})",
            row.fixture,
            row.fault,
            row.parity
        );
    }
    assert_eq!(replayed, 7, "the table records seven builder rows");
}

/// The floor control: the step row is gone, so the surviving product part's
/// trace cell is the only executed element.
#[test]
fn the_floor_control_names_the_lost_trace_cell() {
    let findings = run_on("fixtures/katas/builder-monoid/floor/faults/01-dropped-step.ct.json");
    assert_eq!(findings.checks.len(), 1, "{:?}", findings.checks);
    assert_eq!(findings.checks[0].label, EpistemicLabel::Executed);
    assert_eq!(
        findings.checks[0].kind,
        FindingKind::Dangling(DanglingWitness {
            arrow: Ident("part_step".to_string()),
            src: RowId(0),
        })
    );
    assert!(
        findings.migrations.is_empty(),
        "the control has no Δ surface"
    );
}

/// The near-floor duplicate row is the recorded gap: instance multiplicity
/// is not a path equation, so the lens stays silent.
#[test]
fn the_near_floor_duplicate_row_stays_silent() {
    assert_eq!(
        run_on("fixtures/katas/builder-monoid/floor/faults/02-duplicated-step.ct.json"),
        LensFindings::default()
    );
}

/// The dropped step is the Δ surface: five lost `Step` rows and twenty lost
/// cells, one per dropped arrow and row, on the migrations channel only.
#[test]
fn the_dropped_step_reports_every_lost_row_and_cell() {
    let findings = run_on("fixtures/katas/builder-monoid/faults/01-dropped-step.ct.json");
    assert!(findings.checks.is_empty(), "{:?}", findings.checks);
    assert_eq!(findings.migrations.len(), 25);
    let mut rows = Vec::new();
    let mut cells = Vec::new();
    for finding in &findings.migrations {
        assert_eq!(finding.label, EpistemicLabel::Executed);
        match &finding.kind {
            MigrationFindingKind::LostRow { object, row } => {
                assert_eq!(object.0, "Step");
                rows.push(row.0);
            }
            MigrationFindingKind::LostCell { arrow, row } => {
                cells.push((arrow.0.clone(), row.0));
            }
        }
    }
    assert_eq!(rows, vec![0, 1, 2, 3, 4]);
    cells.sort();
    let expected: Vec<(String, usize)> = ["emits", "part_step", "step_prev", "step_product"]
        .iter()
        .flat_map(|arrow| (0..5).map(move |row| (arrow.to_string(), row)))
        .collect();
    assert_eq!(cells, expected);
}

/// The duplicated trace law is derivable from its own copy, so the rule
/// analysis reports the same witness twice; the duplicate row itself is
/// parity and produces no check finding.
#[test]
fn the_duplicated_step_proves_the_duplicated_law_twice() {
    let findings = run_on("fixtures/katas/builder-monoid/faults/02-duplicated-step.ct.json");
    assert!(findings.migrations.is_empty(), "no Δ surface in this fault");
    let witness = RedundantWitness {
        derivable: Path(vec![
            Ident("part_step".to_string()),
            Ident("step_product".to_string()),
        ]),
        reducing: Path(vec![Ident("part_product".to_string())]),
    };
    assert_eq!(findings.checks.len(), 2, "{:?}", findings.checks);
    for finding in &findings.checks {
        assert_eq!(finding.label, EpistemicLabel::Executed);
        assert_eq!(finding.kind, FindingKind::Redundant(witness.clone()));
    }
}

/// The cross-product splice fails the order equation on the spliced step
/// row, named by the recorded witness.
#[test]
fn the_spliced_chain_is_one_order_violation() {
    let findings = run_on("fixtures/katas/builder-monoid/faults/03-out-of-order-step.ct.json");
    assert!(findings.migrations.is_empty(), "no Δ surface in this fault");
    assert_eq!(findings.checks.len(), 1, "{:?}", findings.checks);
    assert_eq!(
        findings.checks[0].kind,
        FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: 2,
            row: RowId(2),
            lhs: RowId(1),
            rhs: RowId(0),
        }))
    );
}

/// The optional part's lost map cell is the only finding: the part is not
/// emitted by a step and no accepted equation constrains it, so its absent
/// trace cells are outside the recorded law.
#[test]
fn the_optional_part_is_a_migrate_cell_only() {
    let findings = run_on("fixtures/katas/builder-monoid/faults/04-optional-part-dropped.ct.json");
    assert!(findings.checks.is_empty(), "{:?}", findings.checks);
    assert_eq!(findings.migrations.len(), 1);
    assert_eq!(
        findings.migrations[0].kind,
        MigrationFindingKind::LostCell {
            arrow: Ident("optional_part".to_string()),
            row: RowId(0),
        }
    );
}

/// A document that declares no accepted composition equation is an
/// unevaluable request, not a clean run.
#[test]
fn a_document_without_the_law_is_unknown_not_clean() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match builder::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => {
            assert!(scope.contains("composition"), "names the surface: {scope}");
        }
        LensOutcome::Findings(findings) => {
            panic!("an absent law must not be clean: {findings:?}")
        }
    }
}

/// A `--values` request on a lens with no rule table is the unevaluable
/// request: `Unknown` naming the mismatch, never a silently ignored flag.
#[test]
fn values_on_a_lens_without_a_rule_table_are_unknown() {
    let model = fixture("fixtures/katas/builder-monoid/builder.ct.json");
    let values = vec!["Step=s1_0".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match builder::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Unknown { scope } => {
            assert!(scope.contains("--values"), "names the flag: {scope}");
        }
        LensOutcome::Findings(findings) => {
            panic!("an unbindable request is not clean: {findings:?}")
        }
    }
}

/// The disposition row's metadata: id, name, order law, and the honesty
/// boundary the kata verdict recorded (duplicate-row parity).
#[test]
fn metadata_is_the_builder_disposition_row() {
    let meta = builder::Lens.meta();
    assert_eq!(meta.id, "builder");
    assert_eq!(meta.name, "Builder");
    assert_eq!(meta.abstraction, "monoid (accumulation)");
    assert_eq!(meta.law, pattern::LawKind::Order);
    assert_eq!(meta.law.as_str(), "order");
    assert!(meta.scope.contains("traces to a declared step"));
    assert!(meta.scope.contains("splice order equation"));
    assert!(meta.scope.contains("parity"));
}
