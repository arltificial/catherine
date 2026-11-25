//! Purpose: the Strategy lens Meter (beads `catherine-mx1.14`) — every
//! recorded row of `ledger/katas/cor-strategy-rules/` replayed against
//! `pattern::strategy::Lens` directly, never through the registry, so the
//! sealed evidence gates the lens before the orchestrator wires it.
//!
//! Responsibilities: the disposition-row metadata, the recorded
//! `(kind, label)` multiset per fixture, the silent parity/clean arms, and
//! the three bounded non-answers (absent surface, unbound values, expired
//! candidate budget). The comparison is the replay harness's own
//! two-channel multiset over checks and migrations.
//!
//! Rationale: the table is checked in and the ledger sealed, so a mismatch
//! is a lens bug, never a table edit. The lens module is included through
//! `#[path]` because the production registry is the orchestrator's wiring
//! step; the re-export shims below mirror the library's module roots so the
//! source compiles in this test scope with its `crate::…` imports unchanged.
//! Parity is recorded, not asserted as a win: no row makes a unique-detection
//! claim, and the silent rows must stay silent.

#[path = "pattern_expectations.rs"]
mod expectations;

#[path = "pattern_reference.rs"]
#[allow(dead_code)]
mod reference;

#[path = "pattern_replay/harness.rs"]
#[allow(dead_code)]
mod harness;

/// The library's module roots under the names the lens source imports.
mod bounds {}

/// The seam's types and helpers, re-exported for the lens source.
mod pattern {
    pub use catherine_analysis::pattern::*;
}

/// The rule lens the Strategy lens reuses, re-exported for the lens source.
mod rule_lens {
    pub use catherine_analysis::rule_lens::*;
}

#[path = "../src/pattern/strategy.rs"]
mod strategy;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LawKind, LensCall, LensInput, LensOutcome, PatternLens};
use expectations::{Expectation, EXPECTATIONS};
use harness::{compare, fixture};
use strategy::Lens;

/// The registry id this Meter gates.
const ID: &str = "strategy";

/// The recorded rows for this lens, in table order.
fn recorded_rows() -> Vec<&'static Expectation> {
    let selected: Vec<&'static Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == ID)
        .collect();
    assert_eq!(selected.len(), 13, "the checked-in table covers `{ID}`");
    selected
}

/// The first recorded row, for the non-answer tests' fixture and values.
fn first_row() -> &'static Expectation {
    recorded_rows()
        .into_iter()
        .next()
        .expect("the table has strategy rows")
}

/// Run the lens over one row's fixture and bound values, under the default
/// caps the recorded commands used.
fn run_row(row: &Expectation) -> LensOutcome {
    let model = fixture(row.fixture);
    let values: Vec<String> = row.values.iter().map(|spec| spec.to_string()).collect();
    let input = LensInput {
        model: &model,
        values: &values,
    };
    Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    })
}

/// A completed outcome carrying exactly the row's recorded multiset. The
/// recorded `class` is part of the evidence: `finding` rows carry a non-empty
/// multiset, `gap` rows are the ledger's silent arms (no `construction` row
/// exists for this pattern — every fixture parses).
fn assert_row(row: &Expectation) {
    assert!(
        row.class == "finding" || row.class == "gap",
        "fault `{}` has an in-scope recorded class, not `{}`",
        row.fault,
        row.class
    );
    if row.class == "gap" {
        assert!(
            row.findings.is_empty(),
            "a gap row is silent: {}",
            row.fault
        );
    }
    match run_row(row) {
        LensOutcome::Findings(findings) => {
            if let Err(detail) = compare(&findings, row.findings) {
                panic!(
                    "`{ID}` on {} (fault `{}`, parity {}): {detail}; checks {:?}, migrations {:?}",
                    row.fixture, row.fault, row.parity, findings.checks, findings.migrations
                );
            }
        }
        LensOutcome::Unknown { scope } => {
            panic!("`{ID}` on {} reported unknown: {scope}", row.fixture)
        }
    }
}

#[test]
fn metadata_is_the_strategy_disposition_row() {
    let meta = Lens.meta();
    assert_eq!(meta.id, "strategy");
    assert_eq!(meta.name, "Strategy");
    assert_eq!(meta.abstraction, "branch selection as rule regions");
    assert_eq!(meta.law, LawKind::Order);
    assert_eq!(meta.law.as_str(), "order");
    assert_eq!(
        meta.scope,
        "rule-lens regions above the floor; no detection claim"
    );
}

#[test]
fn every_recorded_strategy_row_replays() {
    let rows = recorded_rows();
    assert!(!rows.is_empty(), "the ledger records strategy rows");
    for row in rows {
        assert_row(row);
    }
}

#[test]
fn the_recorded_silent_rows_stay_silent() {
    let silent: Vec<&'static Expectation> = recorded_rows()
        .into_iter()
        .filter(|row| row.findings.is_empty())
        .collect();
    assert_eq!(silent.len(), 5, "the clean and gap-control arms");
    for row in silent {
        match run_row(row) {
            LensOutcome::Findings(findings) => {
                assert!(findings.checks.is_empty(), "{}: {findings:?}", row.fault);
                assert!(
                    findings.migrations.is_empty(),
                    "{}: {findings:?}",
                    row.fault
                );
            }
            LensOutcome::Unknown { scope } => {
                panic!("`{ID}` on {} reported unknown: {scope}", row.fixture)
            }
        }
    }
}

#[test]
fn a_model_without_a_rule_presentation_is_unknown_not_clean() {
    let model = catherine_core::model::Model::empty();
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
    let row = first_row();
    let model = fixture(row.fixture);
    let values = vec!["nope=1".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("does not declare")),
        LensOutcome::Findings(_) => panic!("an unbound domain must not be evaluated"),
    }
}

#[test]
fn an_expired_candidate_budget_is_unknown() {
    let row = first_row();
    let model = fixture(row.fixture);
    let values: Vec<String> = row.values.iter().map(|spec| spec.to_string()).collect();
    let input = LensInput {
        model: &model,
        values: &values,
    };
    let caps = ScopeCaps {
        max_candidates: 0,
        ..ScopeCaps::default()
    };
    match Lens.run(LensCall {
        input: &input,
        caps: &caps,
    }) {
        LensOutcome::Unknown { scope } => {
            assert!(scope.contains("nothing decided"), "scope: {scope}")
        }
        LensOutcome::Findings(_) => panic!("a zero-size budget must be a bounded non-answer"),
    }
}
