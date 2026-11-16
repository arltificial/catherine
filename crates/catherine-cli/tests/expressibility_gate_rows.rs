//! Purpose: hold the expressibility-claim docs gate
//! (`scripts/check-expressibility-claims.sh`) to the registry it protects —
//! every row's *Claimed limitation* cell must be a phrase the gate actually
//! detects, and the openspec `## Close note` sections must actually be
//! scanned.
//!
//! Responsibilities: parse `docs/ref/expressibility-dispositions.md`, feed
//! each row's claim cell through `CLAIM_GATE_SYNTHETIC` and require exit 1
//! (un-cited) and exit 0 once the same sentence cites the row id; build a
//! temporary repository root through the test-only `CLAIM_GATE_ROOT` hook and
//! assert a claim inside a `## Close note` section (including its nested
//! sub-sections) is rejected while sibling sections stay out of scope, and
//! that the same claim accepted when cited; and assert the plain real-tree
//! invocation exits 0.
//!
//! Rationale: the gate's previous phrase list caught only 4 of the 12
//! registered claims, so a real limitation could be restated in the docs and
//! pass — review finding HIGH-003. The close-note half matched zero files
//! because this repository writes close notes as `## Close note` sections
//! rather than `close-note*.md` files — review finding HIGH-002. Feeding the
//! registry's own wording through the gate makes phrase coverage
//! non-tautological: adding a row with unfamiliar phrasing fails this test
//! until the fixed, documented list learns it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The disposition table this test reads; never mirrored in Rust.
const TABLE: &str = "docs/ref/expressibility-dispositions.md";

/// The docs gate under test.
const CLAIM_GATE: &str = "scripts/check-expressibility-claims.sh";

/// One table row (the two cells this test enforces).
struct Row {
    id: String,
    claim: String,
}

/// The repository root: this binary lives at `crates/catherine-cli`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

/// Read and parse the disposition table from disk.
fn table_rows() -> Vec<Row> {
    let path = repo_root().join(TABLE);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    text.lines().filter_map(parse_row).collect()
}

/// Parse one `| … |` row, or `None` for the header and separator lines.
fn parse_row(line: &str) -> Option<Row> {
    if !line.trim_start().starts_with('|') {
        return None;
    }
    let cells: Vec<&str> = line.split('|').map(str::trim).collect();
    assert_eq!(
        cells.len(),
        8,
        "malformed disposition row (expected six cells): {line}"
    );
    let id = cells[1];
    if id.is_empty() || id == "Case id" || id.bytes().all(|byte| byte == b'-') {
        return None;
    }
    Some(Row {
        id: id.to_string(),
        claim: cells[2].to_string(),
    })
}

/// Run the gate, optionally pointed at `root` and/or fed one synthetic line.
fn run_gate(root: Option<&Path>, synthetic: Option<&str>) -> Output {
    let script = repo_root().join(CLAIM_GATE);
    let mut command = Command::new("bash");
    command.arg(&script);
    if let Some(root) = root {
        command.env("CLAIM_GATE_ROOT", root);
    }
    if let Some(line) = synthetic {
        command.env("CLAIM_GATE_SYNTHETIC", line);
    }
    command
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", script.display()))
}

/// Both streams, so a rejection message can be asserted wherever it lands.
fn gate_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// A throwaway repository root: the real table plus an empty `openspec` tree.
fn temp_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!("catherine-claim-gate-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(root.join("docs/ref")).expect("temp docs dir");
    std::fs::create_dir_all(root.join("openspec/changes/synthetic")).expect("temp openspec dir");
    std::fs::copy(repo_root().join(TABLE), root.join(TABLE)).expect("copy disposition table");
    root
}

/// Every registered *Claimed limitation* is detected when left un-cited, and
/// the gate stays green on the real tree.
#[test]
fn every_claim_cell_is_rejected_without_a_row_id() {
    let clean = run_gate(None, None);
    assert!(
        clean.status.success(),
        "the gate fails on the real tree:\n{}",
        gate_text(&clean)
    );

    let rows = table_rows();
    assert!(
        rows.len() >= 12,
        "the disposition table lost rows: {} parsed",
        rows.len()
    );

    for row in &rows {
        let rejected = run_gate(None, Some(&row.claim));
        assert_eq!(
            rejected.status.code(),
            Some(1),
            "row `{}`: the claim cell is not detected as a limitation claim:\n{}",
            row.id,
            gate_text(&rejected)
        );
        assert!(
            gate_text(&rejected).contains("no disposition row id"),
            "row `{}`: the rejection does not name the missing row id",
            row.id
        );

        let cited = format!("{} (row `{}`)", row.claim, row.id);
        let accepted = run_gate(None, Some(&cited));
        assert!(
            accepted.status.success(),
            "row `{}`: the same claim is rejected once cited:\n{}",
            row.id,
            gate_text(&accepted)
        );
    }
}

/// A claim in a `## Close note` section is scanned (nested sub-sections too),
/// lines outside such a section are not, and citation clears the claim.
#[test]
fn close_note_sections_are_scanned_and_scoped() {
    let root = temp_root();
    let tasks = root.join("openspec/changes/synthetic/tasks.md");
    let body = "## Tasks\n\noutside-marker: optional arrows cannot be declared.\n\n\
                ## Close note\n\ninside-marker: optional arrows cannot be declared.\n\n\
                ### Nested detail\n\nnested-marker: nothing can express this.\n\n\
                ## After\n\nafter-marker: cannot be declared.\n";
    std::fs::write(&tasks, body).expect("write synthetic tasks.md");

    let rejected = run_gate(Some(&root), None);
    let text = gate_text(&rejected);
    assert_eq!(
        rejected.status.code(),
        Some(1),
        "an un-cited close-note claim passes the gate:\n{text}"
    );
    assert!(
        text.contains("tasks.md") && text.contains("inside-marker"),
        "the close-note claim is not reported against its file:\n{text}"
    );
    assert!(
        text.contains("nested-marker"),
        "a sub-section nested inside the close note is not scanned:\n{text}"
    );
    assert!(
        !text.contains("outside-marker") && !text.contains("after-marker"),
        "lines outside a `## Close note` section leak into scope:\n{text}"
    );

    let cited = "## Close note\n\n\
                 inside-marker: optional arrows cannot be declared (row `optional-references`).\n\n\
                 ### Nested detail\n\n\
                 nested-marker: nothing can express this (row `coproduct-absence`).\n";
    std::fs::write(&tasks, cited).expect("rewrite synthetic tasks.md");
    let accepted = run_gate(Some(&root), None);
    assert!(
        accepted.status.success(),
        "a cited close-note claim is still rejected:\n{}",
        gate_text(&accepted)
    );

    std::fs::remove_dir_all(&root).ok();
}