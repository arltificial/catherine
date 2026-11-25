//! The validator's per-entry and phase rules: integrity pins,
//! supersession, control self-negation and ordering, baseline-arm
//! claims, catalog hashes, and rollup placement/coverage. Split out of
//! the validator module verbatim (mechanical size refactor); bodies are
//! unchanged and all types come from `super`.

use std::collections::BTreeSet;

use super::{is_control, verdict_spelling, Violation, VERDICTS};
use crate::catalog::{is_probe, probe_catalog_hash};
use crate::entry::{BaselineMode, Entry};

/// Per-entry integrity pins: contiguous seq from 1, model hashes
/// present, closed verdict vocabulary, supersession targets real and
/// earlier.
pub(super) fn integrity_violations(entries: &[Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        violations.extend(pin_violations(entry, (index + 1) as u32));
        // Decisions: kind, payload, and outcome vocabulary — the rules
        // live with the schema (`crate::decision`).
        violations.extend(crate::decision::decision_violations(entry));
        violations.extend(supersession_violations(entries, entry));
    }
    violations
}

/// The positional pins: contiguous seq, model hashes present, verdict
/// spelling in the closed vocabulary.
pub(super) fn pin_violations(entry: &Entry, expected: u32) -> Vec<Violation> {
    let seq_order = (entry.content.seq != expected).then_some(Violation::SeqOrder {
        found: entry.content.seq,
        expected,
    });
    let model_hashes =
        entry
            .content
            .model_hashes
            .is_empty()
            .then_some(Violation::MissingModelHashes {
                seq: entry.content.seq,
            });
    let verdict = verdict_spelling(entry)
        .filter(|v| !VERDICTS.contains(v))
        .map(|verdict| Violation::InvalidVerdict {
            seq: entry.content.seq,
            verdict: verdict.to_string(),
        });
    let pins = [seq_order, model_hashes, verdict];
    pins.into_iter().flatten().collect()
}

/// A supersession must target a real, earlier entry; a correction
/// points forward in the append-only ledger, never at a missing or
/// later record.
pub(super) fn supersession_violations(entries: &[Entry], entry: &Entry) -> Vec<Violation> {
    let mut violations = Vec::new();
    if let Some(target) = &entry.content.supersedes {
        let exists = entries
            .iter()
            .any(|other| other.id == *target && other.content.seq < entry.content.seq);
        if !exists {
            violations.push(Violation::BadSupersession {
                seq: entry.content.seq,
                target: target.clone(),
            });
        }
    }
    violations
}

/// The control did not self-negate: an effective control entry whose
/// verdict is not `killed`. A corrected control is appended as a
/// superseding entry, so the old one is not effective and never reaches
/// this check — there is no separate correction branch to reconcile.
pub(super) fn control_self_negation_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        if !is_control(entry) {
            continue;
        }
        let verdict = entry
            .content
            .kata
            .as_ref()
            .map(|kata| kata.derivation.verdict.as_str())
            .unwrap_or("");
        if verdict == "killed" {
            continue;
        }
        violations.push(Violation::ControlDidNotSelfNegate {
            seq: entry.content.seq,
            reason: format!("control verdict `{verdict}`, expected killed"),
        });
    }
    violations
}

/// Negative control first: every effective positive kata needs an
/// earlier effective self-negated control.
pub(super) fn control_first_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        let Some(kata) = &entry.content.kata else {
            continue;
        };
        if kata.identity.control {
            continue;
        }
        let controlled = entries.iter().any(|earlier| {
            earlier.content.seq < entry.content.seq
                && earlier.content.kata.as_ref().is_some_and(|kata| {
                    kata.identity.control && kata.derivation.verdict == "killed"
                })
        });
        if !controlled {
            violations.push(Violation::NegativeControlMissing {
                seq: entry.content.seq,
            });
        }
    }
    violations
}

