//! Purpose: the `proxy` lens Meter (beads `catherine-mx1.8`) — the
//! wrapper-laws identity-preservation evidence replayed through the lens
//! against the checked-in expectation table.
//!
//! Responsibilities: call [`proxy::Lens::run`] directly (never the
//! registry, which the orchestrator wires) on every `proxy` row of
//! `pattern_expectations.rs`, assert the recorded `(kind, label)` multiset
//! for the row's fixture, and pin the ledger's exact witnesses for the
//! proxy identity equation and for totality; also assert the metadata and
//! the seam's fail-closed `Unknown` arms.
//!
//! Rationale: the module under test is included from `src/` by `#[path]`
//! because `pattern/mod.rs` is the orchestrator's registry file, not a lens
//! ticket's; the re-export shims below give the module the same
//! `crate::bounds` / `crate::pattern` paths it sees inside the library, so
//! the tested source is the shipped source unchanged. The comparison is a
//! multiset, never a sequence, matching the replay harness; the rows are
//! the sealed ledger's, so a mismatch is a lens bug, never a table edit.
//! The floor control's three faults are three separate fault fixtures with
//! one recorded element each, and the clean control row beside them
//! requires silence.

#[path = "../src/pattern/proxy.rs"]
mod proxy;

mod bounds {}

mod pattern {
    pub use catherine_analysis::pattern::{
        LawKind, LensCall, LensFindings, LensOutcome, PatternLens, PatternMeta,
    };
}

#[path = "pattern_expectations.rs"]
mod expectations;

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{LensCall, LensFindings, LensInput, LensOutcome, PatternLens};
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, Finding, FindingKind, ViolationWitness,
};
use catherine_core::model::{Model, RowId};
use catherine_core::parse::parse;
use expectations::{Expectation, FindingKey, D, EXPECTATIONS, V};

/// The clean floor control the table's C rows name.
const FLOOR_CONTROL: &str = "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json";

#[test]
fn the_proxy_metadata_is_the_disposition_row() {
    let meta = proxy::Lens.meta();
    assert_eq!(meta.id, "proxy");
    assert_eq!(meta.name, "Proxy");
    assert_eq!(meta.abstraction, "wrapper law preservation");
    assert_eq!(meta.law, catherine_analysis::pattern::LawKind::Coverage);
    assert_eq!(meta.law.as_str(), "coverage");
    assert_eq!(
        meta.scope,
        "identity preservation through delegation; coverage-only"
    );
}

#[test]
fn every_recorded_proxy_row_replays_exactly() {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == "proxy")
        .collect();
    // The row count pins the sealed table's coverage, exactly like the
    // replay Meter: the floor's three faults, the clean arm, and the four
    // above-floor faults.
    assert_eq!(rows.len(), 8, "the checked-in table covers `proxy`");
    let caps = ScopeCaps::default();
    for row in rows {
        let model = fixture_model(row.fixture);
        let input = LensInput {
            model: &model,
            values: &[],
        };
        match proxy::Lens.run(LensCall {
            input: &input,
            caps: &caps,
        }) {
            LensOutcome::Findings(findings) => {
                assert!(
                    findings.migrations.is_empty(),
                    "`proxy` has no migration surface: {:?}",
                    findings.migrations
                );
                let mut expected = row.findings.to_vec();
                expected.sort_unstable();
                assert_eq!(
                    multiset(&findings),
                    expected,
                    "`proxy` on {} (fault `{}`, parity {})",
                    row.fixture,
                    row.fault,
                    row.parity
                );
            }
            LensOutcome::Unknown { scope } => {
                panic!("`proxy` on {} reported unknown: {scope}", row.fixture)
            }
        }
    }
}

