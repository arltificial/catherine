//! Purpose: the GREEN proof for the domain-probe ledger
//! (`add-domain-probe-validation`, tasks.md task 4) — a probe directory
//! under `probes/migration/` validates, every sealed id recomputes from
//! the bytes on disk, the harness derives the recorded verdict, and the
//! task-1 rejection matrix still refuses the mutated ledgers. Plus the
//! invariant that recording probes never touches the sealed kata rollup.
//!
//! Responsibilities: build the two-entry GREEN fixture (floor control +
//! `probe-migration` entry with its pre-registered catalog hash and an
//! executed baseline arm); round-trip each written file through
//! `validate::read_entry_file` and `entry_id`; pin the four refusals
//! (`UniqueWithoutBaseline`, `CitedBaselineCannotSurvive`,
//! `NegativeControlMissing`, `ControlDidNotSelfNegate`); pin the
//! catalog-hash refusals that make a retrofitted expectation
//! unresolvable; prove supersession keeps the ledger append-only; and
//! compare `ledger-check` stdout/status plus every `ledger/*.json`
//! digest before and after probe recording.
//!
//! Rationale: a probe entry is evidence only if its content address, its
//! catalog pin, and its derived verdict all survive a re-read from disk —
//! the same discipline the kata ledger enforces. The kata rollup is a
//! sealed artifact of a different ledger; probe recording MUST leave it
//! byte-identical, so invariance is asserted by digest, never by
//! convention.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use catherine_core::hash::sha256_digest;
use catherine_ledger::catalog;
use catherine_ledger::entry::{
    entry_id, seal, BaselineArm, BaselineMode, Comparison, EntryContent, Expectation, KataRecord,
    RunLog, Verdict,
};
use catherine_ledger::harness::derive_probe_verdict;
use catherine_ledger::validate::{ledger_files, read_entry_file, validate_dir, Violation};

/// A throwaway probe-ledger directory per test: `<temp>/…/probes/migration/`
/// (unique suffix, cleaned up).
struct TempLedger {
    root: PathBuf,
    dir: PathBuf,
}

impl TempLedger {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "catherine-ledger-probe-roundtrip-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let dir = root.join("probes/migration");
        fs::create_dir_all(&dir).expect("temp probe ledger dir created");
        Self { root, dir }
    }

    /// Seal contents and write them as the ledger's files, in order.
    fn write(&self, contents: &[EntryContent]) {
        for content in contents {
            let entry = seal(content.clone()).expect("entry seals");
            let hex = entry.id.trim_start_matches("sha256:");
            let path = self
                .dir
                .join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
            fs::write(path, serde_json::to_string(&entry).expect("entry json"))
                .expect("ledger file written");
        }
    }
}

