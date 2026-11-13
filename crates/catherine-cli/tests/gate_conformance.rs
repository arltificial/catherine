//! Purpose: pin the pre-commit complexity gate (catherine-x94 aftermath) —
//! the fail-closed `pretender check --mode gate` must be wired in
//! `lefthook.yml` OUTSIDE the ddl-managed block, and no gate baseline may
//! exist. Both facts are invisible to a normal test run but a silent loss
//! of either reopens the loophole that let a 523-line file through a
//! commit: the managed gate uses `--diff-only`, which diffs the unstaged
//! tree and therefore scanned nothing once everything was staged.
//!
//! Responsibilities: read the repo's `lefthook.yml`, assert the strict
//! command exists after the pre-commit managed block ends, assert the
//! managed region itself still carries only the blind `--diff-only` form
//! (so the strict gate cannot have been added by regeneration), and assert
//! `pretender-baseline.json` is absent.
//!
//! Rationale: the repo's precedent for a gate that can be silently
//! disarmed is `hook-swallow-guard` (a real gate plus a test that notices
//! its removal). A guard in the same spirit keeps ddl/pretender
//! regeneration from quietly restoring the blind gate.

use std::path::Path;

/// The repo root, resolved from this crate's manifest directory.
fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the crate lives two levels under the repo root")
}

fn lefthook() -> String {
    std::fs::read_to_string(repo_root().join("lefthook.yml")).expect("lefthook.yml is committed")
}

#[test]
fn the_strict_gate_is_wired_outside_the_managed_block() {
    let config = lefthook();
    let (managed, unmanaged) = config
        .split_once("# ddl:managed:end (pre-commit)")
        .expect("the pre-commit managed block is delimited");
    assert!(
        managed.contains("pretender check --diff-only --mode gate"),
        "the managed block keeps ddl's blind form; the strict gate must not live there"
    );
    assert!(
        unmanaged.contains("pretender-strict:")
            && unmanaged.contains("run: pretender check --mode gate"),
        "the fail-closed gate must sit after the managed block, or regeneration removes it"
    );
    let threaded: Vec<&str> = unmanaged
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("run:") && line.contains("pretender check"))
        .collect();
    assert!(
        !threaded.iter().any(|line| line.contains("--diff-only")),
        "the strict gate's run line must scan the whole tree, never a diff: {threaded:?}"
    );
}

#[test]
fn the_gate_is_fail_closed_with_no_baseline() {
    let baseline = repo_root().join("pretender-baseline.json");
    assert!(
        !baseline.exists(),
        "a pretender baseline silences gate mode for every adopted violation; \
         it must not exist (deleting it is the fix, not adopting more)"
    );
}
