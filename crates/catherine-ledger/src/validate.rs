//! Purpose: the ledger validator — the RED gate of tasks.md task 1: the
//! ledger refuses bad science. A kata entry without a prior negative
//! control, a verdict outside the closed vocabulary, an entry without
//! model-hash references — each is a named rejection, never a warning;
//! plus the content-address integrity checks (id recomputation, seq
//! order, supersession targets) that make the ledger append-only.
//!
//! Responsibilities: [`validate_dir`] reads a ledger directory through
//! small phase checks — per-entry integrity, control self-negation,
//! control-first ordering, baseline-arm claims, rollup placement/coverage
//! — and refuses the ledger when any phase reports. The Green shape is
//! control-first, contiguous seqs, sealed ids, and a trailing rollup.
//!
//! Rationale: ct-catlab's own failure mode was verdict/record
//! contradictions (REPORT methodology §1) — the validator exists so the
//! tool cannot quietly do that to itself. Rejections are total (any
//! violation fails the ledger): partial ledgers never pass. The
//! baseline-arm refusals ([`Violation::UniqueWithoutBaseline`],
//! [`Violation::CitedBaselineCannotSurvive`]) are wired: a uniqueness
//! claim needs an executed, committed baseline arm, and a cited-only arm
//! caps the verdict at inconclusive. Supersession is the correction path
//! (a new entry whose `supersedes` names the old id, never an edit), so
//! claim-bearing rules run on the EFFECTIVE entries while the integrity
//! pins (id, seq, model hashes, verdict vocabulary, supersession
//! targets) run on all of them.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::entry::{entry_id, BaselineMode, Entry, EntryContent};

/// The closed verdict vocabulary, as the stable spellings.
const VERDICTS: [&str; 4] = ["killed", "survived", "inconclusive", "invalid-run"];

/// One named rejection. The variants are the stable refusal reasons the
/// RED tests pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    /// The directory or a file could not be read/parsed.
    Unreadable(String),
    /// The entry id does not match its canonical content bytes.
    IdMismatch { file: String },
    /// Seq numbers are not 1..n in file order.
    SeqOrder { found: u32, expected: u32 },
    /// An entry validated no model documents.
    MissingModelHashes { seq: u32 },
    /// A verdict outside `killed|survived|inconclusive|invalid-run`.
    InvalidVerdict { seq: u32, verdict: String },
    /// A kata entry with no negative control at a smaller seq.
    NegativeControlMissing { seq: u32 },
    /// The effective control entry did not self-negate.
    ControlDidNotSelfNegate { seq: u32, reason: String },
    /// A `supersedes` target that does not exist or is not earlier.
    BadSupersession { seq: u32, target: String },
    /// The rollup is absent, is not last, or precedes kata entries.
    RollupMisplaced { seq: u32, reason: String },
    /// The rollup's `kata_entry_ids` are not exactly the effective kata
    /// entry ids.
    RollupIncomplete { seq: u32 },
    /// Non-empty `unique_detections` with no baseline arm naming a
    /// committed artifact.
    UniqueWithoutBaseline { seq: u32 },
    /// A `survived` verdict whose baseline arm is cited-only.
    CitedBaselineCannotSurvive { seq: u32 },
}

/// A ledger that passed validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidLedger {
    pub entries: Vec<Entry>,
}

impl ValidLedger {
    /// The effective entries, in seq order: superseded entries dropped.
    pub fn effective_entries(&self) -> Vec<&Entry> {
        effective_entries(&self.entries)
    }

    /// The effective kata entries, in seq order: a superseded kata
    /// record is a correction, not a second claim.
    pub fn kata_entries(&self) -> Vec<&Entry> {
        self.effective_entries()
            .into_iter()
            .filter(|e| e.content.kind == "kata")
            .collect()
    }

    /// The effective (last) rollup entry, when present.
    pub fn rollup(&self) -> Option<&Entry> {
        self.effective_entries()
            .into_iter()
            .filter(|e| e.content.kind == "rollup")
            .last()
    }
}

