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
//! is the honest record of that).
//!
//! Rationale: the proposal text rounds kata-3 (Vending) to "6/6" — the
//! series ledger says otherwise (F2 bounds, F3 command vocabulary, F5
//! collision caught structurally; F6 REFUTED by an executed MR≠RM
//! counterexample). The series record is ground truth; the discrepancy
//! is recorded here so the ledger entry can carry it forward.

use crate::entry::Expectation;

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
