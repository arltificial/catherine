//! Purpose: the frozen assertions of the inaugural migration probe
//! (`add-domain-probe-validation` tasks 5-7, beads `catherine-1od`) —
//! the committed fault set under `fixtures/probes/migration/`, executed
//! through the real Δ engine: the floor control's loss is row-visible,
//! the cell-only loss is named row by row, and the collapsing map is
//! proven non-invertible with the offending element. These are the
//! probe's executed detections, replayable with `cargo test`; the
//! baseline arm is `prototype/migration_baseline.py`.
//!
//! Responsibilities: pin the per-fixture expectation rows the probe
//! catalog pre-registered (`faults.json`), the exact lost-element sets,
//! the target row counts, and the round-trip proof.
//!
//! Rationale: the probe's unique band is derived arithmetically from
//! findings (probe detections minus baseline-identified elements), so
//! the findings themselves must be executable assertions, never prose.

use catherine_core::findings::EpistemicLabel;
use catherine_core::migrate::{reindex, round_trip, MigrationFindingKind};
use catherine_core::model::{Ident, Model, Path, SchemaMap};
use catherine_core::parse::parse;
use std::collections::BTreeMap;

const PROBE_FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/probes/migration/"
);

/// Parse one committed probe fixture.
fn fixture(name: &str) -> Model {
    let text =
        std::fs::read_to_string(format!("{PROBE_FIXTURES}{name}")).expect("fixture readable");
    parse(&text).expect("probe fixture parses (its schema map is total)")
}

/// The lost rows of one object, in finding order.
fn lost_rows(findings: &[catherine_core::migrate::MigrationFinding], object: &str) -> Vec<usize> {
    findings
        .iter()
        .filter_map(|finding| match &finding.kind {
            MigrationFindingKind::LostRow { object: found, row } if found.0 == object => {
                Some(row.0)
            }
            _ => None,
        })
        .collect()
}

/// The lost cells of one arrow, in finding order.
fn lost_cells(findings: &[catherine_core::migrate::MigrationFinding], arrow: &str) -> Vec<usize> {
    findings
        .iter()
        .filter_map(|finding| match &finding.kind {
            MigrationFindingKind::LostCell { arrow: found, row } if found.0 == arrow => Some(row.0),
            _ => None,
        })
        .collect()
}

/// One target object's row count.
fn target_rows(model: &Model, object: &str) -> usize {
    let outcome = reindex(model).expect("the fixture declares a map");
    outcome.instance.rows[&Ident(object.to_string())].len()
}

/// The floor control: a whole object drops, every lost row is exactly
/// the dropped table's row set — the loss the conventional row-count
/// dry run sees in full, so the probe must buy no unique detection.
#[test]
fn floor_control_loses_only_the_fully_visible_table_rows() {
    let model = fixture("floor-visible-loss.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert!(
        outcome
            .lost
            .iter()
            .all(|finding| finding.label == EpistemicLabel::Executed),
        "losses are executed facts"
    );
    assert_eq!(
        lost_rows(&outcome.lost, "RawCapture"),
        (0..6).collect::<Vec<usize>>(),
        "the dropped table's 6 rows are named, in order"
    );
    assert_eq!(
        lost_cells(&outcome.lost, "captured_page"),
        Vec::<usize>::new(),
        "the dropped arrow holds no valued cells in this fixture"
    );
    assert_eq!(outcome.lost.len(), 6, "no other loss is claimed");
    assert_eq!(target_rows(&model, "ServingDocument"), 8);
    assert_eq!(target_rows(&model, "ServingPage"), 16);
    assert_eq!(target_rows(&model, "ServingEdition"), 4);
}

/// The cell-only loss: every row survives, so the row counts and key
/// checksums a conventional dry run reads are unchanged; the lost cells
/// are named arrow by arrow.
#[test]
fn cell_only_loss_names_each_valued_cell_with_rows_preserved() {
    let model = fixture("lost-data-cells.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert!(
        lost_rows(&outcome.lost, "RawCapture").is_empty(),
        "rows are preserved: the loss is cell-level"
    );
    assert_eq!(
        lost_cells(&outcome.lost, "capture_note"),
        (0..4).collect::<Vec<usize>>(),
        "the 4 valued capture_note cells are named"
    );
    assert_eq!(
        lost_cells(&outcome.lost, "captured_page"),
        (0..6).collect::<Vec<usize>>(),
        "the 6 valued captured_page cells are named"
    );
    assert_eq!(outcome.lost.len(), 10, "no other loss is claimed");
    assert_eq!(target_rows(&model, "ServingPage"), 16);
    assert_eq!(target_rows(&model, "ServingCapture"), 6);
}

/// The candidate inverse: the serving schema straight back to the
/// source names — the map that would have to recover both a page row
/// and a capture row from one serving row.
fn candidate_inverse(model: &Model) -> SchemaMap {
    let mut objects: BTreeMap<Ident, Option<Ident>> = BTreeMap::new();
    for (serving, source) in [
        ("ServingDocument", "RawDocument"),
        ("ServingEdition", "RawEdition"),
        ("ServingPage", "RawPage"),
    ] {
        objects.insert(Ident(serving.into()), Some(Ident(source.into())));
    }
    let mut arrows: BTreeMap<Ident, Option<Path>> = BTreeMap::new();
    for (serving, source) in [
        ("serving_capture_note", "capture_note"),
        ("serving_capture_of", "capture_of"),
        ("serving_doc_edition", "doc_edition"),
        ("serving_next_page", "next_page"),
        ("serving_page_of", "page_of"),
    ] {
        arrows.insert(
            Ident(serving.into()),
            Some(Path(vec![Ident(source.into())])),
        );
    }
    SchemaMap {
        target: model.schema.clone(),
        objects,
        arrows,
    }
}

/// The non-invertibility arm: the collapsing map emits no lost-data
/// finding at all (every capture row lands on its page row), and the
/// round-trip proves the smallest capture element unrecoverable by
/// naming it and the element the candidate inverse returns instead.
#[test]
fn collapsing_map_is_proven_non_invertible_with_the_offending_element() {
    let model = fixture("non-invertible-collapse.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert!(
        outcome.lost.is_empty(),
        "no lost-data finding: the collapse places every row"
    );
    assert_eq!(
        target_rows(&model, "ServingPage"),
        16,
        "the target is row-based on the page preimage"
    );
    let forward = model.schema_map.clone().expect("the fixture carries a map");
    let inverse = candidate_inverse(&model);
    let proof = round_trip(&model, &forward, &inverse)
        .expect("the collapse is not invertible under the candidate inverse");
    assert_eq!(proof.object.0, "RawPageCapture", "the smallest offender");
    assert_eq!(proof.row.0, 0, "its row");
    assert!(
        proof
            .detail
            .contains("has the forward image ServingPage row 0"),
        "the proof names the forward image: {}",
        proof.detail
    );
    assert!(
        proof.detail.contains("returns RawPage row 0"),
        "the proof names what the inverse yields instead: {}",
        proof.detail
    );
    assert!(
        proof.detail.contains("not recoverable"),
        "the proof states the non-recovery: {}",
        proof.detail
    );
    // The probe harness parses this line: the executed round-trip arm's
    // machine-readable witness, never a hand-copied string.
    println!(
        "PROBE_ROUND_TRIP {}",
        serde_json::json!({
            "element": format!("round_trip:{}#{}", proof.object.0, proof.row.0),
            "detail": proof.detail,
        })
    );
}
