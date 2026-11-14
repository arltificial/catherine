//! Purpose: the replay harness behind `tests/pattern_replay.rs` — the
//! per-row execution of the checked-in expectation table and the
//! two-channel `(kind, label)` multiset comparison.
//!
//! Responsibilities: resolve the lens under test (production `registry()`
//! first, then the TEST-ONLY reference slice from `pattern_reference.rs`),
//! load each expectation's fixture, run the lens, compare the recorded
//! multiset over both finding channels — the P2 checks and the migration
//! engine's `lost_row`/`lost_cell` findings — and pin the seam's
//! fail-closed refusal for an unregistered id.
//!
//! Rationale: kept beside the replay tests so no test file crosses the
//! file-size gate; the target's behaviour is unchanged. A row whose
//! recorded kinds are migration kinds is satisfiable only by a lens's
//! `migrations` list; a checks-only or empty outcome fails it loudly,
//! never silently. The comparison sorts, so a row is matched as a
//! multiset, never as a sequence.

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::{
    self, LensError, LensFindings, LensInput, LensOutcome, PatternLens,
};
use catherine_core::findings::{EpistemicLabel, Finding, FindingKind};
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::Model;
use catherine_core::parse::parse;

use crate::expectations::{Expectation, FindingKey, EXPECTATIONS};
use crate::reference;

/// Run every table row for one pattern id and return the number of rows
/// replayed. A row whose fixture the parser refuses (`construction`)
/// asserts the refusal; a row whose lens is not registered yet asserts the
/// seam's fail-closed refusal.
pub fn replay(id: &str) -> usize {
    let rows: Vec<&Expectation> = EXPECTATIONS
        .iter()
        .filter(|row| row.pattern == id)
        .collect();
    assert!(!rows.is_empty(), "the checked-in table covers `{id}`");
    let caps = ScopeCaps::default();
    for &row in &rows {
        replay_row(id, row, &caps);
    }
    rows.len()
}

/// Execute one table row: assert the recorded parse refusal for a
/// `construction` row, otherwise run the resolved lens (or the fail-closed
/// refusal when the lens is not registered).
fn replay_row(id: &str, row: &Expectation, caps: &ScopeCaps) {
    let text = std::fs::read_to_string(fixture_path(row.fixture))
        .unwrap_or_else(|error| panic!("{} is readable: {error}", row.fixture));
    if row.class == "construction" {
        assert!(
            parse(&text).is_err(),
            "{} is the recorded parse refusal",
            row.fixture
        );
        return;
    }
    let model = parse(&text).unwrap_or_else(|error| panic!("{} parses: {error:?}", row.fixture));
    let values: Vec<String> = row.values.iter().map(|spec| spec.to_string()).collect();
    let input = LensInput {
        model: &model,
        values: &values,
    };
    match resolve(id) {
        Some(lens) => assert_completed(id, row, lens, &input, caps),
        None => assert_unregistered(id, &input, caps),
    }
}

/// A completed run must match the row exactly; a bounded non-answer is a
/// failure because the row's fixture is inside the lens's domain.
fn assert_completed(
    id: &str,
    row: &Expectation,
    lens: &'static dyn PatternLens,
    input: &LensInput<'_>,
    caps: &ScopeCaps,
) {
    match lens.run(input, caps) {
        LensOutcome::Findings(findings) => assert_findings_match(id, row, &findings),
        LensOutcome::Unknown { scope } => {
            panic!("`{id}` on {} reported unknown: {scope}", row.fixture)
        }
    }
}

/// Compare a completed outcome with its row; the mismatch names the row's
/// fixture, fault, parity, and both multisets.
fn assert_findings_match(id: &str, row: &Expectation, findings: &LensFindings) {
    if let Err(detail) = compare(findings, row.findings) {
        panic!(
            "`{id}` on {} (fault `{}`, parity {}): {detail}",
            row.fixture, row.fault, row.parity
        );
    }
}

/// An unregistered lens is the seam's fail-closed `UnknownPattern` naming
/// the requested id.
fn assert_unregistered(id: &str, input: &LensInput<'_>, caps: &ScopeCaps) {
    let error = pattern::run(id, input, caps).expect_err("an unregistered lens");
    match error {
        LensError::UnknownPattern { id: got, .. } => assert_eq!(got, id),
    }
}

/// The production registry shadows the test-local slice: a landed lens gates
/// these rows; until then the reference Visitor lens proves the seam.
fn resolve(id: &str) -> Option<&'static dyn PatternLens> {
    pattern::find(id).or_else(|| {
        reference::REFERENCE_LENSES
            .iter()
            .copied()
            .find(|lens| lens.meta().id == id)
    })
}

/// The recorded multiset for a completed outcome, over both channels: the
/// P2 checks by their kind, the migration findings by their lost-data kind.
/// Sorting keeps the comparison order-insensitive, so a row is matched as a
/// multiset, never as a sequence.
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

/// Compare one completed outcome with a row's recorded multiset. `Err`
/// carries both sides, so a mismatch names what was recorded and what the
/// lens actually reported.
pub fn compare(findings: &LensFindings, expected: &[FindingKey]) -> Result<(), String> {
    let actual = multiset(findings);
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}, got {actual:?}"))
    }
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

/// One migration finding as the migrate surface's stable `(kind, label)`
/// spelling — `lost_row`/`lost_cell`, never a P2 kind.
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

/// A repository-relative fixture path.
fn fixture_path(relative: &str) -> String {
    format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"))
}

/// Load one fixture model.
pub fn fixture(relative: &str) -> Model {
    let text = std::fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"));
    parse(&text).unwrap_or_else(|error| panic!("{relative} parses: {error:?}"))
}
