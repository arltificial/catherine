//! Purpose: the recorded evidence of the wrapper-laws kata (beads
//! `catherine-lgl.6`) — the Adapter / Proxy / Decorator law battery's
//! fixtures, its pre-registered element inventory, and the ledger entry
//! that carries the comparison.
//!
//! Responsibilities: pin every fixture's canonical model hash (a silent
//! edit breaks the entry's content address); re-execute the kernel on
//! every fixture and assert the finding inventory is exactly the
//! pre-registered one (`fixtures/katas/wrapper-laws/faults.json`, with
//! the clean fixtures empty); and validate the kata ledger: control
//! first and self-negated, the positive entry survived with five unique
//! elements against an executed baseline arm that pins
//! `prototype/wrapper_laws_baseline.py`.
//!
//! Rationale: the entry is evidence only while the files on disk still
//! hash to what it cites and the kernel still emits the witnesses the
//! comparison was frozen against. Everything here is computed from the
//! fixture bytes and the kernel — nothing is copied from the entry.

mod support;

use std::path::Path;

use catherine_core::findings::{EpistemicLabel, FindingKind, ViolationWitness};
use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::harness;
use catherine_ledger::validate::{validate_dir, ValidLedger};
use support::{present, TestError};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const LEDGER: &str = "ledger/katas/wrapper-laws";

/// The baseline artifact's sha256, as the positive entry's arm records it.
const BASELINE_SHA: &str =
    "sha256:08a61ed2061fc1c962d3b3523abf0fef639d54bffcc263df3c47d7727192772e";

/// Every committed fixture, its canonical model hash, and its
/// pre-registered element inventory (empty for a clean fixture).
const FIXTURES: [Fixture; 9] = [
    Fixture {
        path: "fixtures/katas/wrapper-laws/wrapper-laws.ct.json",
        hash: "sha256:b7381782dcaa287963187426cced1b7aa54f236e99bfa5bc1c1e63935dd7447a",
        elements: &[],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json",
        hash: "sha256:90ca8b6119f1af2e05406683a78787b66c95c95eeef70069502a8c989fa9a002",
        elements: &[
            "dangling:adapter_note#0",
            "dangling:adapter_note#1",
            "dangling:adapter_note#2",
        ],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json",
        hash: "sha256:654e0c3569c8ca06ec0a6e99ea41aada2c9cbd70498f8a7018b69d2ce776b682",
        elements: &["violation:eq4@row1"],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json",
        hash: "sha256:42914321bf3a9b226f9d290dc7caf1e3eab26aa3f183ecbe2b060a9751bfdd52",
        elements: &["violation:eq6@row1"],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json",
        hash: "sha256:168e819851031be05eef9261741dbe26507751e8af42cdfac22683a0dae128a9",
        elements: &["dangling:decorator_note#1"],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json",
        hash: "sha256:8f9a71285be087b6eab19206a40b9e85e54712cac4cd2a8969f913f7e131cc5e",
        elements: &[],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json",
        hash: "sha256:cf9ccb0f84bbff19fbac69894b51e58ec7706f42b63d6e8bd2a7df1d0edb0d05",
        elements: &["violation:eq2@row0"],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json",
        hash: "sha256:3afc5473a5706e5843510c03a09298bdf19a364c552203c5dfa4e8e234ea461d",
        elements: &["violation:eq0@row0"],
    },
    Fixture {
        path: "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json",
        hash: "sha256:26bbfb406aa90b04bc21a769f43b879efb4e4396ab215c9acc89de18c8175855",
        elements: &["dangling:wrapper_note#0"],
    },
];

/// One fixture's canonical model hash, recomputed from its canonical bytes.
fn model_hash_of(path: &str) -> Result<String, TestError> {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}"))?;
    let document = parse(&text)?;
    Ok(model_hash(&document))
}

/// One committed fixture: its path, pinned model hash, and the exact
/// witnesses the kernel must emit on it (empty = clean).
#[derive(Clone, Copy)]
struct Fixture {
    path: &'static str,
    hash: &'static str,
    elements: &'static [&'static str],
}

impl Fixture {
    /// The fixture's model hash, recomputed from its canonical bytes.
    fn model_hash(self) -> Result<String, TestError> {
        model_hash_of(self.path)
    }

    /// The kernel's element inventory on this fixture, re-executed.
    fn executed_elements(self) -> Result<Vec<String>, TestError> {
        let run = harness::run(&Path::new(ROOT).join(self.path));
        assert!(run.parsed, "{} parses: {:?}", self.path, run.parse_error);
        let mut found = Vec::new();
        for finding in &run.findings {
            assert_eq!(
                finding.label,
                EpistemicLabel::Executed,
                "{}: every pre-registered element is executed",
                self.path
            );
            found.push(element(&finding.kind));
        }
        Ok(found)
    }
}

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

