//! Purpose: the ledger entry shapes (openspec change
//! `add-kata-validation-ledger`, design Decision 4) — the data every
//! entry carries and its content address.
//!
//! Responsibilities: define [`EntryContent`] (the hashed content: kata
//! record or project rollup), [`Entry`] (content + computed `id`), the
//! closed [`Verdict`] vocabulary, and the id computation: canonical JSON
//! bytes (the kernel's normative writer) over the content, sha256 on
//! top — the P0 hash port. Entries are append-only; a correction is a
//! new entry whose `supersedes` names the old id, never an edit.
//!
//! Rationale: identity is the digest of the canonical form (design
//! Decision 4, via `catherine_core::hash`): equal contents hash equal
//! regardless of key order, so the validator can recompute every id
//! from the bytes on disk. `Verdict` is exactly the four words the
//! series ledger uses (`killed | survived | inconclusive |
//! invalid-run`) — the closed vocabulary of design Decision 3. The
//! epistemic fields (`expected`, `comparison`, `expressibility_gap`,
//! `discrepancies`) exist because gaps and proposal-vs-series conflicts
//! are ledger data, not footnotes (design Decision 5; brief NOTE).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use catherine_core::canonical::write_value;
use catherine_core::hash::sha256_digest;

/// The closed verdict vocabulary (design Decision 3) — exactly four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Hypothesis falsified for this kata (or, at project level: stop).
    Killed,
    /// Demonstrated advantage with executed evidence, scoped.
    Survived,
    /// Cannot decide; the documented cause gates a re-run.
    Inconclusive,
    /// Environment or tooling failure — no verdict, rerun as a new entry.
    InvalidRun,
}

impl Verdict {
    /// The stable spelling the ledger and the series vocabulary share.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Killed => "killed",
            Self::Survived => "survived",
            Self::Inconclusive => "inconclusive",
            Self::InvalidRun => "invalid-run",
        }
    }
}

impl std::str::FromStr for Verdict {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "killed" => Ok(Self::Killed),
            "survived" => Ok(Self::Survived),
            "inconclusive" => Ok(Self::Inconclusive),
            "invalid-run" => Ok(Self::InvalidRun),
            other => Err(format!("verdict `{other}` outside the closed vocabulary")),
        }
    }
}

/// One pre-registered expectation: a fault from the series fault matrix
/// and the class catherine can express it in — fixed before the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expectation {
    /// The series fault id (F1, F2, …) or `schema-forced` / `probe`.
    pub fault: String,
    /// What the series validated: `structural` or `behavioral`.
    pub series_class: String,
    /// How catherine expresses it: `finding` (path equations + totality),
    /// `construction` (load-time rejection probe), or `gap`.
    pub catherine_class: String,
    /// The mapping, stated once — never inferred after the fact.
    pub detail: String,
}

/// One executed construction-time probe (a load that must be rejected).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Probe {
    /// Probe name (e.g. `bounds-out-of-grid`).
    pub name: String,
    /// The expectation this probe witnesses.
    pub fault: String,
    /// The full probe document, embedded for reproducibility.
    pub document: Value,
    /// The observed kernel rejection (code + message), verbatim.
    pub observed: String,
}

/// The executed run: the replay commands, the kernel findings verbatim
/// (the CLI/wasm encoding, unreshaped), and the probes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunLog {
    /// Executed command lines (the CLI replay path).
    pub commands: Vec<String>,
    /// The kernel findings, serialized exactly as `check` emits them.
    pub findings: Vec<Value>,
    /// Executed construction-time probes and their observed rejections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub probes: Vec<Probe>,
}

/// One fired finding, cited by stable id and mapped to its expectation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fired {
    /// Stable citation id (`violation:eq0@row3`, `dangling:label@row2`).
    pub finding_id: String,
    /// The finding kind (`violation`, `dangling`, …).
    pub kind: String,
    /// The epistemic label (`executed`, `hypothesized`).
    pub label: String,
    /// The expectation (`fault`) this firing witnesses.
    pub expectation: String,
}

/// Execution mode of a baseline arm (open vocabulary until the GREEN task
/// closes it): `executed` = the artifact ran before the probe run; `cited` =
/// quoted from literature only and therefore cannot carry a uniqueness claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineMode {
    Executed,
    Cited,
}

/// The baseline arm (design Decision 3, `add-domain-probe-validation`): the
/// committed, version-pinned artifact run on the same fault set, so that
/// `unique_detections` is derived arithmetically (probe detections minus
/// baseline detections) instead of asserted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineArm {
    /// In-tree path + hash, or tool name + pinned revision.
    pub artifact: String,
    /// Whether the artifact was executed before the run.
    pub mode: BaselineMode,
    /// The baseline's detections on the same fault set.
    pub detections: Vec<String>,
}

/// The mechanical comparison the verdict derives from (the procedure is
/// recorded in [`KataRecord::procedure`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    /// The conventional baseline from the series report, quoted.
    pub conventional_baseline: String,
    /// Every fired finding, mapped to its pre-registered expectation.
    pub fired: Vec<Fired>,
    /// Findings catching faults the conventional baseline missed — must
    /// be empty for the negative control to self-negate.
    pub unique_detections: Vec<String>,
    /// The committed baseline arm the unique detections were derived
    /// against; absent = no baseline arm (the parity ceiling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineArm>,
}

/// One named expressibility gap (design Decision 5): a validated
/// invariant the path-equation + totality surface cannot say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
    /// The invariant, named — never papered over.
    pub invariant: String,
    /// Why the v1 kernel surface cannot express it.
    pub why: String,
}

