//! Purpose: the `adapter` lens Meter (beads `catherine-mx1.6`) — the
//! wrapper-laws fixtures replayed through their link-level delegation and
//! field laws, against the checked-in expectation rows.
//!
//! Responsibilities: call [`adapter::Lens::run`] directly (never the
//! registry, which the orchestrator wires) on the recorded `adapter`
//! fixtures and assert the sealed ledger's `(kind, label)` multiset
//! exactly, pin the near-floor control's silence, pin the three floor
//! faults to their own recorded elements, and assert the metadata's
//! coverage-only honesty boundary.
//!
//! Rationale: the module under test is included from `src/` by `#[path]`
//! because `pattern/mod.rs` is the orchestrator's registry file, not a
//! lens ticket's; the alias modules below give the tested source the same
//! `crate::bounds` / `crate::pattern` paths it sees inside the library, so
//! the tested source is the shipped source unchanged. The three `C*`
//! table rows all name the shared floor control and repeat the ledger
//! entry's aggregate `[V, V, D]`; the entry's own `run.findings` maps that
//! aggregate one element per floor fault fixture, so the test replays the
//! fault fixtures and asserts the aggregate relation instead of asking a
//! clean fixture for findings it does not carry. Multisets are compared
//! order-insensitively, and no unique-detection or detection-rate claim is
//! made here.

#[path = "../src/pattern/adapter.rs"]
mod adapter;

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
use catherine_analysis::pattern::{LensInput, LensOutcome, PatternLens};
use catherine_core::findings::{EpistemicLabel, FindingKind};
use catherine_core::model::Model;
use catherine_core::parse::parse;
use expectations::{Expectation, FindingKey, EXPECTATIONS};

/// The floor control every `C*` row names.
const FLOOR_CONTROL: &str = "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json";

/// The floor fault fixtures and the one element the ledger records for
/// each (`violation:eq2@row0`, `violation:eq0@row0`,
/// `dangling:wrapper_note#0`).
const FLOOR_FAULTS: [(&str, FindingKey); 3] = [
    (
        "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json",
        ("violation", "executed"),
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json",
        ("violation", "executed"),
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json",
        ("dangling", "executed"),
    ),
];

#[test]
fn the_adapter_metadata_is_the_disposition_row() {
    let meta = adapter::Lens.meta();
    assert_eq!(meta.id, "adapter");
    assert_eq!(meta.name, "Adapter");
    assert_eq!(meta.abstraction, "wrapper law preservation");
    assert_eq!(meta.law, catherine_analysis::pattern::LawKind::Coverage);
    assert_eq!(meta.law.as_str(), "coverage");
    assert!(meta.scope.contains("delegation"));
    assert!(meta.scope.contains("coverage-only"));
}

#[test]
fn every_recorded_adapter_row_replays_exactly() {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "adapter" && !row.fault.starts_with('C'))
        .collect();
    assert_eq!(
        rows.len(),
        5,
        "the clean arm and the four above-floor faults"
    );
    for row in rows {
        let actual = run_keys(row.fixture);
        let mut expected = row.findings.to_vec();
        expected.sort_unstable();
        assert_eq!(
            actual, expected,
            "`adapter` on {} (fault `{}`, parity {})",
            row.fixture, row.fault, row.parity
        );
    }
}

#[test]
fn the_near_floor_control_is_silent() {
    assert_eq!(
        run_keys(FLOOR_CONTROL),
        Vec::<FindingKey>::new(),
        "a law battery that fires on its own control invalidates the kata"
    );
}

#[test]
fn the_floor_faults_replay_their_recorded_elements() {
    for (relative, expected) in FLOOR_FAULTS {
        assert_eq!(
            run_keys(relative),
            vec![expected],
            "the ledger records one element for {relative}"
        );
    }
}

#[test]
fn the_floor_rows_record_one_element_per_fault_and_aggregate_to_three() {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "adapter" && row.fault.starts_with('C'))
        .collect();
    assert_eq!(rows.len(), 3, "the floor entry records three faults");
    let mut aggregate: Vec<FindingKey> = Vec::new();
    for row in &rows {
        assert_ne!(
            row.fixture, FLOOR_CONTROL,
            "C rows name their fault fixture"
        );
        let mut keys = run_keys(row.fixture);
        keys.sort_unstable();
        let mut expected = row.findings.to_vec();
        expected.sort_unstable();
        assert_eq!(
            keys, expected,
            "the row's own fixture matches its recorded element"
        );
        aggregate.extend(keys);
    }
    let mut recorded: Vec<FindingKey> = rows
        .iter()
        .flat_map(|row| row.findings.iter().copied())
        .collect();
    aggregate.sort_unstable();
    recorded.sort_unstable();
    assert_eq!(
        aggregate, recorded,
        "the three per-fault elements aggregate to the entry's recorded multiset"
    );
}

#[test]
fn an_empty_model_is_a_completed_silence_not_an_unknown() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match adapter::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Findings(findings) => {
            assert!(findings.checks.is_empty());
            assert!(findings.migrations.is_empty());
        }
        LensOutcome::Unknown { scope } => {
            panic!("an empty presentation evaluates to zero findings, not unknown: {scope}")
        }
    }
}

/// Run the lens over one fixture and return its sorted checks multiset.
/// The adapter lens has no migration surface, so an unexpected migration
/// finding fails loudly rather than being compared.
fn run_keys(relative: &str) -> Vec<FindingKey> {
    let model = fixture(relative);
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match adapter::Lens.run(&input, &ScopeCaps::default()) {
        LensOutcome::Findings(findings) => {
            assert!(
                findings.migrations.is_empty(),
                "`adapter` has no migration surface: {:?}",
                findings.migrations
            );
            let mut keys: Vec<FindingKey> = findings.checks.iter().map(check_key).collect();
            keys.sort_unstable();
            keys
        }
        LensOutcome::Unknown { scope } => {
            panic!("`adapter` on {relative} reported unknown: {scope}")
        }
    }
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
