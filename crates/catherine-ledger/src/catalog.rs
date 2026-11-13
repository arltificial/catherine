//! Purpose: the pre-registered per-kata expectation mapping — what each
//! kata's faults CAN mean under catherine's path-equation language,
//! fixed before any run (design Decision 3: no goalpost moves). The
//! source of truth is the ct-catlab series verdict ledger
//! (`ak/matadero/series/ct-catlab/SERIES.md`); every entry here cites
//! its series row.
//!
//! Responsibilities: [`expectations`] maps a kata id to its expectation
//! list; an unknown kata id maps to an empty list (an unregistered kata
//! cannot run under the ledger — the entry's expectations being empty
//! is the honest record of that). Domain probes are pre-registered in
//! the same catalog surface ([`probe_expectations`], same row shape);
//! the class-level rows are the pre-registration and
//! [`probe_catalog_hash`] is its content address, which the entry
//! carries.
//!
//! Rationale: the proposal text rounds kata-3 (Vending) to "6/6" — the
//! series ledger says otherwise (F2 bounds, F3 command vocabulary, F5
//! collision caught structurally; F6 REFUTED by an executed MR≠RM
//! counterexample). The series record is ground truth; the discrepancy
//! is recorded here so the ledger entry can carry it forward.

use crate::entry::Expectation;
use catherine_core::canonical::write_value;
use catherine_core::hash::sha256_digest;

/// kata-6 FizzBuzz: the negative control — the series verdict is killed
/// with zero unique detections; catherine must self-negate.
fn fizzbuzz() -> Vec<Expectation> {
    vec![Expectation {
        fault: "schema-forced".into(),
        series_class: "behavioral".into(),
        catherine_class: "construction".into(),
        detail: "negative control: the series verdict is killed (conventional 3/3, \
                 categorical zero unique detections); catherine must self-negate — \
                 no path-equation expression of FizzBuzz rules may buy detection"
            .into(),
    }]
}

/// kata-1 Bowling: partial structural catches; temporal semantics is
/// the validated gap.
fn bowling() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "schema-forced".into(),
            series_class: "structural".into(),
            catherine_class: "finding".into(),
            detail: "the frame/roll accounting the series caught structurally (3/5): \
                     path equations + totality over the scoring relation"
                .into(),
        },
        Expectation {
            fault: "F-temporal".into(),
            series_class: "behavioral".into(),
            catherine_class: "gap".into(),
            detail: "history-dependence is the validated CT limit — expected PARTIAL; \
                     the gap is recorded, never papered over"
                .into(),
        },
    ]
}

/// kata-2 Gilded Rose: the two structural catches (Sulfuras, backstage).
fn gilded_rose() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "F-sulfuras".into(),
            series_class: "structural".into(),
            catherine_class: "finding".into(),
            detail: "Sulfuras frozen-quality invariant: the series caught it \
                     structurally; a path equation + totality expresses it"
                .into(),
        },
        Expectation {
            fault: "F-backstage".into(),
            series_class: "structural".into(),
            catherine_class: "finding".into(),
            detail: "post-concert pass invalidity: the series' second structural catch".into(),
        },
    ]
}

/// kata-3 Vending: F2/F3/F5 caught structurally; F6 refuted — the
/// proposal's "6/6" rounding is wrong, the series ledger is ground
/// truth, and the discrepancy is recorded.
fn vending() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "F2".into(),
            series_class: "structural".into(),
            catherine_class: "finding".into(),
            detail: "coin bounds caught structurally in the series run".into(),
        },
        Expectation {
            fault: "F3".into(),
            series_class: "structural".into(),
            catherine_class: "finding".into(),
            detail: "command vocabulary caught structurally in the series run".into(),
        },
        Expectation {
            fault: "F5".into(),
            series_class: "structural".into(),
            catherine_class: "finding".into(),
            detail: "collision caught structurally in the series run".into(),
        },
        Expectation {
            fault: "F6".into(),
            series_class: "structural".into(),
            catherine_class: "gap".into(),
            detail: "REFUTED in the series by an executed MR≠RM counterexample — \
                     the proposal's '6/6' rounding is wrong; the series ledger is \
                     ground truth and the discrepancy is recorded in the entry"
                .into(),
        },
    ]
}

