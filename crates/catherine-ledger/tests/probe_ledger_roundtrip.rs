//! Purpose: the GREEN proof for the domain-probe ledger
//! (`add-domain-probe-validation`, tasks.md task 4) — a probe directory
//! under `probes/migration/` validates, every sealed id recomputes from
//! the bytes on disk, the harness derives the recorded verdict, and a
//! retrofitted catalog pin is refused.
//!
//! Responsibilities: build the GREEN fixture (floor control +
//! `probe-migration` entry with its pre-registered catalog hash and an
//! executed baseline arm), round-trip each written file through
//! `validate::read_entry_file` and `entry_id`, and pin the catalog-hash
//! refusals that make a retrofitted expectation unresolvable.
//!
//! Rationale: a probe entry is evidence only if its content address, its
//! catalog pin, and its derived verdict all survive a re-read from disk —
//! the same discipline the kata ledger enforces.

mod support;

use catherine_ledger::catalog;
use catherine_ledger::entry::{entry_id, Verdict};
use catherine_ledger::harness::derive_probe_verdict;
use catherine_ledger::validate::{ledger_files, read_entry_file, validate_dir, Violation};

use support::{green_contents, TempLedger};

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