/// Read one ledger file's entry: the content must deserialize and its
/// recomputed id must equal the file's sealed id. A file whose content
/// cannot be recognized under the sealed id IS an id mismatch — the
/// seal is the integrity contract, not a formality.
pub fn read_entry_file(path: &Path) -> Result<Entry, Violation> {
    let file = path.display().to_string();
    let bytes =
        fs::read_to_string(path).map_err(|e| Violation::Unreadable(format!("{file}: {e}")))?;
    let value: serde_json::Value =
        serde_json::from_str(&bytes).map_err(|e| Violation::Unreadable(format!("{file}: {e}")))?;
    let content: EntryContent = serde_json::from_value(value.clone())
        .map_err(|_| Violation::IdMismatch { file: file.clone() })?;
    let entry: Entry =
        serde_json::from_value(value).map_err(|_| Violation::IdMismatch { file: file.clone() })?;
    let recomputed =
        entry_id(&content).map_err(|e| Violation::Unreadable(format!("{file}: {e}")))?;
    if entry.id != recomputed {
        return Err(Violation::IdMismatch { file });
    }
    Ok(entry)
}

/// List a directory's `*.json` files, sorted (the ledger's file order).
pub fn ledger_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, Violation> {
    let mut files: Vec<std::path::PathBuf> = fs::read_dir(dir)
        .map_err(|e| Violation::Unreadable(format!("{}: {e}", dir.display())))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    Ok(files)
}

/// The entry's verdict spelling, when it declares one (kata verdict or
/// rollup verdict).
fn verdict_spelling(entry: &Entry) -> Option<&str> {
    if let Some(kata) = &entry.content.kata {
        return Some(kata.verdict.as_str());
    }
    entry
        .content
        .rollup
        .as_ref()
        .map(|rollup| rollup.verdict.as_str())
}

/// Whether the entry is a negative-control kata entry.
fn is_control(entry: &Entry) -> bool {
    entry.content.kata.as_ref().is_some_and(|kata| kata.control)
}

/// The effective entries: drop any entry whose id appears in a later
/// entry's `supersedes`. Supersession is the ledger's correction path —
/// a corrected claim is a new entry, never an edit — so claim-bearing
/// rules evaluate only the last word on each claim. Integrity rules
/// still run over every entry, superseded ones included.
fn effective_entries(entries: &[Entry]) -> Vec<&Entry> {
    entries
        .iter()
        .enumerate()
        .filter(|(index, entry)| {
            !entries[*index + 1..]
                .iter()
                .any(|later| later.content.supersedes.as_deref() == Some(entry.id.as_str()))
        })
        .map(|(_, entry)| entry)
        .collect()
}

/// Per-entry integrity pins: contiguous seq from 1, model hashes
/// present, closed verdict vocabulary, supersession targets real and
/// earlier.
fn integrity_violations(entries: &[Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let expected = (index + 1) as u32;
        if entry.content.seq != expected {
            violations.push(Violation::SeqOrder {
                found: entry.content.seq,
                expected,
            });
        }
        if entry.content.model_hashes.is_empty() {
            violations.push(Violation::MissingModelHashes {
                seq: entry.content.seq,
            });
        }
        if let Some(verdict) = verdict_spelling(entry) {
            if !VERDICTS.contains(&verdict) {
                violations.push(Violation::InvalidVerdict {
                    seq: entry.content.seq,
                    verdict: verdict.to_string(),
                });
            }
        }
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
    }
    violations
}

