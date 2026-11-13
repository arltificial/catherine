//! Purpose: the recorded evidence of kata `cor-strategy-rules` (beads
//! `catherine-lgl.7`) — a legacy Chain-of-Responsibility + Strategy
//! dispatcher modelled as an authored rule table, compared against a
//! frozen scripted dispatch sample. The committed kata ledger validates,
//! every fixture's canonical model hash is pinned, and the near-floor
//! control self-negated while the above-floor entry carries the executed
//! baseline arm and the three structural unique detections.
//!
//! Responsibilities: pin each fixture's model hash (a silent fixture edit
//! breaks the entry's content address), validate the ledger directory
//! through the real validator, and check the control-first discipline,
//! the executed baseline artifact's script hash, and the model-hash set.
//!
//! Rationale: the entry is evidence only while the files on disk still
//! hash to what it cites, and while the unique set stays arithmetic
//! (lens findings minus the scripted sample's detections) rather than a
//! claim. The scope caveat lives in the kata README and the entry's
//! `expressibility_gap`/`discrepancies`: authored rule tables are a
//! commodity, so the result is reverse-engineering transfer, not a
//! detection-rate win.

use std::path::Path;

use catherine_core::hash::{model_hash, sha256_digest};
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

const BASELINE: &str = "prototype/cor_strategy_rules_baseline.py";

/// The above-floor table, its seven injected faults, and the near-floor
/// control with its three faults — every document the ledger cites.
const FIXTURES: [(&str, &str); 12] = [
    (
        "fixtures/katas/cor-strategy-rules/dispatch.ct.json",
        "sha256:a982cb510fa444ca2803c77b5023e8cea2993ff305612cab6ce31a2620282be2",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/01-chain-gap-remove-fallback.ct.json",
        "sha256:fc5da4f7302c55bf8bcf93c065de934e2355de78961c75b6273a8a689fa6a611",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/02-chain-gap-fallback-narrow.ct.json",
        "sha256:d716eb7b2ee7387bee020040ff8faedea84fd0a39c48d5ebe559ecd6d9e0e74e",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/03-shadow-duplicate-payout.ct.json",
        "sha256:09f895d1a3c5feb4a324c87923979ab530885249f3d4a97b5addce0112dc5343",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/04-shadow-widen-payout.ct.json",
        "sha256:4a3ba64d09ced9fa7fb0732901c4188c01c08cd3064335ce89fab04a7e70423b",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/05-derivable-strategy-branch.ct.json",
        "sha256:6be6a0d74a49dd68de65dc45daa6f8fb2f60c543caa91ed3a86632d6017b476d",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/06-overlap-declared-disjoint.ct.json",
        "sha256:f3aede4a30ac2fdf3d47f94f1464d2458601021fcbdf5d53fb6cf68ce74f7a2e",
    ),
    (
        "fixtures/katas/cor-strategy-rules/faults/07-strategy-selection-mismatch.ct.json",
        "sha256:2afb2d11927e396c940d13a4ac390a3a8877ae991c9a0eeeabe56e640e114bb1",
    ),
    (
        "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json",
        "sha256:86732d37536b0f277b6c2779f2020d58cc7ff7dbb362cd32860a861872589c20",
    ),
    (
        "fixtures/katas/cor-strategy-rules/control/faults/01-gap-remove-fallback.ct.json",
        "sha256:d07a928d779e3cfb6334d898f18483bcb6e196674260372e090f8b7e31c486c3",
    ),
    (
        "fixtures/katas/cor-strategy-rules/control/faults/02-overlap-total-fallback.ct.json",
        "sha256:df3995c60fa84dd6553b656187ed3f5c475f0fd775ea084909416d11e48c6e6a",
    ),
    (
        "fixtures/katas/cor-strategy-rules/control/faults/03-strategy-selection-mismatch.ct.json",
        "sha256:dbf8a832cd8ab1b4d7f4eb75fbdfbb3885200ab6b31f795706a5b6f71488ff96",
    ),
];

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// Every kata fixture is pinned: an edit must become a superseding ledger
/// entry, never a silent change.
#[test]
fn cor_strategy_fixture_hashes_are_pinned() {
    let mut mismatches = Vec::new();
    for (path, pin) in FIXTURES {
        let got = fixture_hash(path);
        println!("HASH {path} {got}");
        if got != pin {
            mismatches.push(format!("{path}: {got} != {pin}"));
        }
    }
    assert!(mismatches.is_empty(), "fixture hash drift: {mismatches:?}");
}

/// The kata ledger validates with the control first, and the above-floor
/// entry carries the executed baseline arm, the three unique structural
/// detections and the model hashes of every document it names.
#[test]
fn cor_strategy_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/katas/cor-strategy-rules");
    let valid = validate_dir(&dir).expect("the CoR kata ledger validates");
    assert_eq!(valid.entries.len(), 2, "control + above-floor kata");

    let control = valid.kata_entries()[0].content.kata.clone().expect("kata");
    assert!(control.control, "the near-floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    assert_eq!(
        control.fixture,
        "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json"
    );

    let above = valid.kata_entries()[1].content.kata.clone().expect("kata");
    assert_eq!(above.kata_id, "cor-strategy-rules");
    assert_eq!(
        above.fixture,
        "fixtures/katas/cor-strategy-rules/dispatch.ct.json"
    );
    assert_eq!(above.verdict, "survived");
    assert!(
        above.catalog_hash.is_none(),
        "a kata, not a registered probe"
    );
    assert_eq!(
        above.comparison.unique_detections.len(),
        3,
        "an unsampled guard gap, a duplicate handler and a derivable branch"
    );
    let baseline = above
        .comparison
        .baseline
        .as_ref()
        .expect("an executed baseline arm carries the unique set");
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert!(
        !baseline.detections.is_empty(),
        "the scripted dispatch sample fired on the same faults"
    );
    let script_hash = sha256_digest(&std::fs::read(format!("{ROOT}/{BASELINE}")).expect("script"));
    assert!(
        baseline
            .artifact
            .contains(&format!("{BASELINE} {script_hash}")),
        "the arm names the committed script and its current sha256: {}",
        baseline.artifact
    );
    let listed: Vec<String> = FIXTURES
        .iter()
        .map(|(path, _)| fixture_hash(path))
        .collect();
    assert_eq!(
        valid.kata_entries()[1].content.model_hashes,
        listed[..8].to_vec(),
        "the above-floor run validated the table and its seven faults"
    );
    assert_eq!(
        valid.kata_entries()[0].content.model_hashes,
        listed[8..].to_vec(),
        "the control validated the floor chain and its three faults"
    );
    assert_eq!(above.fixture_hash, listed[0], "the table the entry names");
}
