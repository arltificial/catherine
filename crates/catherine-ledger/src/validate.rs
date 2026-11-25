//! Purpose: the ledger validator — the RED gate of tasks.md task 1: the
//! ledger refuses bad science. A kata entry without a prior negative
//! control, a verdict outside the closed vocabulary, an entry without
//! model-hash references — each is a named rejection, never a warning;
//! plus the content-address integrity checks that keep it append-only.
//!
//! Responsibilities: [`validate_dir`] reads a ledger directory through
//! phase checks — per-entry integrity, control self-negation,
//! control-first ordering, baseline-arm claims, catalog hashes, rollup
//! placement/coverage — and refuses the ledger when any phase reports.
//! The integrity phase asks [`crate::decision`] about decision entries.
//! The Green shape is control-first, contiguous seqs, sealed ids, and a
//! trailing rollup (terminal decisions may follow it).
//!
//! Rationale: ct-catlab's own failure mode was verdict/record
//! contradictions (REPORT methodology §1) — the validator exists so the
//! tool cannot quietly do that to itself, and rejections are total. A
//! uniqueness claim needs an executed baseline arm
//! ([`Violation::UniqueWithoutBaseline`]) and a cited-only arm caps the
//! verdict ([`Violation::CitedBaselineCannotSurvive`]); a probe entry's
//! `catalog_hash` must match the pre-registered catalog
//! ([`Violation::CatalogHashMismatch`]). Supersession is the correction
//! path (a new entry whose `supersedes` names the old id, never an
//! edit), so claim-bearing rules run on the EFFECTIVE entries while the
//! integrity pins run on all of them.

use std::fs;
use std::path::Path;

use crate::entry::{entry_id, Entry};

mod rules;

use rules::{
    baseline_violations, catalog_violations, control_first_violations,
    control_self_negation_violations, integrity_violations, rollup_violations,
};

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
    /// The entry's `catalog_hash` does not match the committed catalog
    /// for its id, or names an unregistered probe.
    CatalogHashMismatch { seq: u32 },
    /// A decision outcome outside `extend|hold|retire`.
    InvalidDecisionOutcome { seq: u32, outcome: String },
    /// The `decision` kind and its payload disagree: kind `decision`
    /// without a record, or a record on a non-decision kind.
    MalformedDecision { seq: u32, reason: String },
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
            .rfind(|e| e.content.kind == "rollup")
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
    let entry: Entry =
        serde_json::from_value(value).map_err(|_| Violation::IdMismatch { file: file.clone() })?;
    let recomputed =
        entry_id(&entry.content).map_err(|e| Violation::Unreadable(format!("{file}: {e}")))?;
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
        return Some(kata.derivation.verdict.as_str());
    }
    entry.content.rollup.as_ref().map(|r| r.verdict.as_str())
}

/// Whether the entry is a negative-control kata entry.
fn is_control(entry: &Entry) -> bool {
    entry
        .content
        .kata
        .as_ref()
        .is_some_and(|kata| kata.identity.control)
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

/// Validate the ledger directory through the phase checks: file-level
/// integrity, per-entry pins (including decision kind/payload agreement
/// and the closed `extend | hold | retire` outcome vocabulary),
/// control self-negation, control-first ordering, baseline-arm claims,
/// catalog hashes, and rollup placement/coverage. The integrity phases
/// see every entry; the claim-bearing phases see only the effective
/// entries (superseded ones are corrections, not claims).
pub fn validate_dir(dir: &Path) -> Result<ValidLedger, Vec<Violation>> {
    let files = match ledger_files(dir) {
        Ok(files) => files,
        Err(violation) => return Err(vec![violation]),
    };
    let read: Vec<_> = files.iter().map(|path| read_entry_file(path)).collect();
    let mut violations: Vec<Violation> = read
        .iter()
        .filter_map(|result| result.as_ref().err().cloned())
        .collect();
    let entries: Vec<Entry> = read.iter().flatten().cloned().collect();
    violations.extend(integrity_violations(&entries));
    let effective = effective_entries(&entries);
    violations.extend(baseline_violations(&effective));
    violations.extend(catalog_violations(&effective));
    violations.extend(control_self_negation_violations(&effective));
    violations.extend(control_first_violations(&effective));
    violations.extend(rollup_violations(&effective));
    if violations.is_empty() {
        Ok(ValidLedger { entries })
    } else {
        Err(violations)
    }
}