/// The kata-level record (one per ported kata; the negative control is
/// `control: true` and must be the first kata entry in the ledger).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KataRecord {
    /// The series kata id (`kata-6`, …).
    pub kata_id: String,
    /// Human name, including the negative-control role when present.
    pub name: String,
    /// True only for the FizzBuzz negative control.
    pub control: bool,
    /// The series report this expectation is grounded in.
    pub series_report: String,
    /// The series verdict, quoted.
    pub series_verdict: String,
    /// The fixture path relative to the workspace root.
    pub fixture: String,
    /// The model hash of the fixture (`model_hash` over canonical bytes).
    pub fixture_hash: String,
    /// The pre-registered expectation mapping (fixed before the run).
    pub expected: Vec<Expectation>,
    /// The executed run.
    pub run: RunLog,
    /// The mechanical comparison the verdict derives from.
    pub comparison: Comparison,
    /// The named expressibility gaps (design Decision 5).
    pub expressibility_gap: Vec<Gap>,
    /// Recorded conflicts with upstream documents (e.g. proposal vs series).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discrepancies: Vec<String>,
    /// The recorded comparison procedure — the steps `comparison` and
    /// `verdict` were derived by, mechanically.
    pub procedure: Vec<String>,
    /// The derived verdict (closed vocabulary).
    pub verdict: String,
}

/// The per-kata verdict line of a project rollup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KataVerdict {
    pub kata_id: String,
    pub entry_id: String,
    pub verdict: String,
}

/// The project-level gates: P7 (`catherine-alr`) / P8 (`catherine-nq5`)
/// stay blocked unless the project verdict is `survived`; the bead flip
/// is orchestrator-reserved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gates {
    pub p7_catherine_alr: String,
    pub p8_catherine_nq5: String,
    pub note: String,
}

/// The project rollup record (kind `rollup`; the last ledger entry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollupRecord {
    /// The recorded derivation procedure, step by step.
    pub procedure: Vec<String>,
    /// The kata entry ids consumed, in seq order.
    pub kata_entry_ids: Vec<String>,
    /// The per-kata verdicts, in seq order.
    pub per_kata: Vec<KataVerdict>,
    /// The control-gate outcome (`passed` / the failure).
    pub control_gate: String,
    /// The gated phases' status.
    pub gates: Gates,
    /// The derived project verdict (closed vocabulary).
    pub verdict: String,
}

/// The hashed content of one ledger entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryContent {
    /// The -based run order — part of the evidence that the negative
    /// control went first.
    pub seq: u32,
    /// `kata` or `rollup`.
    pub kind: String,
    /// Recording date (YYYY-MM-DD).
    pub recorded: String,
    /// Present iff `kind == "kata"`.
    pub kata: Option<KataRecord>,
    /// Present iff `kind == "rollup"`.
    pub rollup: Option<RollupRecord>,
    /// The model-document hashes this entry validated (fixture + probes).
    pub model_hashes: Vec<String>,
    /// The id of the entry this one corrects, when superseding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
}

/// One content-addressed ledger entry: content + its id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(flatten)]
    pub content: EntryContent,
    /// `sha256:<hex>` over the canonical bytes of `content`.
    pub id: String,
}

/// Canonical bytes of the content — the hash input (design Decision 4).
pub fn content_bytes(content: &EntryContent) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(content)?;
    Ok(write_value(&value))
}

/// The content address: `sha256:` over the canonical content bytes.
pub fn entry_id(content: &EntryContent) -> Result<String, serde_json::Error> {
    Ok(sha256_digest(content_bytes(content)?.as_bytes()))
}

/// Seal content into an entry by computing its id.
pub fn seal(content: EntryContent) -> Result<Entry, serde_json::Error> {
    let id = entry_id(&content)?;
    Ok(Entry { content, id })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal kata content, for the id-stability properties.
    fn sample() -> EntryContent {
        EntryContent {
            seq: 1,
            kind: "kata".into(),
            recorded: "2026-11-12".into(),
            kata: None,
            rollup: None,
            model_hashes: vec!["sha256:aa".into()],
            supersedes: None,
        }
    }

    /// The id is the documented digest of the canonical bytes.
    #[test]
    fn id_is_sha256_of_canonical_content_bytes() {
        let content = sample();
        let expected = format!("sha256:{}", {
            let digest = sha256_digest(content_bytes(&content).unwrap().as_bytes());
            digest.trim_start_matches("sha256:").to_string()
        });
        assert_eq!(entry_id(&content).unwrap(), expected);
    }

    /// Equal contents hash equal; any content change changes the id —
    /// the append-only correction rule's basis (structural, never
    /// snapshot).
    #[test]
    fn equal_content_equal_id_changed_content_changed_id() {
        let a = seal(sample()).unwrap();
        let b = seal(sample()).unwrap();
        assert_eq!(a.id, b.id);
        let mut changed = sample();
        changed.seq = 2;
        assert_ne!(entry_id(&changed).unwrap(), a.id);
    }

    /// The verdict vocabulary is closed: exactly the four series words.
    #[test]
    fn verdict_vocabulary_is_the_closed_series_set() {
        let words: Vec<&str> = [
            Verdict::Killed,
            Verdict::Survived,
            Verdict::Inconclusive,
            Verdict::InvalidRun,
        ]
        .iter()
        .map(|v| v.as_str())
        .collect();
        assert_eq!(
            words,
            vec!["killed", "survived", "inconclusive", "invalid-run"]
        );
    }
}