impl Drop for TempLedger {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// One probe-ledger entry — a kata record, with the pre-registered
/// catalog pin and the baseline arm the uniqueness claim derives from.
#[allow(clippy::too_many_arguments)]
fn probe_kata(
    seq: u32,
    kata_id: &str,
    control: bool,
    verdict: &str,
    expected: Vec<Expectation>,
    catalog_hash: Option<String>,
    unique: &[&str],
    baseline: Option<BaselineArm>,
) -> EntryContent {
    EntryContent {
        seq,
        kind: "kata".into(),
        recorded: "2026-11-13".into(),
        kata: Some(KataRecord {
            kata_id: kata_id.into(),
            name: if control {
                "Migration floor (negative control)".into()
            } else {
                kata_id.into()
            },
            control,
            series_report:
                ".espectacular/changes/add-domain-probe-validation/probes/migration/REPORT.md"
                    .into(),
            series_verdict: "killed".into(),
            fixture:
                ".espectacular/changes/add-domain-probe-validation/probes/migration/fixture.ct.json"
                    .into(),
            fixture_hash: "sha256:fixture".into(),
            expected,
            run: RunLog {
                commands: vec!["catherine check probes/migration/fixture.ct.json".into()],
                findings: vec![],
                probes: vec![],
            },
            comparison: Comparison {
                conventional_baseline: "row-count/checksum dry run".into(),
                fired: vec![],
                unique_detections: unique.iter().map(|s| s.to_string()).collect(),
                baseline,
            },
            expressibility_gap: vec![],
            discrepancies: vec![],
            procedure: vec![
                "derive unique detections as probe detections minus baseline detections".into(),
            ],
            catalog_hash,
            verdict: verdict.into(),
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
    }
}

/// A committed, version-pinned baseline artifact that was executed before
/// the probe run.
fn executed_arm() -> BaselineArm {
    BaselineArm {
        artifact: "tools/migration-dry-run@v1.2.0 (sha256:5e1c…)".into(),
        mode: BaselineMode::Executed,
        detections: vec!["lost-data:orders (row count mismatch)".into()],
    }
}

/// The probe's unique detections — faults the executed arm missed.
const UNIQUE: [&str; 2] = [
    "lost-data:orders.shipping_address",
    "non-invertibility:customers.email",
];

/// The GREEN probe ledger: the floor control self-negates at seq 1, then
/// `probe-migration` carries the pre-registered catalog pin, the executed
/// baseline arm, and the unique detections that arm did not see.
fn green_contents() -> Vec<EntryContent> {
    vec![
        probe_kata(
            1,
            "probe-migration-floor",
            true,
            "killed",
            vec![],
            None,
            &[],
            None,
        ),
        probe_kata(
            2,
            "probe-migration",
            false,
            "survived",
            catalog::probe_expectations("probe-migration"),
            catalog::probe_catalog_hash("probe-migration"),
            &UNIQUE,
            Some(executed_arm()),
        ),
    ]
}

#[test]
fn fixture_probe_ledger_validates_green_and_round_trips_its_ids() {
    let ledger = TempLedger::new("green");
    ledger.write(&green_contents());

    let valid = validate_dir(&ledger.dir).expect("the green probe ledger validates");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "the floor control and the probe entry");

    let control = katas[0].content.kata.as_ref().expect("control kata");
    assert!(control.control, "seq 1 is the negative control");
    assert_eq!(control.verdict, "killed", "the floor control self-negates");

    let probe = katas[1].content.kata.as_ref().expect("probe kata");
    assert!(!probe.control);
    assert_eq!(probe.kata_id, "probe-migration");
    assert_eq!(
        probe.catalog_hash,
        catalog::probe_catalog_hash("probe-migration"),
        "the entry pins the pre-registered catalog it ran against"
    );

    // The content address recomputes from the bytes on disk.
    let files = ledger_files(&ledger.dir).expect("ledger files listed");
    assert_eq!(files.len(), 2, "one file per written entry");
    for path in &files {
        let entry = read_entry_file(path).expect("the sealed entry re-reads");
        assert_eq!(
            entry_id(&entry.content).expect("content hashes"),
            entry.id,
            "{} content-addresses to its sealed id",
            path.display()
        );
    }

    // The recorded verdict is the derived one, from the recorded evidence.
    assert_eq!(
        derive_probe_verdict(
            "probe-migration",
            &probe.comparison.unique_detections,
            probe.comparison.baseline.as_ref()
        ),
        Some(Verdict::Survived),
        "an executed arm over a non-empty unique set derives survived"
    );
    assert_eq!(probe.verdict, "survived");
}

#[test]
fn baseline_less_and_cited_baseline_claims_are_refused() {
    // (a) a unique claim with no baseline arm.
    let no_baseline = TempLedger::new("no-baseline");
    let mut contents = green_contents();
    contents[1]
        .kata
        .as_mut()
        .expect("probe kata")
        .comparison
        .baseline = None;
    no_baseline.write(&contents);
    let violations = validate_dir(&no_baseline.dir).expect_err("no arm, no uniqueness");
    assert!(
        violations.contains(&Violation::UniqueWithoutBaseline { seq: 2 }),
        "expected UniqueWithoutBaseline at seq 2, got {violations:?}"
    );

    // (b) a cited-only arm cannot support `survived`.
    let cited_only = TempLedger::new("cited-survived");
    let mut contents = green_contents();
    let arm = contents[1]
        .kata
        .as_mut()
        .expect("probe kata")
        .comparison
        .baseline
        .as_mut()
        .expect("green arm");
    arm.mode = BaselineMode::Cited;
    cited_only.write(&contents);
    let violations = validate_dir(&cited_only.dir).expect_err("cited-only caps at inconclusive");
    assert!(
        violations.contains(&Violation::CitedBaselineCannotSurvive { seq: 2 }),
        "expected CitedBaselineCannotSurvive at seq 2, got {violations:?}"
    );

    // (c) a probe entry with no negative control.
    let no_control = TempLedger::new("no-control");
    no_control.write(&[probe_kata(
        1,
        "probe-migration",
        false,
        "survived",
        catalog::probe_expectations("probe-migration"),
        catalog::probe_catalog_hash("probe-migration"),
        &UNIQUE,
        Some(executed_arm()),
    )]);
    let violations = validate_dir(&no_control.dir).expect_err("an uncontrolled probe is refused");
    assert!(
        violations.contains(&Violation::NegativeControlMissing { seq: 1 }),
        "expected NegativeControlMissing at seq 1, got {violations:?}"
    );

    // (d) a control that did not self-negate.
    let surviving_control = TempLedger::new("control-survived");
    let mut contents = green_contents();
    contents[0].kata.as_mut().expect("control kata").verdict = "survived".into();
    surviving_control.write(&contents);
    let violations =
        validate_dir(&surviving_control.dir).expect_err("a control that survived is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { seq: 1, .. })),
        "expected ControlDidNotSelfNegate at seq 1, got {violations:?}"
    );
}

#[test]
fn stale_catalog_hash_after_the_catalog_edit_is_refused() {
    // The entry pinned a catalog that is not the committed one (the
    // catalog changed after the entry was sealed, or the entry was edited).
    let stale = TempLedger::new("stale-catalog-hash");
    let mut contents = green_contents();
    contents[1].kata.as_mut().expect("probe kata").catalog_hash =
        Some(format!("sha256:{}", "0".repeat(64)));
    stale.write(&contents);
    let violations = validate_dir(&stale.dir).expect_err("a stale catalog pin is refused");
    assert!(
        violations.contains(&Violation::CatalogHashMismatch { seq: 2 }),
        "expected CatalogHashMismatch at seq 2, got {violations:?}"
    );

    // A registered probe with expectations but no pin: the entry was sealed
    // before the catalog, so it cannot be retrofitted to it.
    let unpinned = TempLedger::new("unpinned-catalog");
    let mut contents = green_contents();
    contents[1].kata.as_mut().expect("probe kata").catalog_hash = None;
    unpinned.write(&contents);
    let violations =
        validate_dir(&unpinned.dir).expect_err("an unpinned expectation set is refused");
    assert!(
        violations.contains(&Violation::CatalogHashMismatch { seq: 2 }),
        "expected CatalogHashMismatch at seq 2, got {violations:?}"
    );
}

#[test]
fn superseding_a_probe_entry_keeps_the_ledger_append_only() {
    let ledger = TempLedger::new("supersede");
    let mut contents = green_contents();
    let original = seal(contents[1].clone()).expect("the original probe entry seals");

    // The correction is a new entry naming the old id — the cited arm caps
    // the verdict at inconclusive, with the same unique set and catalog.
    let cited = BaselineArm {
        artifact: "docs/migration-dry-run.md (pinned 2026-10-30)".into(),
        mode: BaselineMode::Cited,
        detections: executed_arm().detections,
    };
    let mut correction = probe_kata(
        3,
        "probe-migration",
        false,
        "inconclusive",
        catalog::probe_expectations("probe-migration"),
        catalog::probe_catalog_hash("probe-migration"),
        &UNIQUE,
        Some(cited.clone()),
    );
    correction.supersedes = Some(original.id.clone());
    contents.push(correction);
    ledger.write(&contents);

    let valid = validate_dir(&ledger.dir).expect("supersession is the correction path");
    let effective_entries = valid.kata_entries();
    let seqs: Vec<u32> = effective_entries
        .iter()
        .map(|entry| entry.content.seq)
        .collect();
    assert_eq!(seqs, vec![1, 3], "the superseded entry is not effective");

    let unique: Vec<String> = UNIQUE.iter().map(|s| s.to_string()).collect();
    let effective = effective_entries[1]
        .content
        .kata
        .as_ref()
        .expect("effective probe kata");
    assert_eq!(
        effective.comparison.unique_detections, unique,
        "the correction records the same unique set"
    );
    assert_eq!(
        derive_probe_verdict("probe-migration", &unique, Some(&cited)),
        Some(Verdict::Inconclusive),
        "a cited-only arm derives inconclusive, matching the recorded correction"
    );
    assert_eq!(effective.verdict, "inconclusive");
}

#[test]
fn kata_ledger_is_byte_identical_after_probe_recording() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let kata_ledger = root.join("ledger");
    let bin = env!("CARGO_BIN_EXE_ledger-check");

    let before = run_ledger_check(bin, &kata_ledger);
    let before_digests = ledger_digests(&kata_ledger);
    assert!(
        !before_digests.is_empty(),
        "the committed kata ledger has sealed entries"
    );

    let probe_ledger = TempLedger::new("kata-byte-identity");
    probe_ledger.write(&green_contents());
    validate_dir(&probe_ledger.dir).expect("the green probe ledger validates");

    let after = run_ledger_check(bin, &kata_ledger);
    let after_digests = ledger_digests(&kata_ledger);

    assert_eq!(
        before.0, after.0,
        "ledger-check reports the same derived rollup after probe recording"
    );
    assert_eq!(before.1, after.1, "ledger-check exits the same way");
    assert_eq!(
        before_digests, after_digests,
        "no sealed kata ledger file is rewritten by probe recording"
    );
}

/// Run `ledger-check` over the directory: (stdout, exit code).
fn run_ledger_check(bin: &str, dir: &Path) -> (String, Option<i32>) {
    let output = std::process::Command::new(bin)
        .arg(dir)
        .output()
        .expect("ledger-check runs");
    (
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        output.status.code(),
    )
}

/// The content digests of every sealed `*.json` file in the ledger
/// directory, keyed by file name.
fn ledger_digests(dir: &Path) -> BTreeMap<String, String> {
    let mut digests = BTreeMap::new();
    for entry in fs::read_dir(dir).expect("ledger dir readable") {
        let path = entry.expect("dir entry").path();
        if !path.extension().is_some_and(|ext| ext == "json") || !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .expect("ledger file has a name")
            .to_string_lossy()
            .into_owned();
        digests.insert(
            name,
            sha256_digest(&fs::read(&path).expect("ledger file readable")),
        );
    }
    digests
}
