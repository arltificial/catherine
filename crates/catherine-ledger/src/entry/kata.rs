//! The kata-level ledger record and its field groups.
//!
//! Responsibilities: [`KataRecord`] — one per ported kata; the negative
//! control is `control: true` and must be the first kata entry in the
//! ledger — and the three sub-records its fields group into:
//! [`KataIdentity`] (which kata, its display name, the negative-control
//! role), [`Provenance`] (the series grounding: report, verdict,
//! fixture), and [`Derivation`] (the mechanical comparison, the
//! procedure that derived it, and the verdict).
//!
//! Rationale: the flat record grew past the field-count cap, and the
//! fields already read as three blocks. Every group is `#[serde(flatten)]`,
//! so the serialized JSON — and therefore every sealed entry id over the
//! canonical bytes — is unchanged.

use serde::{Deserialize, Serialize};

use super::{Comparison, Expectation, Gap, RunLog};

/// Which kata the record is about: the series id, the human name, and
/// the negative-control role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KataIdentity {
    /// The series kata id (`kata-6`, …).
    pub kata_id: String,
    /// Human name, including the negative-control role when present.
    pub name: String,
    /// True only for the FizzBuzz negative control.
    pub control: bool,
}

/// The series grounding: the report the expectation is read from, the
/// series' own verdict, and the fixture (path + model hash) the run used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// The series report this expectation is grounded in.
    pub series_report: String,
    /// The series verdict, quoted.
    pub series_verdict: String,
    /// The fixture path relative to the workspace root.
    pub fixture: String,
    /// The model hash of the fixture (`model_hash` over canonical bytes).
    pub fixture_hash: String,
}

/// The mechanical comparison, the recorded procedure that derived it,
/// and the verdict that follows — kept together because the procedure
/// is exactly the derivation trail from `comparison` to `verdict`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Derivation {
    /// The mechanical comparison the verdict derives from.
    pub comparison: Comparison,
    /// The recorded comparison procedure — the steps `comparison` and
    /// `verdict` were derived by, mechanically.
    pub procedure: Vec<String>,
    /// The derived verdict (closed vocabulary).
    pub verdict: String,
}

/// The kata-level record (one per ported kata; the negative control is
/// `control: true` and must be the first kata entry in the ledger).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KataRecord {
    /// Which kata this record is about.
    #[serde(flatten)]
    pub identity: KataIdentity,
    /// The series grounding (report, verdict, fixture).
    #[serde(flatten)]
    pub provenance: Provenance,
    /// The pre-registered expectation mapping (fixed before the run).
    pub expected: Vec<Expectation>,
    /// The executed run.
    pub run: RunLog,
    /// The comparison, its derivation procedure, and the verdict.
    #[serde(flatten)]
    pub derivation: Derivation,
    /// The named expressibility gaps (design Decision 5).
    pub expressibility_gap: Vec<Gap>,
    /// Recorded conflicts with upstream documents (e.g. proposal vs series).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discrepancies: Vec<String>,
    /// The hash of the committed expectation catalog the entry ran
    /// against; `None` for the ct-catlab katas and the rule-lens ledgers
    /// whose expectations are code-frozen and predate this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_hash: Option<String>,
}
