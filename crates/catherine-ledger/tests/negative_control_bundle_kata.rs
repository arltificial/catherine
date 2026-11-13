//! Purpose: the recorded evidence of the negative-control-bundle kata
//! (`catherine-lgl.9`) — the suite's honesty gate. The committed slice
//! ledger validates, its fixture hashes are pinned, and the kernel's
//! silence is asserted exactly: zero findings on the unexpressible
//! Flyweight identity and the procedural CRUD snapshot, exactly one true
//! violation on the lookalike's drift (never the false legitimate-row
//! one), and the near-floor control naming the same element as the
//! scripted conventional arm.
//!
//! Responsibilities: pin every clean/fault fixture's canonical model
//! hash, execute the kernel over the fixtures (clean = zero findings;
//! faults = the pre-registered findings and witnesses), validate the
//! ledger directory through the real validator, and assert the
//! control-first/killed/empty-unique shape plus each positive's executed
//! baseline arm, recorded conventional catch, and expressibility gap.
//!
//! Rationale: a fabricated finding is a failure of the bundle, so the
//! fixtures are executable assertions, not prose. Same discipline as the
//! probe ledgers: an entry is evidence only while the files on disk still
//! hash to what it cites, and a null result is recorded as a null, never
//! rounded up to a detection.

use std::path::{Path, PathBuf};

use catherine_core::findings::{FindingKind, ViolationWitness};
use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::harness::run;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

const FIXTURES: [(&str, &str); 8] = [
    (
        "cache-control.ct.json",
        "sha256:e8f15b3ec44f8e6d930eb3178925cafe59c97ae354a40e470e33b3bb26eb8627",
    ),
    (
        "cache-control-fault-stale.ct.json",
        "sha256:ed6ddd795e07fa464839155fa66067fbb1a78337c7c98b7f4e74e41968f3b135",
    ),
    (
        "flyweight-shared.ct.json",
        "sha256:143ed2362abc7d0d0ebbfddf089eaab13ed7271661903f001b229c67a46bf3b3",
    ),
    (
        "flyweight-shared-fault-duplicate.ct.json",
        "sha256:cb9e568660d1506693be8a8e43025ad99949d2b9c0144054246e647e05d2c63a",
    ),
    (
        "crud-procedural.ct.json",
        "sha256:422dcce40aa6e40f608992b6ebce83fa7049eaa90ea8da95a44b40d24bca4616",
    ),
    (
        "crud-procedural-fault-stale-read.ct.json",
        "sha256:c9bb975a8fa8f42c610f1d90c89ae3dd827b2f82b86aeda57fb0ea7024e32f2f",
    ),
    (
        "decorator-lookalike.ct.json",
        "sha256:ed508a19c3ac4332436465a1eff5afd30ad099ac714208bf272674427cf1a82a",
    ),
    (
        "decorator-lookalike-fault-undiscounted-charge.ct.json",
        "sha256:a6c2132d9cda29c7e382cef338e3d190b9d7924a49a06ea0e44890fbed5b65bf",
    ),
];

/// The positives in ledger order: fault id, fixture, element, findings.
const POSITIVES: [(&str, &str, &str, usize); 3] = [
    (
        "flyweight-duplicate-instance",
        "flyweight-shared-fault-duplicate.ct.json",
        "flyweight:duplicate-instance@Client#1",
        0,
    ),
    (
        "crud-stale-read",
        "crud-procedural-fault-stale-read.ct.json",
        "crud:read_back!=created@Request#1",
        0,
    ),
    (
        "lookalike-undiscounted-charge",
        "decorator-lookalike-fault-undiscounted-charge.ct.json",
        "lookalike:charged!=list.discount@Product#1",
        1,
    ),
];

fn fixture_path(name: &str) -> PathBuf {
    Path::new(ROOT)
        .join("fixtures/katas/negative-control-bundle")
        .join(name)
}

