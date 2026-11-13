//! Purpose: the recorded evidence of the wrapper-laws kata (beads
//! `catherine-lgl.6`) — the Adapter / Proxy / Decorator law battery's
//! fixtures, its pre-registered element inventory, and the ledger entry
//! that carries the comparison.
//!
//! Responsibilities: pin every fixture's canonical model hash (a silent
//! edit breaks the entry's content address); re-execute the kernel on
//! every fault fixture and assert the finding inventory is exactly the
//! pre-registered one (`fixtures/katas/wrapper-laws/faults.json`); assert
//! the two clean fixtures produce zero findings; and validate the kata
//! ledger: control first and self-negated, the positive entry survived
//! with five unique elements against an executed baseline arm that pins
//! `prototype/wrapper_laws_baseline.py`.
//!
//! Rationale: the entry is evidence only while the files on disk still
//! hash to what it cites and the kernel still emits the witnesses the
//! comparison was frozen against. Everything here is computed from the
//! fixture bytes and the kernel — nothing is copied from the entry.

use std::path::Path;

use catherine_core::findings::{EpistemicLabel, FindingKind, ViolationWitness};
use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::harness;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The baseline artifact's sha256, as the positive entry's arm records it.
const BASELINE_SHA: &str =
    "sha256:08a61ed2061fc1c962d3b3523abf0fef639d54bffcc263df3c47d7727192772e";

/// Every committed fixture and its canonical model hash.
const FIXTURES: [(&str, &str); 9] = [
    (
        "fixtures/katas/wrapper-laws/wrapper-laws.ct.json",
        "sha256:b7381782dcaa287963187426cced1b7aa54f236e99bfa5bc1c1e63935dd7447a",
    ),
    (
        "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json",
        "sha256:90ca8b6119f1af2e05406683a78787b66c95c95eeef70069502a8c989fa9a002",
    ),
    (
        "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json",
        "sha256:654e0c3569c8ca06ec0a6e99ea41aada2c9cbd70498f8a7018b69d2ce776b682",
    ),
    (
        "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json",
        "sha256:42914321bf3a9b226f9d290dc7caf1e3eab26aa3f183ecbe2b060a9751bfdd52",
    ),
    (
        "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json",
        "sha256:168e819851031be05eef9261741dbe26507751e8af42cdfac22683a0dae128a9",
    ),
    (
        "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json",
        "sha256:8f9a71285be087b6eab19206a40b9e85e54712cac4cd2a8969f913f7e131cc5e",
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json",
        "sha256:cf9ccb0f84bbff19fbac69894b51e58ec7706f42b63d6e8bd2a7df1d0edb0d05",
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json",
        "sha256:3afc5473a5706e5843510c03a09298bdf19a364c552203c5dfa4e8e234ea461d",
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json",
        "sha256:26bbfb406aa90b04bc21a769f43b879efb4e4396ab215c9acc89de18c8175855",
    ),
];

/// The pre-registered finding inventory per fault fixture.
const FAULT_ELEMENTS: [(&str, &[&str]); 7] = [
    (
        "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json",
        &["violation:eq2@row0"],
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json",
        &["violation:eq0@row0"],
    ),
    (
        "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json",
        &["dangling:wrapper_note#0"],
    ),
    (
        "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json",
        &[
            "dangling:adapter_note#0",
            "dangling:adapter_note#1",
            "dangling:adapter_note#2",
        ],
    ),
    (
        "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json",
        &["violation:eq4@row1"],
    ),
    (
        "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json",
        &["violation:eq6@row1"],
    ),
    (
        "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json",
        &["dangling:decorator_note#1"],
    ),
];

/// The pre-registered unique set the positive entry records.
const UNIQUE_SET: [&str; 5] = [
    "dangling:adapter_note#1",
    "dangling:adapter_note#2",
    "dangling:decorator_note#1",
    "violation:eq4@row1",
    "violation:eq6@row1",
];

/// One finding's stable element id, in the pre-registered spelling.
fn element(kind: &FindingKind) -> String {
    match kind {
        FindingKind::Dangling(witness) => format!("dangling:{}#{}", witness.arrow.0, witness.src.0),
        FindingKind::Violation(ViolationWitness::Equation(witness)) => {
            format!("violation:eq{}@row{}", witness.equation, witness.row.0)
        }
        other => panic!("unexpected finding kind in a wrapper-laws fixture: {other:?}"),
    }
}

