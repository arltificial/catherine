//! Purpose: the proof-phase stop-gate decision on the ledger (openspec
//! change `add-lean-kernel-model`, design Decision 5; spec requirement
//! "The stop gate records the assurance outcome") — a content-addressed
//! `decision` entry records `extend | hold | retire` with the
//! cost/benefit accounting, without changing any pre-existing entry id.
//!
//! Rationale: a decision is a terminal record like a rollup, so it is
//! content-addressed and validated by the same crate. Its outcome is a
//! closed vocabulary (like the kata verdict); its content field is
//! skipped when absent, so the schema extension cannot rewrite the
//! canonical bytes — or the ids — of the committed kata and rollup
//! ledgers.

use std::path::{Path, PathBuf};

use catherine_ledger::entry::{
    seal, DecisionRecord, EntryContent, Gates, KataVerdict, RollupRecord,
};
use catherine_ledger::rollup::PROCEDURE;
use catherine_ledger::validate::{validate_dir, Violation};

mod support;

use support::{green_contents, present, TempLedger, TestError};

/// The workspace root, from this test crate's manifest directory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One stop-gate decision record — the accounting the spec requires the
/// gate to write down.
fn decision_record(outcome: &str) -> DecisionRecord {
    DecisionRecord {
        outcome: outcome.into(),
        scope: "coproduct pair semantics; landed phase-2 scope".into(),
        rationale: "the proofs closed the route-exhaustiveness route without changing the kernel \
             surface, so the model is maintained at landed scope"
            .into(),
        proofs: vec![
            "coproduct_route_exhaustive".into(),
            "coproduct_injection_disjoint".into(),
        ],
        defects_found: vec![
            "the `voucher` row of `Method` was accepted with no injection covering it".into(),
        ],
        holes_found: vec![
            "pair-semantics injectivity is not stated by the path-equation surface".into(),
        ],
        fixture_only_comparison:
            "the fixture corpus pinned the accepted/rejected coproduct shapes operationally; \
             the proofs additionally pinned route exhaustiveness and injection disjointness \
             for every accepted row, which the corpus only sampled"
                .into(),
    }
}

/// A decision entry: kind `decision`, the record present, model hashes
/// non-empty, no kata or rollup payload.
fn decision(seq: u32, outcome: &str) -> EntryContent {
    EntryContent {
        seq,
        kind: "decision".into(),
        recorded: "2026-11-17".into(),
        kata: None,
        rollup: None,
        model_hashes: vec!["sha256:lean-coproduct-pair".into()],
        supersedes: None,
        decision: Some(decision_record(outcome)),
    }
}

/// A minimal rollup over the given sealed kata entries (control first).
fn rollup(seq: u32, katas: &[&catherine_ledger::entry::Entry]) -> Result<EntryContent, TestError> {
    Ok(EntryContent {
        seq,
        kind: "rollup".into(),
        recorded: "2026-11-17".into(),
        kata: None,
        rollup: Some(RollupRecord {
            procedure: PROCEDURE.iter().map(|s| s.to_string()).collect(),
            kata_entry_ids: katas.iter().map(|entry| entry.id.clone()).collect(),
            per_kata: katas
                .iter()
                .map(|entry| {
                    let kata = present(entry.content.kata.as_ref(), "kata record")?;
                    Ok(KataVerdict {
                        kata_id: kata.identity.kata_id.clone(),
                        entry_id: entry.id.clone(),
                        verdict: kata.derivation.verdict.clone(),
                    })
                })
                .collect::<Result<Vec<_>, TestError>>()?,
            control_gate: "passed".into(),
            gates: Gates {
                p7_catherine_alr: "blocked".into(),
                p8_catherine_nq5: "blocked".into(),
                note: "the bead flip is orchestrator-reserved".into(),
            },
            verdict: "survived".into(),
        }),
        model_hashes: vec!["sha256:rollup".into()],
        supersedes: None,
        decision: None,
    })
}

/// The `Err` arm of a validation expected to fail: the named refusal, or
/// a test failure naming the case that validated instead.
fn refusals(
    case: &str,
    validated: Result<catherine_ledger::validate::ValidLedger, Vec<Violation>>,
) -> Result<Vec<Violation>, TestError> {
    match validated {
        Ok(_) => Err(case.to_string().into()),
        Err(violations) => Ok(violations),
    }
}

/// A decision entry round-trips through the canonical bytes and validates
/// under the same content-address rules as the kata and rollup entries.
#[test]
fn decision_entry_round_trips_and_validates() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("round-trip")?;
    ledger.write(&[decision(1, "extend")])?;

    let valid = validate_dir(&ledger.dir)
        .map_err(|v| format!("a stop-gate decision is a valid ledger: {v:?}"))?;
    assert_eq!(valid.entries.len(), 1);
    let entry = &valid.entries[0];
    assert_eq!(entry.content.kind, "decision");
    assert!(!entry.content.model_hashes.is_empty());
    assert!(
        entry.content.kata.is_none(),
        "decision carries no kata record"
    );
    assert!(entry.content.rollup.is_none(), "decision carries no rollup");

    let record = present(entry.content.decision.as_ref(), "the decision record")?;
    assert_eq!(record.outcome, "extend");
    assert_eq!(record.proofs.len(), 2, "the theorem names are recorded");
    assert_eq!(
        (record.defects_found.len(), record.holes_found.len()),
        (1, 1),
        "defects and holes are recorded separately"
    );
    assert!(
        record.fixture_only_comparison.contains("fixture corpus"),
        "the fixture-only baseline is part of the accounting"
    );

    // Round trip: the file's sealed content deserializes to the same
    // content and re-seals to the same id.
    let files = catherine_ledger::validate::ledger_files(&ledger.dir)
        .map_err(|v| format!("ledger files listed: {v:?}"))?;
    let read = catherine_ledger::validate::read_entry_file(&files[0])
        .map_err(|v| format!("entry read back: {v:?}"))?;
    assert_eq!(read.content, entry.content);
    assert_eq!(read.id, entry.id);
    Ok(())
}