/// One fixture's canonical model hash, recomputed from its bytes.
fn fixture_hash(name: &str) -> String {
    let text = std::fs::read_to_string(fixture_path(name)).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// An edit to any committed fixture must become a superseding entry.
#[test]
fn negative_control_fixture_hashes_are_pinned() {
    let mut mismatches = Vec::new();
    for (name, pin) in FIXTURES {
        let got = fixture_hash(name);
        println!("HASH {name} {got}");
        if got != pin {
            mismatches.push(format!("{name}: {got} != {pin}"));
        }
    }
    assert!(mismatches.is_empty(), "fixture hash drift: {mismatches:?}");
}

/// The clean fixtures are silent: no lens claim is fabricated.
#[test]
fn clean_fixtures_are_silent() {
    for name in [
        "cache-control.ct.json",
        "flyweight-shared.ct.json",
        "crud-procedural.ct.json",
        "decorator-lookalike.ct.json",
    ] {
        let executed = run(&fixture_path(name));
        assert!(executed.parsed, "{name} parses");
        assert!(
            executed.findings.is_empty(),
            "{name} must emit no finding, got {:?}",
            executed.findings
        );
    }
}

/// The equation witness of a violation finding, for exact comparisons.
fn violation_witness(
    finding: &catherine_core::findings::Finding,
) -> Option<(usize, usize, usize, usize)> {
    match &finding.kind {
        FindingKind::Violation(ViolationWitness::Equation(witness)) => Some((
            witness.equation,
            witness.row.0,
            witness.lhs.0,
            witness.rhs.0,
        )),
        _ => None,
    }
}

/// The seeded faults emit exactly the pre-registered findings: the
/// Flyweight duplicate and the CRUD snapshot are invisible (null), the
/// control is one coherence violation, and the lookalike is one true
/// drift — never the legitimately differing row.
#[test]
fn seeded_faults_emit_exactly_the_preregistered_findings() {
    for name in [
        "flyweight-shared-fault-duplicate.ct.json",
        "crud-procedural-fault-stale-read.ct.json",
    ] {
        let executed = run(&fixture_path(name));
        assert!(executed.parsed, "{name} parses");
        assert!(
            executed.findings.is_empty(),
            "{name} is null by design, got {:?}",
            executed.findings
        );
    }
    let control = run(&fixture_path("cache-control-fault-stale.ct.json"));
    assert_eq!(control.findings.len(), 1, "control emits one violation");
    assert_eq!(
        violation_witness(&control.findings[0]),
        Some((0, 0, 0, 1)),
        "the control witness is the conventional coherence failure"
    );
    let lookalike = run(&fixture_path(
        "decorator-lookalike-fault-undiscounted-charge.ct.json",
    ));
    assert_eq!(lookalike.findings.len(), 1, "one true lookalike drift");
    assert_eq!(
        violation_witness(&lookalike.findings[0]),
        Some((0, 1, 1, 0)),
        "the lookalike witness is Product row 1, the true drift"
    );
    assert!(
        lookalike
            .findings
            .iter()
            .all(|finding| violation_witness(finding).is_some_and(|w| w.1 != 0)),
        "no false violation on the legitimately differing Product row 0"
    );
}

/// The ledger validates control-first, the control self-negates, and
/// each positive carries its executed baseline arm, conventional catch,
/// and honest gap.
#[test]
fn negative_control_ledger_records_the_preregistered_dispositions() {
    let dir = Path::new(ROOT).join("ledger/katas/negative-control-bundle");
    let valid = validate_dir(&dir).expect("the bundle ledger validates");
    assert_eq!(valid.entries.len(), 4, "control + three positives");
    let control = valid.kata_entries()[0].content.kata.clone().expect("kata");
    assert!(control.control, "the near-floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    assert_eq!(control.expected[0].fault, "control-stale-cache");
    assert!(control.catalog_hash.is_none(), "non-probe kata, no catalog");
    assert_eq!(control.run.findings.len(), 1);
    assert_eq!(
        control
            .comparison
            .baseline
            .as_ref()
            .expect("arm")
            .detections,
        vec!["cache:source.current!=cached@Cache#0"]
    );
    for (entry, (fault, fixture, element, count)) in valid.kata_entries()[1..].iter().zip(POSITIVES)
    {
        let kata = entry.content.kata.as_ref().expect("kata");
        assert_eq!(kata.kata_id, "negative-control-bundle");
        assert_eq!(kata.verdict, "killed", "{fault} stays a null/parity result");
        assert!(kata.comparison.unique_detections.is_empty());
        assert_eq!(kata.expected[0].fault, fault);
        assert!(kata.catalog_hash.is_none(), "{fault} has no catalog");
        assert_eq!(kata.run.findings.len(), count, "{fault} finding count");
        assert_eq!(
            kata.fixture,
            format!("fixtures/katas/negative-control-bundle/{fixture}")
        );
        let baseline = kata.comparison.baseline.as_ref().expect("executed arm");
        assert_eq!(baseline.mode, BaselineMode::Executed);
        assert_eq!(baseline.detections, vec![element.to_string()]);
        assert_eq!(entry.content.model_hashes, vec![kata.fixture_hash.clone()]);
        assert_eq!(kata.fixture_hash, fixture_hash(fixture));
        assert!(
            !kata.expressibility_gap.is_empty(),
            "{fault} records the lens gap"
        );
        assert!(
            kata.discrepancies
                .iter()
                .any(|line| line.contains("Honesty purpose")),
            "{fault} records the honesty purpose"
        );
    }
}
