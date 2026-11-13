//! Purpose: shared fixtures for the domain-probe ledger tests
//! (`add-domain-probe-validation`, task 4) — a throwaway probe ledger
//! directory, the GREEN two-entry contents (floor control + probe
//! entry), and the digest/CLI helpers the byte-identity proof needs.
//!
//! Rationale: the round-trip, refusal, and byte-identity proofs assert
//! the same GREEN ledger from different angles; one copy of the fixture
//! keeps the ledger shape honest and each test file under the
//! maintainability caps.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use catherine_core::hash::sha256_digest;
use catherine_ledger::catalog;
use catherine_ledger::entry::{
    seal, BaselineArm, BaselineMode, Comparison, EntryContent, Expectation, KataRecord, RunLog,
};

/// A throwaway probe-ledger directory per test: `<temp>/…/probes/migration/`
/// (unique suffix, cleaned up).
pub struct TempLedger {
    pub root: PathBuf,
    pub dir: PathBuf,
}

impl TempLedger {
    pub fn new(tag: &str) -> Self {
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
    pub fn write(&self, contents: &[EntryContent]) {
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
pub fn probe_kata(
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
pub fn executed_arm() -> BaselineArm {
    BaselineArm {
        artifact: "tools/migration-dry-run@v1.2.0 (sha256:5e1c…)".into(),
        mode: BaselineMode::Executed,
        detections: vec!["lost-data:orders (row count mismatch)".into()],
    }
}

/// The probe's unique detections — faults the executed arm missed.
pub const UNIQUE: [&str; 2] = [
    "lost-data:orders.shipping_address",
    "non-invertibility:customers.email",
];

/// The GREEN probe ledger: the floor control self-negates at seq 1, then
/// `probe-migration` carries the pre-registered catalog pin, the executed
/// baseline arm, and the unique detections that arm did not see.
pub fn green_contents() -> Vec<EntryContent> {
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

/// Run `ledger-check` over the directory: (stdout, exit code).
pub fn run_ledger_check(bin: &str, dir: &Path) -> (String, Option<i32>) {
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
pub fn ledger_digests(dir: &Path) -> BTreeMap<String, String> {
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