/// The control did not self-negate: an effective control entry whose
/// verdict is not `killed`. A corrected control is appended as a
/// superseding entry, so the old one is not effective and never reaches
/// this check — there is no separate correction branch to reconcile.
fn control_self_negation_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        if !is_control(entry) {
            continue;
        }
        let verdict = entry
            .content
            .kata
            .as_ref()
            .map(|kata| kata.verdict.as_str())
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
fn control_first_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        let Some(kata) = &entry.content.kata else {
            continue;
        };
        if kata.control {
            continue;
        }
        let controlled = entries.iter().any(|earlier| {
            earlier.content.seq < entry.content.seq
                && is_control(earlier)
                && earlier
                    .content
                    .kata
                    .as_ref()
                    .is_some_and(|kata| kata.verdict.as_str() == "killed")
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
fn baseline_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for entry in entries {
        let Some(kata) = &entry.content.kata else {
            continue;
        };
        if kata.comparison.unique_detections.is_empty() {
            continue;
        }
        let seq = entry.content.seq;
        let Some(baseline) = &kata.comparison.baseline else {
            violations.push(Violation::UniqueWithoutBaseline { seq });
            continue;
        };
        if baseline.artifact.trim().is_empty() {
            violations.push(Violation::UniqueWithoutBaseline { seq });
            continue;
        }
        if baseline.mode == BaselineMode::Cited && kata.verdict == "survived" {
            violations.push(Violation::CitedBaselineCannotSurvive { seq });
        }
    }
    violations
}

/// The effective rollup, when present: last entry, covering exactly the
/// effective kata entries (set equality between `kata_entry_ids` and
/// the effective kata ids). A ledger WITHOUT a rollup is a valid
/// append-only intermediate state — entries land before the rollup is
/// derived. A stale rollup that was not superseded is misplaced.
fn rollup_violations(entries: &[&Entry]) -> Vec<Violation> {
    let mut violations = Vec::new();
    let rollups: Vec<&Entry> = entries
        .iter()
        .copied()
        .filter(|entry| entry.content.kind == "rollup")
        .collect();
    let Some(rollup) = rollups.last().copied() else {
        return violations;
    };
    let last_seq = entries.last().map(|last| last.content.seq);
    for candidate in &rollups {
        if last_seq != Some(candidate.content.seq) {
            violations.push(Violation::RollupMisplaced {
                seq: candidate.content.seq,
                reason: "the rollup must be the last entry".into(),
            });
        }
    }
    let recorded: BTreeSet<&str> = rollup
        .content
        .rollup
        .as_ref()
        .map(|record| record.kata_entry_ids.iter().map(String::as_str).collect())
        .unwrap_or_default();
    let actual: BTreeSet<&str> = entries
        .iter()
        .filter(|entry| entry.content.kind == "kata")
        .map(|entry| entry.id.as_str())
        .collect();
    if recorded != actual {
        violations.push(Violation::RollupIncomplete {
            seq: rollup.content.seq,
        });
    }
    violations
}

/// Validate the ledger directory through the phase checks: file-level
/// integrity, per-entry pins, control self-negation, control-first
/// ordering, baseline-arm claims, and rollup placement/coverage. The
/// integrity phases see every entry; the claim-bearing phases see only
/// the effective entries (superseded ones are corrections, not claims).
pub fn validate_dir(dir: &Path) -> Result<ValidLedger, Vec<Violation>> {
    let files = match ledger_files(dir) {
        Ok(files) => files,
        Err(violation) => return Err(vec![violation]),
    };
    let read: Vec<Result<Entry, Violation>> =
        files.iter().map(|path| read_entry_file(path)).collect();
    let mut violations: Vec<Violation> = read
        .iter()
        .filter_map(|result| result.as_ref().err().cloned())
        .collect();
    let entries: Vec<Entry> = read.iter().flatten().cloned().collect();
    violations.extend(integrity_violations(&entries));
    let effective = effective_entries(&entries);
    violations.extend(baseline_violations(&effective));
    violations.extend(control_self_negation_violations(&effective));
    violations.extend(control_first_violations(&effective));
    violations.extend(rollup_violations(&effective));
    if violations.is_empty() {
        Ok(ValidLedger { entries })
    } else {
        Err(violations)
    }
}
