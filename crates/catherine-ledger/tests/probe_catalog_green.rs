//! Purpose: the GREEN gate for the probe catalogs (tasks.md task 3) —
//! the pre-registered expectation rows are content-addressed, the
//! validator refuses a probe entry whose `catalog_hash` names no
//! committed catalog, and accepts the matching one. Assertions are
//! structural (recomputed digests, named rejections), never snapshots.

mod support;

use catherine_core::canonical::write_value;
use catherine_core::hash::sha256_digest;
use catherine_ledger::catalog::{is_probe, probe_catalog_hash, probe_expectations};
use catherine_ledger::entry::Expectation;
use catherine_ledger::validate::{validate_dir, Violation};

use support::{present, probe_kata, ProbeSpec, TempLedger, TestError};

/// The documented digests' input, recomputed locally: canonical bytes
/// through the kernel writer, sha256 on top.
fn recomputed_hash(rows: &[Expectation]) -> Result<String, TestError> {
    let value = serde_json::to_value(rows)?;
    Ok(sha256_digest(write_value(&value).as_bytes()))
}

/// The registered probes' hashes are content addresses of their rows —
/// recomputable from the catalog, never string-formatted.
#[test]
fn catalog_hash_is_content_addressed() -> Result<(), TestError> {
    for probe_id in [
        "probe-migration",
        "probe-rule-lens-extracted",
        "probe-naturality",
        "probe-composition",
        "probe-coproduct",
    ] {
        let hash = present(
            probe_catalog_hash(probe_id),
            "registered probe has a catalog",
        )?;
        let recompute = recomputed_hash(&probe_expectations(probe_id))?;
        assert_eq!(hash, recompute, "catalog hash for {probe_id}");
        let hex = hash.trim_start_matches("sha256:");
        assert_eq!(hex.len(), 64, "sha256 digest length for {probe_id}");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "hex digest for {probe_id}"
        );
    }
    Ok(())
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

    let naturality = probe_expectations("probe-naturality");
    let faults: Vec<&str> = naturality.iter().map(|row| row.fault.as_str()).collect();
    assert_eq!(faults, vec!["floor-sync-defect", "wrong-question"]);

    for row in migration
        .iter()
        .chain(rule_lens.iter())
        .chain(naturality.iter())
    {
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
fn validator_refuses_a_catalog_hash_mismatch() -> Result<(), TestError> {
    let ledger = TempLedger::new("mismatch")?;
    ledger.write(&[
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
            expected: probe_expectations("probe-migration"),
            catalog_hash: Some("sha256:deadbeef".into()),
            ..Default::default()
        }),
    ])?;
    let Err(violations) = validate_dir(&ledger.dir) else {
        return Err("a wrong catalog hash is refused: the ledger validated".into());
    };
    assert!(
        violations.contains(&Violation::CatalogHashMismatch { seq: 2 }),
        "expected CatalogHashMismatch at seq 2, got {violations:?}"
    );
    Ok(())
}

/// A registered probe carrying pre-registered rows must name the
/// catalog hash it ran against; `None` is the refusal.
#[test]
fn validator_refuses_a_registered_probe_without_a_catalog_hash() -> Result<(), TestError> {
    let ledger = TempLedger::new("missing-hash")?;
    ledger.write(&[
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
            expected: probe_expectations("probe-migration"),
            catalog_hash: None,
            ..Default::default()
        }),
    ])?;
    let Err(violations) = validate_dir(&ledger.dir) else {
        return Err("a probe without its catalog hash is refused: the ledger validated".into());
    };
    assert!(
        violations.contains(&Violation::CatalogHashMismatch { seq: 2 }),
        "expected CatalogHashMismatch at seq 2, got {violations:?}"
    );
    Ok(())
}

/// The matching pre-registration validates: the entry names the
/// committed catalog it ran against.
#[test]
fn validator_accepts_a_probe_entry_with_the_matching_catalog_hash() -> Result<(), TestError> {
    let ledger = TempLedger::new("matching-hash")?;
    ledger.write(&[
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
            expected: probe_expectations("probe-migration"),
            catalog_hash: probe_catalog_hash("probe-migration"),
            ..Default::default()
        }),
    ])?;
    let validated = validate_dir(&ledger.dir);
    assert!(
        validated.is_ok(),
        "a probe naming its committed catalog passes, got {:?}",
        validated.err()
    );
    Ok(())
}
