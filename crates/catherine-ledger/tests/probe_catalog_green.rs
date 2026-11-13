//! Purpose: the GREEN gate for the probe catalogs (tasks.md task 3) —
//! the pre-registered expectation rows are content-addressed, the
//! validator refuses a probe entry whose `catalog_hash` names no
//! committed catalog, and accepts the matching one. Assertions are
//! structural (recomputed digests, named rejections), never snapshots.

use std::fs;
use std::path::PathBuf;

use catherine_core::canonical::write_value;
use catherine_core::hash::sha256_digest;
use catherine_ledger::catalog::{is_probe, probe_catalog_hash, probe_expectations};
use catherine_ledger::entry::{seal, Comparison, EntryContent, Expectation, KataRecord, RunLog};
use catherine_ledger::validate::{validate_dir, Violation};

/// A throwaway ledger directory per test (unique suffix, cleaned up).
struct TempLedger(PathBuf);

impl TempLedger {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "catherine-ledger-probe-catalog-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp ledger dir created");
        Self(dir)
    }

    /// Seal contents and write them as the ledger's files, in order.
    fn write(&self, contents: &[EntryContent]) {
        for content in contents {
            let entry = seal(content.clone()).expect("entry seals");
            let hex = entry.id.trim_start_matches("sha256:");
            let path = self
                .0
                .join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
            fs::write(path, serde_json::to_string(&entry).expect("entry json"))
                .expect("ledger file written");
        }
    }
}

impl Drop for TempLedger {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A minimal kata record — the fields the validator reads. Probe ids
/// ride the same record shape with a catalog reference.
fn kata(
    seq: u32,
    kata_id: &str,
    control: bool,
    verdict: &str,
    expected: Vec<Expectation>,
    catalog_hash: Option<String>,
) -> EntryContent {
    EntryContent {
        seq,
        kind: "kata".into(),
        recorded: "2026-11-13".into(),
        kata: Some(KataRecord {
            kata_id: kata_id.into(),
            name: if control {
                "floor control".into()
            } else {
                kata_id.into()
            },
            control,
            series_report: "openspec/changes/add-domain-probe-validation/proposal.md".into(),
            series_verdict: "pre-registered".into(),
            fixture: "fixtures/probes/migration.ct.json".into(),
            fixture_hash: "sha256:fixture".into(),
            expected,
            run: RunLog {
                commands: vec![],
                findings: vec![],
                probes: vec![],
            },
            comparison: Comparison {
                conventional_baseline: "row-count/checksum dry run".into(),
                fired: vec![],
                unique_detections: vec![],
                baseline: None,
            },
            expressibility_gap: vec![],
            discrepancies: vec![],
            procedure: vec![],
            catalog_hash,
            verdict: verdict.into(),
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
    }
}

/// The documented digests' input, recomputed locally: canonical bytes
/// through the kernel writer, sha256 on top.
fn recomputed_hash(rows: &[Expectation]) -> String {
    let value = serde_json::to_value(rows).expect("rows serialize");
    sha256_digest(write_value(&value).as_bytes())
}

/// The registered probes' hashes are content addresses of their rows —
/// recomputable from the catalog, never string-formatted.
#[test]
fn catalog_hash_is_content_addressed() {
    for probe_id in ["probe-migration", "probe-rule-lens-extracted"] {
        let hash = probe_catalog_hash(probe_id).expect("registered probe has a catalog");
        let expected = recomputed_hash(&probe_expectations(probe_id));
        assert_eq!(hash, expected, "catalog hash for {probe_id}");
        let hex = hash.trim_start_matches("sha256:");
        assert_eq!(hex.len(), 64, "sha256 digest length for {probe_id}");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "hex digest for {probe_id}"
        );
    }
}

/// An unregistered id has no catalog: no rows, no hash, not a probe.
#[test]
fn unregistered_probe_has_no_catalog() {
    assert_eq!(probe_catalog_hash("probe-job-board"), None);
    assert!(probe_expectations("probe-job-board").is_empty());
    assert!(!is_probe("probe-job-board"));
}

/// The rows are the pre-registration: the proposal's faults, each with
/// a stated mapping and a catherine class the ledger vocabulary admits.
#[test]
fn registered_probes_carry_their_pre_registered_rows() {
    let migration = probe_expectations("probe-migration");
    let faults: Vec<&str> = migration.iter().map(|row| row.fault.as_str()).collect();
    assert_eq!(
        faults,
        vec!["lost-data", "non-invertibility", "floor-visible-loss"]
    );

    let rule_lens = probe_expectations("probe-rule-lens-extracted");
    let faults: Vec<&str> = rule_lens.iter().map(|row| row.fault.as_str()).collect();
    assert_eq!(
        faults,
        vec![
            "gap",
            "overlap",
            "shadow",
            "derivable",
            "floor-near-trivial"
        ]
    );

    for row in migration.iter().chain(rule_lens.iter()) {
        assert!(
            !row.detail.trim().is_empty(),
            "row {} states its mapping",
            row.fault
        );
        assert!(
            ["finding", "construction", "gap"].contains(&row.catherine_class.as_str()),
            "row {} carries a known catherine class, got {}",
            row.fault,
            row.catherine_class
        );
    }
}

/// A `catalog_hash` that is not the committed catalog's digest is
/// refused — the entry cannot claim a pre-registration it did not run
/// against.
#[test]
fn validator_refuses_a_catalog_hash_mismatch() {
    let ledger = TempLedger::new("mismatch");
    ledger.write(&[
        kata(1, "probe-migration-floor", true, "killed", vec![], None),
        kata(
            2,
            "probe-migration",
            false,
            "survived",
            probe_expectations("probe-migration"),
            Some("sha256:deadbeef".into()),
        ),
    ]);
    let violations = validate_dir(&ledger.0).expect_err("a wrong catalog hash is refused");
    assert!(
        violations.contains(&Violation::CatalogHashMismatch { seq: 2 }),
        "expected CatalogHashMismatch at seq 2, got {violations:?}"
    );
}

/// A registered probe carrying pre-registered rows must name the
/// catalog hash it ran against; `None` is the refusal.
#[test]
fn validator_refuses_a_registered_probe_without_a_catalog_hash() {
    let ledger = TempLedger::new("missing-hash");
    ledger.write(&[
        kata(1, "probe-migration-floor", true, "killed", vec![], None),
        kata(
            2,
            "probe-migration",
            false,
            "survived",
            probe_expectations("probe-migration"),
            None,
        ),
    ]);
    let violations =
        validate_dir(&ledger.0).expect_err("a probe without its catalog hash is refused");
    assert!(
        violations.contains(&Violation::CatalogHashMismatch { seq: 2 }),
        "expected CatalogHashMismatch at seq 2, got {violations:?}"
    );
}

/// The matching pre-registration validates: the entry names the
/// committed catalog it ran against.
#[test]
fn validator_accepts_a_probe_entry_with_the_matching_catalog_hash() {
    let ledger = TempLedger::new("matching-hash");
    ledger.write(&[
        kata(1, "probe-migration-floor", true, "killed", vec![], None),
        kata(
            2,
            "probe-migration",
            false,
            "survived",
            probe_expectations("probe-migration"),
            probe_catalog_hash("probe-migration"),
        ),
    ]);
    let validated = validate_dir(&ledger.0);
    assert!(
        validated.is_ok(),
        "a probe naming its committed catalog passes, got {:?}",
        validated.err()
    );
}