/// The baseline-arm refusals, over effective kata records with a
/// non-empty `unique_detections`: the claim is arithmetic (probe
/// detections minus baseline detections), so it needs an arm naming a
/// committed artifact, and a cited-only arm cannot support `survived`.
/// An empty unique set is the parity ceiling and needs no arm.
pub(super) fn baseline_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        let Some(kata) = &entry.content.kata else {
            continue;
        };
        if kata.derivation.comparison.unique_detections.is_empty() {
            continue;
        }
        let seq = entry.content.seq;
        let Some(baseline) = &kata.derivation.comparison.baseline else {
            violations.push(Violation::UniqueWithoutBaseline { seq });
            continue;
        };
        if baseline.artifact.trim().is_empty() {
            violations.push(Violation::UniqueWithoutBaseline { seq });
            continue;
        }
        if baseline.mode == BaselineMode::Cited && kata.derivation.verdict == "survived" {
            violations.push(Violation::CitedBaselineCannotSurvive { seq });
        }
    }
    violations
}

/// The catalog-hash rule, over effective kata-shaped entries: a
/// declared `catalog_hash` must be the committed catalog's hash for
/// the entry's id (an unregistered probe has no catalog to match), and
/// a registered probe with expectations must carry the pre-registration
/// it ran against.
pub(super) fn catalog_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        let Some(kata) = &entry.content.kata else {
            continue;
        };
        let seq = entry.content.seq;
        if let Some(hash) = &kata.catalog_hash {
            if probe_catalog_hash(&kata.identity.kata_id).as_deref() != Some(hash.as_str()) {
                violations.push(Violation::CatalogHashMismatch { seq });
            }
        } else if is_probe(&kata.identity.kata_id) && !kata.expected.is_empty() {
            violations.push(Violation::CatalogHashMismatch { seq });
        }
    }
    violations
}

/// The effective rollup, when present: last non-decision entry,
/// covering exactly the effective kata entries (set equality between
/// `kata_entry_ids` and the effective kata ids). A ledger WITHOUT a
/// rollup is a valid append-only intermediate state — entries land
/// before the rollup is derived. A stale rollup that was not superseded
/// is misplaced. Terminal decision entries may follow the rollup without
/// displacing it (design Decision 5).
pub(super) fn rollup_violations(entries: &[&Entry]) -> Vec<Violation> {
    let rollups: Vec<&Entry> = entries
        .iter()
        .copied()
        .filter(|entry| entry.content.kind == "rollup")
        .collect();
    let Some(rollup) = rollups.last().copied() else {
        return Vec::new();
    };
    misplaced_violations(entries, &rollups)
        .into_iter()
        .chain(coverage_violation(entries, rollup))
        .collect()
}

/// The rollup is the last entry: decisions are terminal (appended after
/// the rollup), so they do not displace it — the rollup must be the
/// last non-decision entry.
pub(super) fn misplaced_violations(entries: &[&Entry], rollups: &[&Entry]) -> Vec<Violation> {
    let last_seq = entries
        .iter()
        .rev()
        .find(|entry| entry.content.kind != "decision")
        .map(|entry| entry.content.seq);
    rollups
        .iter()
        .filter(|candidate| last_seq != Some(candidate.content.seq))
        .map(|candidate| Violation::RollupMisplaced {
            seq: candidate.content.seq,
            reason: "the rollup must be the last entry".into(),
        })
        .collect()
}

/// Coverage: set equality between the rollup's `kata_entry_ids` and the
/// effective kata ids.
pub(super) fn coverage_violation(entries: &[&Entry], rollup: &Entry) -> Option<Violation> {
    let recorded: BTreeSet<&str> = rollup
        .content
        .rollup
        .iter()
        .flat_map(|record| record.kata_entry_ids.iter().map(String::as_str))
        .collect();
    let actual: BTreeSet<&str> = entries
        .iter()
        .filter(|entry| entry.content.kind == "kata")
        .map(|entry| entry.id.as_str())
        .collect();
    (recorded != actual).then_some(Violation::RollupIncomplete {
        seq: rollup.content.seq,
    })
}
