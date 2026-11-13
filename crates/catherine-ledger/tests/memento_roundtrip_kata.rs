//! Purpose: the frozen assertions of the memento-roundtrip kata
//! (`catherine-lgl.5`) — the committed fault set under
//! `fixtures/katas/memento-roundtrip/`, executed through the real Δ
//! engine: a faithful save/restore identity carries everything, a
//! dropped required field and a lossy optional field are named cell by
//! cell, and a version-skew restore collapses the old per-page snapshot
//! and is proven non-invertible with the offending element.
//!
//! Responsibilities: pin each fixture's model hash, validate the ledger
//! directory through the real validator, check the control went first
//! and self-negated, pin the positives' unique sets and executed
//! baseline arms, and replay the Δ findings plus the round-trip proof.
//!
//! Rationale: the kata claims its unique band arithmetically (probe
//! elements minus baseline-identified elements); the findings and the
//! proof must be executable assertions, never prose.

use std::collections::BTreeMap;
use std::path::Path;

use catherine_core::hash::{model_hash, sha256_digest};
use catherine_core::migrate::{reindex, round_trip, MigrationFinding, MigrationFindingKind};
use catherine_core::model::{Ident, Model, Path as ModelPath, SchemaMap};
use catherine_core::parse::parse;
use catherine_ledger::entry::{BaselineMode, Entry};
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const KATA_FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/katas/memento-roundtrip/"
);

/// Parse one committed kata fixture.
fn fixture(name: &str) -> Model {
    let text = std::fs::read_to_string(format!("{KATA_FIXTURES}{name}")).expect("fixture readable");
    parse(&text).expect("kata fixture parses (its schema map is total)")
}

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// The lost cells of one arrow, in finding order.
fn lost_cells(findings: &[MigrationFinding], arrow: &str) -> Vec<usize> {
    findings
        .iter()
        .filter_map(|finding| match &finding.kind {
            MigrationFindingKind::LostCell { arrow: found, row } if found.0 == arrow => Some(row.0),
            _ => None,
        })
        .collect()
}

/// The candidate inverse: each target object back to its sorted-first
/// source preimage, each target arrow back to the source arrow whose
/// forward path names it. The map that would have to recover every
/// source element from its forward image.
fn reverse_map(model: &Model) -> SchemaMap {
    let forward = model.schema_map.as_ref().expect("fixture carries a map");
    let mut preimages: BTreeMap<Ident, Vec<Ident>> = BTreeMap::new();
    for (source, target) in &forward.objects {
        if let Some(target) = target {
            preimages
                .entry(target.clone())
                .or_default()
                .push(source.clone());
        }
    }
    let objects: BTreeMap<Ident, Option<Ident>> = preimages
        .iter()
        .map(|(target, sources)| (target.clone(), Some(sources[0].clone())))
        .collect();
    let arrows: BTreeMap<Ident, Option<ModelPath>> = forward
        .target
        .arrows
        .keys()
        .map(|target| {
            let source = forward
                .arrows
                .iter()
                .find(|(_, path)| {
                    path.as_ref()
                        .is_some_and(|path| path.0.len() == 1 && path.0[0] == *target)
                })
                .map(|(source, _)| source.clone());
            (target.clone(), source.map(|source| ModelPath(vec![source])))
        })
        .collect();
    SchemaMap {
        target: model.schema.clone(),
        objects,
        arrows,
    }
}

/// The committed baseline artifact's content address.
fn baseline_hash() -> String {
    let bytes = std::fs::read(Path::new(ROOT).join("prototype/memento_roundtrip_baseline.py"))
        .expect("baseline readable");
    sha256_digest(&bytes)
}

/// The fixtures' identities are pinned: an edit to any committed kata
/// fixture must be a new ledger entry, never a silent change.
#[test]
fn fixture_hashes_are_pinned() {
    assert_eq!(
        fixture_hash("fixtures/katas/memento-roundtrip/restore-identity.ct.json"),
        "sha256:ea85d088cf0fba624b3f491b8f36fe6c6f20efe3933c6bcb699021b0a003b8c3"
    );
    assert_eq!(
        fixture_hash("fixtures/katas/memento-roundtrip/restore-dropped-field.ct.json"),
        "sha256:7cd69dea2fe94b4c259731fea35b3e1222e4049ca7eb4a7ffc6645db41c33c15"
    );
    assert_eq!(
        fixture_hash("fixtures/katas/memento-roundtrip/restore-lossy-optional.ct.json"),
        "sha256:cd3ab02117aa48cef890fc6d296096403e01eb1ad5d0df64b422aeed2ed06d4d"
    );
    assert_eq!(
        fixture_hash("fixtures/katas/memento-roundtrip/restore-version-skew.ct.json"),
        "sha256:79af49a19e1145932e0465e6b22aadc00d18144e5421b50a980c718c0ec513fa"
    );
}

/// The identity control entry: first, faithful, self-negated.
fn assert_control_entry(entry: &Entry) {
    let kata = entry.content.kata.as_ref().expect("kata");
    assert!(kata.control, "the identity control is first");
    assert_eq!(kata.verdict, "killed", "the control self-negated");
    assert!(kata.comparison.unique_detections.is_empty());
    assert_eq!(
        kata.fixture,
        "fixtures/katas/memento-roundtrip/restore-identity.ct.json"
    );
    assert_eq!(
        entry.content.model_hashes,
        vec![kata.fixture_hash.clone()],
        "the control validated the fixture it names"
    );
    assert_eq!(
        kata.fixture_hash,
        fixture_hash(&kata.fixture),
        "the control's recorded hash is the canonical model hash"
    );
}