/// The committed kata ledger, validated; a refusal becomes the test's
/// error with the violations spelled out.
fn validated() -> Result<ValidLedger, TestError> {
    validate_dir(Path::new(ROOT).join(LEDGER).as_path())
        .map_err(|violations| format!("the wrapper-laws ledger refused: {violations:?}").into())
}

/// One fixture matrix pass: each fixture's hash pin and its re-executed
/// element inventory are exactly the pre-registered pair — the law
/// battery holds on its own models, and a silent fixture edit breaks
/// the pin instead of the ledger's arithmetic.
#[test]
fn fixture_matrix_is_the_preregistered_one() -> Result<(), TestError> {
    for fixture in FIXTURES {
        assert_eq!(
            fixture.model_hash()?,
            fixture.hash,
            "{} drifted from the entry or its matrix row",
            fixture.path
        );
        assert_eq!(
            fixture.executed_elements()?,
            fixture.elements,
            "{} drifted from its inventory",
            fixture.path
        );
    }
    Ok(())
}

/// The ledger validates and the near-floor control went first and
/// self-negated (killed, empty unique set, no baseline arm at parity),
/// while the positive entry carries the pre-registered unique set
/// against an executed baseline arm that pins the baseline script.
#[test]
fn wrapper_laws_ledger_records_control_first_and_the_scoped_survivor() -> Result<(), TestError> {
    let valid = validated()?;
    assert_eq!(valid.entries.len(), 2, "control + positive kata");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2);

    let control = present(katas[0].content.kata.as_ref(), "the control kata record")?;
    assert!(control.identity.control, "the near-floor control is first");
    assert_eq!(control.identity.kata_id, "wrapper-laws-floor");
    assert_eq!(
        control.derivation.verdict, "killed",
        "the control self-negated"
    );
    assert!(control.derivation.comparison.unique_detections.is_empty());
    assert!(
        control.derivation.comparison.baseline.is_none(),
        "no arm at parity"
    );

    let positive = present(katas[1].content.kata.as_ref(), "the positive kata record")?;
    assert!(!positive.identity.control);
    assert_eq!(positive.identity.kata_id, "wrapper-laws");
    assert_eq!(positive.derivation.verdict, "survived");
    let faults: Vec<&str> = positive
        .expected
        .iter()
        .map(|row| row.fault.as_str())
        .collect();
    assert_eq!(faults, ["W1", "W2", "W3", "W4"]);
    assert_eq!(
        positive.derivation.comparison.unique_detections, UNIQUE_SET,
        "the recorded unique set is the pre-registered arithmetic"
    );
    let arm = present(
        positive.derivation.comparison.baseline.as_ref(),
        "the executed arm behind a non-empty unique set",
    )?;
    assert_eq!(arm.mode, BaselineMode::Executed);
    assert!(
        arm.artifact.contains(BASELINE_SHA),
        "the arm pins the script"
    );
    assert_eq!(
        arm.detections,
        ["delegation:Adapter:adapter_note#0:missing"]
    );
    Ok(())
}

/// The entries cite the fixtures they validated, and their recorded
/// fixture hash is the canonical model hash of the file on disk.
#[test]
fn wrapper_laws_entries_cite_the_fixtures_they_validated() -> Result<(), TestError> {
    let valid = validated()?;
    let control = valid.kata_entries()[0].content.clone();
    let positive = valid.kata_entries()[1].content.clone();
    assert_eq!(
        control.model_hashes,
        [
            FIXTURES[5].hash,
            FIXTURES[6].hash,
            FIXTURES[7].hash,
            FIXTURES[8].hash,
        ],
        "the control validated its clean model and three floor faults"
    );
    assert_eq!(
        positive.model_hashes,
        [
            FIXTURES[0].hash,
            FIXTURES[1].hash,
            FIXTURES[2].hash,
            FIXTURES[3].hash,
            FIXTURES[4].hash,
        ],
        "the positive kata validated its clean model and four faults"
    );
    for entry in [&control, &positive] {
        let kata = present(entry.kata.as_ref(), "kata record")?;
        assert_eq!(
            kata.provenance.fixture_hash,
            model_hash_of(&kata.provenance.fixture)?
        );
        assert_eq!(entry.model_hashes[0], kata.provenance.fixture_hash);
    }
    Ok(())
}
