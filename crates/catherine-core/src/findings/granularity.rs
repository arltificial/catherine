//! Purpose: the granularity finding's witness shapes (ticket
//! `catherine-kw9.3`) — the calibrated arrow cap and stored-column case
//! plus the witness carrying the offending arrows. Split out of
//! `findings.rs` (now `findings/mod.rs`) to keep the taxonomy module
//! inside the file-size gate; the public surface stays on
//! `crate::findings` via re-exports.

use crate::model::Ident;
use serde::{Deserialize, Serialize};

/// The granularity finding's case: the calibrated arrow cap or the
/// stored-column test. Serialized adjacently tagged as
/// `{"case": …, "detail": {…}}`, like `DomainQuestion`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "case", content = "detail", rename_all = "snake_case")]
pub enum GranularityCase {
    /// More declared arrows than the calibrated cap allows over the
    /// instance's total row count.
    OverCap {
        /// The calibrated cap: `ceil(4.5·√rows)`.
        cap: usize,
        /// The instance's total row count across all objects.
        rows: usize,
    },
    /// More than a third of the declared arrows carry no `stored` entry.
    Unnamed {
        /// How many arrows declare a storing column.
        stored: usize,
    },
}

/// The granularity witness: the tagged case first, then the offending
/// arrows in canonical (sorted) id order — the same
/// structural-witness-before-list order the other witnesses follow.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GranularityWitness {
    /// The tagged granularity case.
    #[serde(flatten)]
    pub case: GranularityCase,
    /// The offending arrows in canonical id order: every declared arrow
    /// for `OverCap`, the arrows lacking a `stored` entry for `Unnamed`.
    pub arrows: Vec<Ident>,
}
