//! Purpose: the replay gate (openspec change `add-pattern-lenses`, design
//! Decision 7; beads `catherine-mx1.2`) — every pattern lens is held to the
//! sealed kata ledger's recorded `(kind, label)` multisets, per fixture,
//! with parity reported as parity only.
//!
//! Responsibilities: resolve the lens under test (production `registry()`
//! first, then the TEST-ONLY reference slice from `pattern_reference.rs`),
//! load each expectation's fixture, run the lens, and assert the recorded
//! `(kind, label)` multiset exactly over both finding channels — the P2
//! checks and the migration engine's `lost_row`/`lost_cell` findings. A row
//! whose recorded kinds are migration kinds is therefore satisfiable only
//! by a lens's `migrations` list; a checks-only or empty outcome fails it
//! loudly, never silently. It also pins the seam's fail-closed paths: an
//! absent lens is
//! [`catherine_analysis::pattern::LensError::UnknownPattern`], a model
//! without the lens's surface is `Unknown { scope }`, an expired budget is
//! `Unknown`, and `gap`/`construction` rows stay silent. Test names are the
//! pattern ids (Rust's underscore spelling), so
//! `cargo test -p catherine-analysis --test pattern_replay <id>` is the
//! per-lens Meter.
//!
//! Rationale: the expectation table is checked in and the ledger is sealed,
//! so a mismatch is a lens bug or a spec bug — never a table edit. The
//! migration class's findings are `MigrationFinding`, not `Finding`, so the
//! multiset is composed channel by channel: flattening them into the P2
//! taxonomy would invent a kind the taxonomy does not have, and comparing
//! only the checks would let a Δ row pass on an empty list. The harness has
//! no unique-detection output and no detection-rate assertion: a parity row
//! neither wins nor loses. The foundation ships zero production lenses, so
//! the test-local slice proves the seam; when the orchestrator wires a lens
//! into `registry()`, the production lookup shadows the reference and the
//! same rows gate it unchanged. The row execution and multiset comparison
//! live in `pattern_replay/harness.rs`; each Meter also asserts the row
//! count it replayed, so a silently shortened table fails the pattern.

#[path = "pattern_expectations.rs"]
mod expectations;

#[path = "pattern_reference.rs"]
#[allow(dead_code)]
mod reference;

#[path = "pattern_replay/harness.rs"]
mod harness;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{
    self, LensError, LensFindings, LensInput, LensOutcome, PatternLens,
};
use catherine_core::model::Model;
use expectations::{FindingKey, EXPECTATIONS};
use harness::{compare, fixture, replay};

/// The seam's fail-closed refusal names the id and lists the registry ids.
#[test]
fn find_fails_closed_on_an_unknown_id() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    let error = pattern::run("nope", &input, &ScopeCaps::default()).expect_err("not registered");
    match error {
        LensError::UnknownPattern { id, known } => {
            assert_eq!(id, "nope");
            assert_eq!(known, pattern::known_ids());
        }
    }
    assert_eq!(
        LensError::UnknownPattern {
            id: "nope".to_string(),
            known: vec![],
        }
        .code(),
        "bad_argument"
    );
    assert!(pattern::find("nope").is_none());
}

/// Registry metadata is complete and unique; the literal is ordered by id.
#[test]
fn registry_metadata_is_complete_and_unique() {
    let lens_list = pattern::registry();
    let ids: Vec<&str> = lens_list.iter().map(|lens| lens.meta().id).collect();
    assert!(
        ids.windows(2).all(|pair| pair[0] < pair[1]),
        "the registry literal is ordered by id and unique: {ids:?}"
    );
    for lens in lens_list {
        let meta = lens.meta();
        assert!(!meta.id.is_empty());
        assert!(!meta.name.is_empty());
        assert!(!meta.abstraction.is_empty());
        assert!(!meta.scope.is_empty());
        assert!(matches!(
            meta.law.as_str(),
            "identity" | "totality" | "coverage" | "order" | "delta"
        ));
        assert_eq!(
            pattern::find(meta.id).map(|found| found.meta().id),
            Some(meta.id)
        );
    }
}

