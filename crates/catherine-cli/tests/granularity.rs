//! Purpose: the granularity lint's end-to-end contract (ticket
//! `catherine-kw9.3`) — `catherine check` emits the stored-granularity
//! findings over the real binary, the negative controls stay silent.
//!
//! Responsibilities: drive `catherine check` exactly as a user does —
//! the bogart-derived over-cap fixture exits nonzero with a
//! `granularity` finding; the re-model `clean` fixture exits 0; every
//! kata fixture and the committed bogart slice (the wave's measured
//! negative controls) produce zero `granularity` findings even when
//! they carry findings of other kinds.
//!
//! Rationale: the lint's whole measured motivation (docs/USING.md, the
//! 2/69 bogart pilot) is that ordinary well-grained models must never
//! drown in granularity noise, so the kata corpus doubles as the
//! calibration's regression pin: a threshold change that fires on a
//! kata fails here, and recalibration — never filename special-casing —
//! is the only fix.

use std::path::Path;
use std::process::{Command, Output};

fn run_check(path: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(["check", path, "--json"])
        .output()
        .expect("spawn catherine binary")
}

fn kinds(out: &Output) -> Vec<String> {
    let value: serde_json::Value = match serde_json::from_slice(&out.stdout) {
        Ok(value) => value,
        // A run that failed before findings (a parse error envelope on
        // stderr, an empty stdout) carried no granularity finding.
        Err(_) => return Vec::new(),
    };
    value["data"]["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .map(|f| f["kind"].as_str().expect("kind").to_string())
        .collect()
}

fn fixture(name: &str) -> String {
    format!("{}/../../fixtures/granularity/{name}", env!("CARGO_MANIFEST_DIR"))
}

/// The over-cap fixture, derived from the bogart pilot's model shape
/// (many unstored planner-intent arrows over a handful of rows), must
/// exit nonzero carrying at least one `granularity` finding.
#[test]
fn over_cap_fires_a_granularity_finding() {
    let out = run_check(&fixture("over_cap.ct.json"));
    assert_ne!(
        out.status.code(),
        Some(0),
        "over_cap must exit nonzero: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        kinds(&out).iter().any(|kind| kind == "granularity"),
        "over_cap must carry a granularity finding: {:?}",
        kinds(&out)
    );
}

/// The re-model fixture (one arrow per stored reference, the bogart
/// slice's shape) stays silent: exit 0, no findings at all.
#[test]
fn clean_stays_silent() {
    let out = run_check(&fixture("clean.ct.json"));
    assert_eq!(out.status.code(), Some(0), "clean must exit 0");
    assert!(kinds(&out).is_empty(), "clean must carry no findings");
}

/// Every kata fixture — the calibration's negative controls — produces
/// zero granularity findings. Fault/probe variants may exit nonzero for
/// their own defects; only the granularity kind is asserted.
#[test]
fn kata_corpus_stays_silent() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/katas");
    let mut files: Vec<_> = walk(&dir);
    files.sort();
    assert!(files.len() > 20, "the kata corpus is present");
    for file in files {
        let out = run_check(&file);
        let gran: Vec<_> = kinds(&out).into_iter().filter(|k| k == "granularity").collect();
        assert!(
            gran.is_empty(),
            "{file} fired a granularity finding — recalibrate the thresholds, never special-case the filename"
        );
    }
}

/// The committed bogart slice — the wave's real measured negative
/// control (2 arrows, 289+3+1 rows) — stays silent.
#[test]
fn bogart_slice_stays_silent() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/probes/granularity-bogart/fts-anchor-slice.ct.json");
    let out = run_check(&path.to_str().unwrap());
    assert_eq!(out.status.code(), Some(0), "the bogart slice is clean");
    assert!(!kinds(&out).iter().any(|k| k == "granularity"));
}

/// Every `.ct.json` under `dir`, recursively (order is the caller's
/// concern — it sorts).
fn walk(dir: &Path) -> Vec<String> {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .expect("kata dir readable")
        .flatten()
        .collect();
    let (dirs, plain): (Vec<_>, Vec<_>) =
        entries.into_iter().partition(|e| e.path().is_dir());
    dirs.iter()
        .flat_map(|e| walk(&e.path()))
        .chain(
            plain
                .iter()
                .filter(|e| {
                    e.path().extension() == Some(std::ffi::OsStr::new("json"))
                })
                .map(|e| e.path().to_string_lossy().into_owned()),
        )
        .collect()
}