/// The outcome vocabulary is closed: extend | hold | retire, and nothing
/// else. An unknown outcome is a named rejection.
#[test]
fn outcome_outside_the_closed_vocabulary_is_named() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("bad-outcome")?;
    ledger.write(&[decision(1, "escalate")])?;

    let violations = refusals("an unknown outcome is refused", validate_dir(&ledger.dir))?;
    assert!(
        violations.contains(&Violation::InvalidDecisionOutcome {
            seq: 1,
            outcome: "escalate".into(),
        }),
        "the violation names the seq and the offending outcome: {violations:?}"
    );
    Ok(())
}

/// The existing model-hash rule applies unchanged to decision entries.
#[test]
fn decision_without_model_hashes_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("no-model-hashes")?;
    let mut content = decision(1, "hold");
    content.model_hashes.clear();
    ledger.write(&[content])?;

    let violations = refusals(
        "a decision must cite model hashes",
        validate_dir(&ledger.dir),
    )?;
    assert!(
        violations.contains(&Violation::MissingModelHashes { seq: 1 }),
        "the existing model-hash rule still applies: {violations:?}"
    );
    Ok(())
}

/// Kind and payload must agree: `decision` without a record is malformed,
/// never a silently empty decision.
#[test]
fn decision_kind_without_record_is_malformed() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("missing-record")?;
    let mut content = decision(1, "retire");
    content.decision = None;
    ledger.write(&[content])?;

    let violations = refusals(
        "a decision kind needs its record",
        validate_dir(&ledger.dir),
    )?;
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, Violation::MalformedDecision { seq: 1, .. })),
        "the violation names the malformed decision: {violations:?}"
    );
    Ok(())
}

/// Kind and payload must agree the other way too: a decision record on a
/// non-decision kind is malformed.
#[test]
fn decision_record_on_a_non_decision_kind_is_malformed() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("wrong-kind")?;
    let mut content = decision(1, "hold");
    content.kind = "kata".into();
    ledger.write(&[content])?;

    let violations = refusals(
        "a decision record needs the decision kind",
        validate_dir(&ledger.dir),
    )?;
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, Violation::MalformedDecision { seq: 1, .. })),
        "the violation names the malformed decision: {violations:?}"
    );
    Ok(())
}

/// Decisions are terminal records after the rollup: appending one must not
/// displace the rollup or change its coverage over the kata entries.
#[test]
fn decision_after_the_rollup_keeps_the_rollup_in_place() -> Result<(), Box<dyn std::error::Error>> {
    let contents = green_contents();
    let sealed: Vec<_> = contents
        .iter()
        .cloned()
        .map(seal)
        .collect::<Result<_, _>>()?;
    let katas: Vec<&catherine_ledger::entry::Entry> = sealed.iter().collect();
    let mut ledger_contents = contents;
    ledger_contents.push(rollup(3, &katas)?);

    let ledger = TempLedger::new("after-rollup")?;
    ledger.write(&ledger_contents)?;
    validate_dir(&ledger.dir)
        .map_err(|v| format!("the kata ledger with its rollup validates: {v:?}"))?;

    ledger.write(&[decision(4, "hold")])?;
    let valid = validate_dir(&ledger.dir)
        .map_err(|v| format!("a terminal decision after the rollup does not displace it: {v:?}"))?;
    assert_eq!(valid.entries.len(), 4);
    assert_eq!(
        valid.rollup().map(|entry| entry.content.seq),
        Some(3),
        "the rollup is still the last claim-bearing entry"
    );
    assert_eq!(
        valid.kata_entries().len(),
        2,
        "the decision neither joins nor disturbs kata coverage"
    );
    Ok(())
}

/// Regression: the committed ledger's canonical bytes and ids are
/// untouched by the schema extension. These ids were captured from the
/// files before the `decision` field existed.
#[test]
fn existing_coproduct_entry_ids_are_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let valid = validate_dir(&workspace_root().join("ledger/probes/coproduct"))
        .map_err(|v| format!("the committed coproduct ledger still validates: {v:?}"))?;
    let ids: Vec<&str> = valid
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            "sha256:84847b2d5cab899b950fdbaee0901d0f35e8ced6f20db4c0f80d3bfbef0ced98",
            "sha256:384db90355b38f947cc5f5fcd3aa520b83fc9177ac54badbef5a7079d52a3bf2",
            "sha256:2709a9909f27a4a73baebb4413dd2e787f6e3bdaf5d3197b34e9ce865cb64842",
        ],
        "pre-existing entry ids are byte-identical"
    );
    for entry in &valid.entries {
        assert!(
            entry.content.decision.is_none(),
            "existing entries carry no decision payload"
        );
        let bytes = serde_json::to_string(&entry.content)?;
        assert!(
            !bytes.contains("\"decision\""),
            "the absent field is skipped, not serialized as null: {bytes}"
        );
    }
    Ok(())
}
