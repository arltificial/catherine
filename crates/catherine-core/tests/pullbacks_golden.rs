//! Purpose: the pullback golden fixture — the checked shape of
//! `tests/golden/violation_pullback.json` (format 1.6, openspec change
//! `add-pullback-checks`), pinned structurally rather than by output
//! text.
//!
//! Responsibilities: parse the golden document, build and check it, and
//! assert exactly one executed coverage violation with its exact witness
//! JSON and a lossless serde round-trip.
//!
//! Rationale: the pullback half of the golden corpus is its own binary so
//! the scenario suite (`pullbacks_checks`) stays under the per-file line
//! cap; the fixture still loads through the kernel's one pipeline, so the
//! golden pins the same parse → build → check path the CLI runs.

use catherine_core::checks::check;
use catherine_core::findings::{
    EpistemicLabel, Finding, FindingKind, PullbackWitness, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Model, RowId};
use catherine_core::parse::parse;

/// The dense id of an object's `index`-th parsed row — never a guessed
/// name → id map. The lookup is drawn as one `(object, index)` group.
fn row(model: &Model, at: (&str, usize)) -> RowId {
    let (object, index) = at;
    model.instance.rows[&Ident(object.to_string())][index]
}

/// The executed coverage violation the fixture describes.
fn covers(pair: [RowId; 2]) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Covers {
            pullback: 0,
            pair,
        })),
    }
}

/// The finding's structural JSON must round-trip losslessly.
fn round_trips(finding: &Finding) {
    let text = serde_json::to_string(finding).expect("finding serializes");
    let back: Finding = serde_json::from_str(&text).expect("finding deserializes");
    assert_eq!(&back, finding, "serde round-trip is lossless");
}

/// Scenario "Every finding kind has a golden fixture": the pullback
/// fixture parses and checks to exactly one executed coverage violation,
/// with its structural JSON pinned and a lossless round-trip.
#[test]
fn pullback_witness_golden_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/violation_pullback.json"
    );
    let text = std::fs::read_to_string(path).expect("the golden fixture exists");
    let model = parse(&text).expect("the golden fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    let findings = check(&model.schema, &built);

    let pair = [row(&model, ("A", 1)), row(&model, ("B", 0))];
    assert_eq!(
        findings,
        vec![covers(pair)],
        "exactly one coverage violation, pair in canonical order"
    );

    let json = serde_json::to_value(&findings).expect("findings serialize");
    assert_eq!(json[0]["label"].as_str(), Some("executed"));
    assert_eq!(json[0]["kind"].as_str(), Some("violation"));
    assert_eq!(json[0]["witness"]["pullback"].as_u64(), Some(0));
    assert_eq!(
        json[0]["witness"]["pair"][0].as_u64(),
        Some(pair[0].0 as u64)
    );
    assert_eq!(
        json[0]["witness"]["pair"][1].as_u64(),
        Some(pair[1].0 as u64)
    );
    assert!(json[0]["witness"].get("row").is_none());
    round_trips(&findings[0]);
}