#[test]
fn the_proxy_identity_equation_witness_is_the_recorded_one() {
    let findings = run_findings(
        "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json",
        &[],
    );
    assert_eq!(findings.checks.len(), 1, "{:?}", findings.checks);
    match &findings.checks[0].kind {
        FindingKind::Violation(ViolationWitness::Equation(witness)) => {
            assert_eq!(witness.equation, 4, "the proxy identity law's parsed index");
            assert_eq!(witness.row, RowId(1), "the rewired proxy row");
            assert_eq!((witness.lhs, witness.rhs), (RowId(1), RowId(0)));
        }
        other => panic!("the proxy identity law is the recorded equation violation: {other:?}"),
    }
}

#[test]
fn totality_names_every_dropped_cell() {
    let findings = run_findings(
        "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json",
        &[],
    );
    let dangling: Vec<&DanglingWitness> = findings
        .checks
        .iter()
        .filter_map(|finding| match &finding.kind {
            FindingKind::Dangling(witness) => Some(witness),
            _ => None,
        })
        .collect();
    assert_eq!(dangling.len(), 3, "{:?}", findings.checks);
    assert!(dangling.iter().all(|w| w.arrow.0 == "adapter_note"));
    let rows: Vec<usize> = dangling.iter().map(|w| w.src.0).collect();
    assert_eq!(rows, [0, 1, 2], "one dangling cell per dropped row");
}

#[test]
fn the_clean_arms_stay_silent() {
    for fixture in [
        "fixtures/katas/wrapper-laws/wrapper-laws.ct.json",
        FLOOR_CONTROL,
    ] {
        let findings = run_findings(fixture, &[]);
        assert!(
            findings.checks.is_empty(),
            "the clean arm {fixture} is silent: {:?}",
            findings.checks
        );
    }
}

#[test]
fn the_floor_control_evidence_replays_per_fault() {
    let cases = [
        (
            "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json",
            V,
        ),
        (
            "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json",
            V,
        ),
        (
            "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json",
            D,
        ),
    ];
    for (fixture, expected) in cases {
        let findings = run_findings(fixture, &[]);
        assert_eq!(multiset(&findings), vec![expected], "{fixture}");
    }
}

#[test]
fn a_document_without_an_equation_surface_is_unknown() {
    let model = Model::empty();
    let input = LensInput {
        model: &model,
        values: &[],
    };
    match proxy::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("surface")),
        LensOutcome::Findings(findings) => {
            panic!("an absent surface must not be clean: {findings:?}")
        }
    }
}

#[test]
fn a_value_declaration_the_equation_law_cannot_bind_is_unknown() {
    let model = fixture_model(FLOOR_CONTROL);
    let values = vec!["kind=lit".to_string()];
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match proxy::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Unknown { scope } => assert!(scope.contains("--values")),
        LensOutcome::Findings(_) => panic!("an unbound domain must not be evaluated"),
    }
}

/// Run the lens on one fixture and return its checks channel.
fn run_findings(fixture: &str, values: &[String]) -> LensFindings {
    let model = fixture_model(fixture);
    let input = LensInput {
        model: &model,
        values,
    };
    match proxy::Lens.run(LensCall {
        input: &input,
        caps: &ScopeCaps::default(),
    }) {
        LensOutcome::Findings(findings) => {
            assert!(findings.migrations.is_empty(), "no migration surface");
            findings
        }
        LensOutcome::Unknown { scope } => panic!("{fixture} reported unknown: {scope}"),
    }
}

/// The recorded multiset for a completed outcome, over the checks channel
/// (`proxy` has no migration surface). Sorting keeps the comparison
/// order-insensitive.
fn multiset(findings: &LensFindings) -> Vec<FindingKey> {
    let mut keys: Vec<FindingKey> = findings.checks.iter().map(check_key).collect();
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
    let label = match finding.label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    };
    (kind, label)
}

/// Load one repository-relative fixture model.
fn fixture_model(relative: &str) -> Model {
    let path = format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"));
    parse(&text).unwrap_or_else(|error| panic!("{relative} parses: {error:?}"))
}