/// One positive entry: executed baseline arm, pinned fixture, exact
/// arithmetic unique set.
fn assert_positive_entry(entry: &Entry, fixture_name: &str, unique: &[&str]) {
    let kata = entry.content.kata.as_ref().expect("kata");
    assert_eq!(kata.kata_id, "memento-roundtrip");
    assert_eq!(kata.verdict, "survived");
    assert!(kata.fixture.ends_with(fixture_name));
    assert_eq!(
        kata.comparison.unique_detections,
        unique
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
    );
    let baseline = kata
        .comparison
        .baseline
        .as_ref()
        .expect("an executed baseline arm carries the unique set");
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert!(
        baseline.artifact.contains(&baseline_hash()),
        "the baseline arm pins the committed script: {}",
        baseline.artifact
    );
    assert_eq!(
        entry.content.model_hashes,
        vec![kata.fixture_hash.clone()],
        "the validated model is the fixture it names"
    );
    assert_eq!(
        kata.fixture_hash,
        fixture_hash(&kata.fixture),
        "the recorded fixture hash is the canonical model hash"
    );
}

/// The ledger validates, the control went first and self-negated, and
/// every positive carries the executed baseline arm and its unique set.
#[test]
fn ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/katas/memento-roundtrip");
    let valid = validate_dir(&dir).expect("the memento ledger validates");
    assert_eq!(valid.entries.len(), 4, "control + three faults");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 4);
    assert_control_entry(katas[0]);
    let expected: [(&str, &[&str]); 3] = [
        (
            "restore-dropped-field.ct.json",
            &["lost_cell:snap_word_count#0", "lost_cell:snap_word_count#1"],
        ),
        (
            "restore-lossy-optional.ct.json",
            &["lost_cell:page_note#1", "lost_cell:page_note#3"],
        ),
        (
            "restore-version-skew.ct.json",
            &[
                "lost_cell:page_next#0",
                "lost_cell:page_note#1",
                "lost_cell:page_text#2",
                "round_trip:SnapshotPage#0",
            ],
        ),
    ];
    for (entry, (fixture_name, unique)) in katas[1..].iter().zip(expected) {
        assert_positive_entry(entry, fixture_name, unique);
    }
}

/// The near-floor control: the identity save/restore loses nothing and
/// the candidate inverse returns every element to itself.
#[test]
fn identity_control_has_no_lost_data_and_is_invertible() {
    let model = fixture("restore-identity.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert!(outcome.lost.is_empty(), "no false finding on the identity");
    assert_eq!(
        outcome
            .instance
            .rows
            .get(&Ident("Document".into()))
            .map(Vec::len),
        Some(2)
    );
    let forward = model.schema_map.clone().expect("fixture carries a map");
    assert!(
        round_trip(&model, &forward, &reverse_map(&model)).is_none(),
        "a faithful round trip is the identity"
    );
}

/// Fault (a): the required field dropped by the restore map is named at
/// every valued cell; every object and row count survives.
#[test]
fn dropped_required_field_is_named_cell_by_cell() {
    let model = fixture("restore-dropped-field.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert_eq!(outcome.lost.len(), 2, "one finding per valued cell");
    assert_eq!(lost_cells(&outcome.lost, "snap_word_count"), vec![0, 1]);
    assert_eq!(
        outcome
            .instance
            .rows
            .get(&Ident("Document".into()))
            .map(Vec::len),
        Some(2),
        "rows survive; only the field is lost"
    );
}

/// Fault (b): the optional field is named only where it carried a value
/// (pages 1 and 3); unvalued rows never produce a finding.
#[test]
fn lossy_optional_field_is_named_only_where_valued() {
    let model = fixture("restore-lossy-optional.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert_eq!(outcome.lost.len(), 2, "only the valued cells are lost");
    assert_eq!(lost_cells(&outcome.lost, "page_note"), vec![1, 3]);
}

/// Fault (c): the version-skew restore names every page-level cell the
/// new shape cannot hold, and the round trip proves the collapsed pages
/// unrecoverable by naming the smallest offending element.
#[test]
fn version_skew_collapse_is_proven_non_invertible() {
    let model = fixture("restore-version-skew.ct.json");
    let outcome = reindex(&model).expect("the fixture declares a map");
    assert_eq!(outcome.lost.len(), 5);
    assert_eq!(lost_cells(&outcome.lost, "page_next"), vec![0]);
    assert_eq!(lost_cells(&outcome.lost, "page_note"), vec![1]);
    assert_eq!(lost_cells(&outcome.lost, "page_text"), vec![0, 1, 2]);
    assert_eq!(
        outcome
            .instance
            .rows
            .get(&Ident("Document".into()))
            .map(Vec::len),
        Some(1),
        "three old page rows collapse onto one document row"
    );
    let forward = model.schema_map.clone().expect("fixture carries a map");
    let proof = round_trip(&model, &forward, &reverse_map(&model))
        .expect("the collapse is not invertible under the candidate inverse");
    assert_eq!(proof.object.0, "SnapshotPage", "the smallest offender");
    assert_eq!(proof.row.0, 0, "its row");
    assert!(
        proof
            .detail
            .contains("share the forward image Document row 0"),
        "the proof names the forward image: {}",
        proof.detail
    );
    assert!(
        proof.detail.contains("returns Snapshot row 0"),
        "the proof names what the inverse yields instead: {}",
        proof.detail
    );
    assert!(proof.detail.contains("not recoverable"));
}