/// The pre-registered expectations for one kata id (empty for an
/// unregistered kata — recorded honestly, never invented).
pub fn expectations(kata_id: &str) -> Vec<Expectation> {
    match kata_id {
        "kata-6" => fizzbuzz(),
        "kata-1" => bowling(),
        "kata-2" => gilded_rose(),
        "kata-3" => vending(),
        _ => Vec::new(),
    }
}

/// The registered domain-probe ids. Probes ride the kata entry shape in
/// their own ledger directory (design Decision 1 revised); this list is
/// the closed set the validator recognizes.
pub const PROBE_IDS: [&str; 2] = ["probe-migration", "probe-rule-lens-extracted"];

/// probe-migration: Δ migration at real scale — the pre-registered
/// lost-data and non-invertibility expectations, plus the floor control
/// where the conventional row-count/checksum dry run sees the whole
/// loss.
fn probe_migration() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "lost-data".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "a Δ migration drops a source row with no image witness; the \
                     pre-registered expectation is a named-row lost-data finding (no \
                     incumbent emits named-row lost-data witnesses); the baseline \
                     row-count/checksum dry run catches only losses visible in counts"
                .into(),
        },
        Expectation {
            fault: "non-invertibility".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "Δ∘Δ′ is not the identity on the declared schema map; the \
                     pre-registered expectation is a counterexample naming the failing \
                     element; a dry run cannot express it"
                .into(),
        },
        Expectation {
            fault: "floor-visible-loss".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "floor control — a loss fully visible to the row-count/checksum \
                     dry run; the probe must show zero unique detections here (design \
                     Decision 2)"
                .into(),
        },
    ]
}

/// probe-rule-lens-extracted: the rule lens restricted to extracted
/// tables no DMN analyzer can ingest — the four witness classes, plus
/// the near-floor table where the scripted conventional review catches
/// every injected fault.
fn probe_rule_lens_extracted() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "gap".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "a coverage gap in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named gap finding under \
                     the extracted-table scope"
                .into(),
        },
        Expectation {
            fault: "overlap".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "overlapping rules in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named overlap finding \
                     under the extracted-table scope"
                .into(),
        },
        Expectation {
            fault: "shadow".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "a shadowed rule in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named shadow finding \
                     under the extracted-table scope"
                .into(),
        },
        Expectation {
            fault: "derivable".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "a derivable rule in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named derivable finding \
                     under the extracted-table scope"
                .into(),
        },
        Expectation {
            fault: "floor-near-trivial".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "floor control — at the near-floor table the conventional \
                     scripted review catches every injected fault; the probe must \
                     show zero unique detections here (design Decision 2 / rule-lens \
                     precedent)"
                .into(),
        },
    ]
}

/// Whether the id names a registered domain probe.
pub fn is_probe(id: &str) -> bool {
    PROBE_IDS.contains(&id)
}

/// The pre-registered expectations for one probe id (empty for an
/// unregistered probe — the same discipline as [`expectations`]: an
/// unknown id cannot derive a verdict).
pub fn probe_expectations(probe_id: &str) -> Vec<Expectation> {
    match probe_id {
        "probe-migration" => probe_migration(),
        "probe-rule-lens-extracted" => probe_rule_lens_extracted(),
        _ => Vec::new(),
    }
}

/// The content address of a probe's pre-registered catalog: `sha256:`
/// over the canonical bytes of its rows, via the kernel's normative
/// writer (the ledger's established id discipline). `None` when the id
/// has no rows — there is no catalog to reference.
pub fn probe_catalog_hash(probe_id: &str) -> Option<String> {
    let rows = probe_expectations(probe_id);
    if rows.is_empty() {
        return None;
    }
    let value = serde_json::to_value(&rows).ok()?;
    Some(sha256_digest(write_value(&value).as_bytes()))
}
