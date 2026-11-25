//! Purpose: shared fixtures for the domain-probe ledger tests
//! (`add-domain-probe-validation`, task 4) — a throwaway probe ledger
//! directory, the GREEN two-entry contents (floor control + probe
//! entry), and the digest/CLI helpers the byte-identity proof needs.
//!
//! Rationale: the round-trip, refusal, and byte-identity proofs assert
//! the same GREEN ledger from different angles; one copy of the fixture
//! keeps the ledger shape honest and each test file under the
//! maintainability caps. Fallible helpers return `Result` so tests
//! propagate with `?` instead of unwrapping; `probe_kata` takes a
//! [`ProbeSpec`] so the record's variation axes stay named instead of a
//! positional pile of arguments.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use catherine_core::hash::sha256_digest;
use catherine_ledger::catalog;
use catherine_ledger::entry::{
    seal, BaselineArm, BaselineMode, Comparison, Derivation, EntryContent, Expectation,
    KataIdentity, KataRecord, Provenance, RunLog,
};

/// The error type every fallible test helper and test body uses.
pub type TestError = Box<dyn std::error::Error>;

/// Unwrap an `Option` as a named test failure: the `?`-propagating
/// replacement for `Option::unwrap` in test bodies.
pub fn present<T>(value: Option<T>, what: &str) -> Result<T, TestError> {
    value.ok_or_else(|| format!("{what} absent").into())
}

/// A throwaway probe-ledger directory per test: `<temp>/…/probes/migration/`
/// (unique suffix, cleaned up).
pub struct TempLedger {
    pub root: PathBuf,
    pub dir: PathBuf,
}

impl TempLedger {
    pub fn new(tag: &str) -> Result<Self, TestError> {
        let root = std::env::temp_dir().join(format!(
            "catherine-ledger-probe-roundtrip-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let dir = root.join("probes/migration");
        fs::create_dir_all(&dir)?;
        Ok(Self { root, dir })
    }

    /// Seal contents and write them as the ledger's files, in order.
    pub fn write(&self, contents: &[EntryContent]) -> Result<(), TestError> {
        for content in contents {
            let entry = seal(content.clone())?;
            let hex = entry.id.trim_start_matches("sha256:");
            let path = self
                .dir
                .join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
            fs::write(path, serde_json::to_string(&entry)?)?;
        }
        Ok(())
    }
}

impl Drop for TempLedger {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The variation axes of one probe-ledger kata entry: the seq, which
/// probe it claims, whether it is the floor control, its verdict, the
/// pre-registered catalog pin, and the uniqueness claim (unique
/// detections + the baseline arm they derive from). Fields default to
/// the empty/unclaimed shape, so a call site names only what varies.
#[derive(Default)]
pub struct ProbeSpec {
    pub seq: u32,
    pub kata_id: &'static str,
    pub control: bool,
    pub verdict: &'static str,
    pub expected: Vec<Expectation>,
    pub catalog_hash: Option<String>,
    pub unique: &'static [&'static str],
    pub baseline: Option<BaselineArm>,
}

/// One probe-ledger entry — a kata record, with the pre-registered
/// catalog pin and the baseline arm the uniqueness claim derives from.
pub fn probe_kata(spec: ProbeSpec) -> EntryContent {
    let ProbeSpec {
        seq,
        kata_id,
        control,
        verdict,
        expected,
        catalog_hash,
        unique,
        baseline,
    } = spec;
    EntryContent {
        seq,
        kind: "kata".into(),
        recorded: "2026-11-13".into(),
        kata: Some(KataRecord {
            identity: KataIdentity {
                kata_id: kata_id.into(),
                name: if control {
                    "Migration floor (negative control)".into()
                } else {
                    kata_id.into()
                },
                control,
            },
            provenance: Provenance {
                series_report:
                    ".espectacular/changes/add-domain-probe-validation/probes/migration/REPORT.md"
                        .into(),
                series_verdict: "killed".into(),
                fixture:
                    ".espectacular/changes/add-domain-probe-validation/probes/migration/fixture.ct.json"
                        .into(),
                fixture_hash: "sha256:fixture".into(),
            },
            expected,
            run: RunLog {
                commands: vec!["catherine check probes/migration/fixture.ct.json".into()],
                findings: vec![],
                probes: vec![],
            },
            derivation: Derivation {
                comparison: Comparison {
                    conventional_baseline: "row-count/checksum dry run".into(),
                    fired: vec![],
                    unique_detections: unique.iter().map(|s| s.to_string()).collect(),
                    baseline,
                },
                procedure: vec![
                    "derive unique detections as probe detections minus baseline detections"
                        .into(),
                ],
                verdict: verdict.into(),
            },
            expressibility_gap: vec![],
            discrepancies: vec![],
            catalog_hash,
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
        decision: None,
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
        probe_kata(ProbeSpec {
            seq: 1,
            kata_id: "probe-migration-floor",
            control: true,
            verdict: "killed",
            ..Default::default()
        }),
        probe_kata(ProbeSpec {
            seq: 2,
            kata_id: "probe-migration",
            verdict: "survived",
            expected: catalog::probe_expectations("probe-migration"),
            catalog_hash: catalog::probe_catalog_hash("probe-migration"),
            unique: &UNIQUE,
            baseline: Some(executed_arm()),
            ..Default::default()
        }),
    ]
}

/// Run `ledger-check` over the directory: (stdout, exit code).
pub fn run_ledger_check(bin: &str, dir: &Path) -> Result<(String, Option<i32>), TestError> {
    let output = std::process::Command::new(bin).arg(dir).output()?;
    Ok((String::from_utf8(output.stdout)?, output.status.code()))
}

/// The content digests of every sealed `*.json` file in the ledger
/// directory, keyed by file name.
pub fn ledger_digests(dir: &Path) -> Result<BTreeMap<String, String>, TestError> {
    let mut digests = BTreeMap::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if !path.extension().is_some_and(|ext| ext == "json") || !path.is_file() {
            continue;
        }
        let name = present(
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            "ledger file name",
        )?;
        let bytes = fs::read(&path)?;
        digests.insert(name, sha256_digest(&bytes));
    }
    Ok(digests)
}
