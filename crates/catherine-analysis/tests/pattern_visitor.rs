//! Purpose: the `visitor` lens Meter (beads `catherine-mx1.9`) — holds the
//! shipped Coverage lens to the sealed kata ledger's recorded per-fixture
//! `(kind, label)` multisets from `tests/pattern_expectations.rs`.
//!
//! Responsibilities: run the lens directly (not through the registry),
//! compare each recorded `visitor` row's checks-plus-migrations multiset
//! exactly, pin the near-floor control's silence, and pin the two bounded
//! non-answer paths (expired candidate budget, unbindable declared domain)
//! as `Unknown`, never as clean.
//!
//! Rationale: the production module is compiled here by `#[path]` because
//! the registry line belongs to the orchestrator's wiring ticket; the
//! root-level `bounds`/`pattern`/`rule_lens` aliases satisfy the module's
//! `crate::`-rooted imports in this second compilation. Parity rows
//! (F1/F2 and the floor control) are asserted as plain multisets — no
//! unique-detection or detection-rate claim is made here, and the
//! expectation table is read, never edited.

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{
    self, LensCall, LensFindings, LensInput, LensOutcome, PatternLens,
};
use catherine_analysis::rule_lens;
use catherine_core::findings::{EpistemicLabel, Finding, FindingKind};
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::Model;
use catherine_core::parse::parse;

#[path = "pattern_expectations.rs"]
mod expectations;

#[path = "../src/pattern/visitor.rs"]
mod visitor;

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

/// Every recorded `visitor` row: the lens's checks plus migrations must be
/// exactly the recorded multiset, and the Δ channel must stay empty.
#[test]
fn every_visitor_row_matches_the_recorded_multiset() {
    let mut replayed = 0;
    let caps = ScopeCaps::default();
    for row in EXPECTATIONS.iter().filter(|row| row.pattern == "visitor") {
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
        let findings = match visitor::Lens.run(LensCall {
            input: &input,
            caps: &caps,
        }) {
            LensOutcome::Findings(findings) => findings,
            LensOutcome::Unknown { scope } => {
                panic!("visitor on {} reported unknown: {scope}", row.fixture)
            }
        };
        assert!(
            findings.migrations.is_empty(),
            "{} (fault `{}`): the coverage lens has no Δ surface",
            row.fixture,
            row.fault
        );
        let mut expected = row.findings.to_vec();
        expected.sort_unstable();
        assert_eq!(
            multiset(&findings),
            expected,
            "visitor on {} (fault `{}`, parity {})",
            row.fixture,
            row.fault,
            row.parity
        );
    }
    assert_eq!(replayed, 10, "the table records ten visitor rows");
}

/// The near-floor control (two variants, both sampled by the conventional
/// dispatch test) stays silent: no check, no migration.
#[test]
fn the_near_floor_control_stays_silent() {
    let model = fixture("fixtures/katas/visitor-sum-types/floor/two-variant.ct.json");
    let values = vec!["kind=lit,neg".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match visitor::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Findings(findings) => assert_eq!(findings, LensFindings::default()),
        LensOutcome::Unknown { scope } => panic!("the floor control is decidable: {scope}"),
    }
}

/// A candidate budget below the declared domain is a bounded non-answer
/// naming its bound — never clean, never a partial finding list.
#[test]
fn a_bounded_budget_is_unknown() {
    let model = fixture("fixtures/katas/visitor-sum-types/visitor.ct.json");
    let values = vec!["kind=num,str,add,mul,neg".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    let caps = ScopeCaps {
        max_candidates: 1,
        ..ScopeCaps::default()
    };
    match visitor::Lens.run(LensCall {
        input: &input,
        caps: &caps,
    }) {
        LensOutcome::Unknown { scope } => {
            assert!(!scope.is_empty(), "the bound is spelled out: {scope}");
            assert!(scope.contains("candidate"), "names the bound: {scope}")
        }
        LensOutcome::Findings(findings) => panic!("a bounded run is not clean: {findings:?}"),
    }
}

/// A model with no rule presentation has nothing to evaluate: the exact
/// missing surface is reported, not an empty success.
#[test]
fn a_model_without_a_rule_presentation_is_unknown() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match visitor::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => {
            assert!(
                scope.contains("rule_presentation"),
                "names the surface: {scope}"
            )
        }
        LensOutcome::Findings(findings) => panic!("no presentation is not clean: {findings:?}"),
    }
}

/// A `--values` spelling the presentation does not declare is the domain
/// binder's refusal, surfaced verbatim as the lens's bounded non-answer.
#[test]
fn an_undeclared_domain_column_is_unknown() {
    let model = fixture("fixtures/katas/visitor-sum-types/visitor.ct.json");
    let values = vec!["nope=num".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match visitor::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => {
            assert!(!scope.is_empty(), "the refusal is spelled out");
            assert!(!values.is_empty(), "the refused spelling is present");
            assert_eq!(values.len(), 1, "the refusal covers one spelling")
        }
        LensOutcome::Findings(findings) => {
            panic!("an unbindable domain is not clean: {findings:?}")
        }
    }
}

/// The disposition row's metadata: id, name, coverage law, and the honesty
/// boundary the kata verdict recorded.
#[test]
fn metadata_is_the_visitor_disposition_row() {
    let meta = visitor::Lens.meta();
    assert_eq!(meta.id, "visitor");
    assert_eq!(meta.name, "Visitor");
    assert_eq!(meta.abstraction, "sum type / exhaustiveness");
    assert_eq!(meta.law, pattern::LawKind::Coverage);
    assert_eq!(meta.law.as_str(), "coverage");
    assert_eq!(
        meta.scope,
        "variant coverage + handler classes; no evaluation semantics"
    );
}