/// The checked-in table covers exactly the twelve in-scope patterns, with
/// the recorded fault metadata on every row.
#[test]
fn the_table_covers_all_twelve_in_scope_patterns() {
    let expected = [
        "adapter",
        "builder",
        "chain-of-responsibility",
        "composite",
        "decorator",
        "interpreter",
        "memento",
        "null-object",
        "proxy",
        "state",
        "strategy",
        "visitor",
    ];
    let mut patterns: Vec<&str> = EXPECTATIONS.iter().map(|row| row.pattern).collect();
    patterns.sort_unstable();
    patterns.dedup();
    assert_eq!(patterns, expected);
    assert!(EXPECTATIONS.iter().all(|row| {
        !row.fixture.is_empty()
            && !row.fault.is_empty()
            && matches!(row.class, "finding" | "gap" | "construction")
    }));
}

/// The visitor entry pre-registers precisely three unique classes (F3/F4/F5);
/// F1/F2 and the floor control are parity. The table records that, and no
/// unique-detection count is asserted.
#[test]
fn the_table_records_the_three_visitor_unique_classes() {
    let unique: Vec<&str> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "visitor" && !row.parity)
        .map(|row| row.fault)
        .collect();
    assert_eq!(unique, ["F3", "F4", "F5"]);
    let visitor_keys: Vec<FindingKey> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "visitor" && !row.parity)
        .flat_map(|row| row.findings.iter().copied())
        .collect();
    assert_eq!(
        visitor_keys,
        [
            ("redundant", "executed"),
            ("redundant", "executed"),
            ("underconstraint", "hypothesized")
        ]
    );
}

/// A budget smaller than the reference lens's domain is a completed bounded
/// non-answer reporting the exact scope — never clean, never partial.
#[test]
fn a_bounded_non_answer_is_unknown() {
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
    match reference::VISITOR.run(&input, &caps) {
        LensOutcome::Unknown { scope } => {
            assert!(scope.contains("candidate"), "names the bound: {scope}")
        }
        LensOutcome::Findings(findings) => {
            panic!("a bounded non-answer is not clean: {findings:?}")
        }
    }
}

/// A lens with no migration channel cannot satisfy a recorded lost-data
/// row: the comparison spans both channels, so the row fails loudly instead
/// of passing on an empty check list — the silent-pass hole the widened
/// outcome exists to close.
#[test]
fn a_migrateless_lens_cannot_pass_a_lost_cell_row() {
    let row = EXPECTATIONS
        .iter()
        .find(|row| row.findings.iter().any(|key| key.0 == "lost_cell"))
        .expect("the table records lost-cell rows");
    assert!(
        row.findings.iter().all(|key| key.0.starts_with("lost_")),
        "the row is a pure migration row: {:?}",
        row.findings
    );
    let migrateless = LensFindings::default();
    let error = compare(&migrateless, row.findings)
        .expect_err("an empty findings list must not match a lost-cell row");
    assert!(
        error.contains("lost_cell"),
        "the failure names the missing kind: {error}"
    );
}

/// The replay Meter for `composite`; the row count pins table coverage.
#[test]
fn composite() {
    assert_eq!(replay("composite"), 6);
}

/// The replay Meter for `null-object`; the row count pins table coverage.
#[test]
fn null_object() {
    assert_eq!(replay("null-object"), 6);
}

/// The replay Meter for `builder`; the row count pins table coverage.
#[test]
fn builder() {
    assert_eq!(replay("builder"), 7);
}

/// The replay Meter for `adapter`; the row count pins table coverage.
#[test]
fn adapter() {
    assert_eq!(replay("adapter"), 8);
}

/// The replay Meter for `decorator`; the row count pins table coverage.
#[test]
fn decorator() {
    assert_eq!(replay("decorator"), 8);
}

/// The replay Meter for `proxy`; the row count pins table coverage.
#[test]
fn proxy() {
    assert_eq!(replay("proxy"), 8);
}

/// The replay Meter for `visitor`; the row count pins table coverage.
#[test]
fn visitor() {
    assert_eq!(replay("visitor"), 10);
}

/// The replay Meter for `interpreter`; the row count pins table coverage.
#[test]
fn interpreter() {
    assert_eq!(replay("interpreter"), 10);
}

/// The replay Meter for `memento`; the row count pins table coverage.
#[test]
fn memento() {
    assert_eq!(replay("memento"), 5);
}

/// The replay Meter for `state`; the row count pins table coverage.
#[test]
fn state() {
    assert_eq!(replay("state"), 10);
}

/// The replay Meter for `chain-of-responsibility`; the row count pins table
/// coverage.
#[test]
fn chain_of_responsibility() {
    assert_eq!(replay("chain-of-responsibility"), 13);
}

/// The replay Meter for `strategy`; the row count pins table coverage.
#[test]
fn strategy() {
    assert_eq!(replay("strategy"), 13);
}