/// Execute the kernel over one fixture and return its element inventory.
fn elements(path: &str) -> Vec<String> {
    let run = harness::run(&Path::new(ROOT).join(path));
    assert!(run.parsed, "{path} parses: {:?}", run.parse_error);
    run.findings
        .iter()
        .map(|finding| {
            assert_eq!(
                finding.label,
                EpistemicLabel::Executed,
                "{path}: every pre-registered element is executed"
            );
            element(&finding.kind)
        })
        .collect()
}

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// The fixtures' identities are pinned: an edit to any committed fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn wrapper_laws_fixture_hashes_are_pinned() {
    for (path, pinned) in FIXTURES {
        assert_eq!(fixture_hash(path), pinned, "{path} drifted from the entry");
    }
}

/// The law battery holds on its own models: both clean fixtures execute
/// with zero findings.
#[test]
fn clean_fixtures_execute_without_findings() {
    for clean in [
        "fixtures/katas/wrapper-laws/wrapper-laws.ct.json",
        "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json",
    ] {
        assert_eq!(elements(clean), Vec::<String>::new(), "{clean} is clean");
    }
}

/// Each fault fixture emits exactly its pre-registered witnesses — the
/// probe arm the ledger entry records, re-executed.
#[test]
fn fault_fixtures_report_the_preregistered_elements() {
    for (path, expected) in FAULT_ELEMENTS {
        assert_eq!(
            elements(path),
            expected,
            "{path} drifted from its matrix row"
        );
    }
}

/// The ledger validates and the near-floor control went first and
/// self-negated (killed, empty unique set, no baseline arm at parity).
#[test]
fn wrapper_laws_ledger_records_control_first() {
    let dir = Path::new(ROOT).join("ledger/katas/wrapper-laws");
    let valid = validate_dir(&dir).expect("the wrapper-laws ledger validates");
    assert_eq!(valid.entries.len(), 2, "control + positive kata");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2);

    let control = katas[0].content.kata.clone().expect("control kata");
    assert!(control.control, "the near-floor control is first");
    assert_eq!(control.kata_id, "wrapper-laws-floor");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    assert!(control.comparison.baseline.is_none(), "no arm at parity");
}

/// The positive entry carries the pre-registered unique set against an
/// executed baseline arm that pins the baseline script.
#[test]
fn wrapper_laws_ledger_records_the_scoped_survivor() {
    let dir = Path::new(ROOT).join("ledger/katas/wrapper-laws");
    let valid = validate_dir(&dir).expect("the wrapper-laws ledger validates");
    let positive = valid.kata_entries()[1]
        .content
        .kata
        .clone()
        .expect("positive kata");
    assert!(!positive.control);
    assert_eq!(positive.kata_id, "wrapper-laws");
    assert_eq!(positive.verdict, "survived");
    let faults: Vec<&str> = positive
        .expected
        .iter()
        .map(|row| row.fault.as_str())
        .collect();
    assert_eq!(faults, ["W1", "W2", "W3", "W4"]);
    assert_eq!(
        positive.comparison.unique_detections, UNIQUE_SET,
        "the recorded unique set is the pre-registered arithmetic"
    );
    let arm = positive
        .comparison
        .baseline
        .as_ref()
        .expect("a non-empty unique set carries an executed arm");
    assert_eq!(arm.mode, BaselineMode::Executed);
    assert!(
        arm.artifact.contains(BASELINE_SHA),
        "the arm pins the script"
    );
    assert_eq!(
        arm.detections,
        ["delegation:Adapter:adapter_note#0:missing"]
    );
}

/// The entries cite the fixtures they validated, and their recorded
/// fixture hash is the canonical model hash of the file on disk.
#[test]
fn wrapper_laws_entries_cite_the_fixtures_they_validated() {
    let dir = Path::new(ROOT).join("ledger/katas/wrapper-laws");
    let valid = validate_dir(&dir).expect("the wrapper-laws ledger validates");
    let control = valid.kata_entries()[0].content.clone();
    let positive = valid.kata_entries()[1].content.clone();
    assert_eq!(
        control.model_hashes,
        [FIXTURES[5].1, FIXTURES[6].1, FIXTURES[7].1, FIXTURES[8].1,],
        "the control validated its clean model and three floor faults"
    );
    assert_eq!(
        positive.model_hashes,
        [
            FIXTURES[0].1,
            FIXTURES[1].1,
            FIXTURES[2].1,
            FIXTURES[3].1,
            FIXTURES[4].1,
        ],
        "the positive kata validated its clean model and four faults"
    );
    for entry in [&control, &positive] {
        let kata = entry.kata.as_ref().expect("kata record");
        assert_eq!(kata.fixture_hash, fixture_hash(&kata.fixture));
        assert_eq!(entry.model_hashes[0], kata.fixture_hash);
    }
}
