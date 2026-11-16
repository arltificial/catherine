//! Purpose: the disposition runner of the expressibility gap suite
//! (`openspec/changes/add-expressibility-gap-suite`, beads `catherine-j6g`) —
//! it reads `docs/ref/expressibility-dispositions.md` and holds the table and
//! the executable cases in [`crate::cases`] to each other in both directions.
//!
//! Responsibilities: parse the table's first column (case id), fifth column
//! (bare `gap`/`closed`) and sixth column (sealing evidence); assert every
//! `gap` row still observes [`Observation::Limitation`] and every `closed` row
//! [`Observation::Cleared`]; require `closed` evidence to be a fixing commit
//! hash or a git-tracked (index or HEAD) artifact path and `gap` evidence to
//! name the missing capability; fail on a case without a row or a row without
//! a case; run the textual docs gate
//! (`scripts/check-expressibility-claims.sh`) and prove it rejects an
//! unregistered claim; and assert the gate stays wired into the `ci` recipe.
//! The row list is never duplicated in Rust. Each entry point returns a
//! [`Report`] — rows in scope, the rows actually visited, and the ids whose
//! case contradicted the row — so the harness asserts the run was not vacuous.
//!
//! Rationale: a one-way suite would ossify limitations into folklore, so a
//! behavior change must fail the row that describes it (openspec design
//! Decision 1). The docs gate is deliberately textual — a fixed, documented
//! phrase list, no NLP — matching the other mechanical gates (Decision 4).
//! Claim detection scans README.md, CHANGELOG.md, `docs/**/*.md` and openspec
//! close notes only; `.beads/**`, `.whisper/**`, the rest of
//! `openspec/changes/**`, the two disposition tables and the generated
//! `docs/generated/**` mirrors are out of scope. The phrase list is
//! case-insensitive and fixed: `cannot be expressed`, `can't be expressed`,
//! `cannot be stated`, `cannot be declared`, `cannot be represented`,
//! `nothing can express`, `not expressible`, `no way to express`,
//! `inexpressible`. A matching line must contain a table row id; otherwise the
//! gate names the file and line and exits 1.

use crate::cases::{Case, Observation};
use crate::evidence::{is_fixing_evidence, names_missing_capability, repo_root};
use std::path::Path;
use std::process::{Command, Output};

/// The disposition table this runner reads; never mirrored in Rust.
const TABLE: &str = "docs/ref/expressibility-dispositions.md";

/// The docs gate recipe the `ci` recipe must invoke.
const CLAIM_GATE_RECIPE: &str = "check-gap-claims";

/// The disposition half of one table row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    Gap,
    Closed,
}

/// One parsed disposition row (the columns the runner enforces).
#[derive(Debug)]
struct Row {
    id: String,
    disposition: Disposition,
    evidence: String,
}

/// What one runner direction verified, so a test cannot pass vacuously: the
/// rows in scope for this run, the rows actually visited, and the rows whose
/// case contradicted their disposition. A run that parses nothing, or visits
/// fewer rows than it registered, fails the harness assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Disposition rows in scope for this run.
    pub rows: usize,
    /// Rows the run actually visited.
    pub checked: usize,
    /// Ids of rows whose observed verdict contradicted their disposition.
    pub flipped: Vec<String>,
}

impl Report {
    fn new(rows: usize) -> Self {
        Self {
            rows,
            checked: 0,
            flipped: Vec::new(),
        }
    }

    /// Record one row's observed verdict against the one its disposition claims.
    fn record(&mut self, id: &str, observed: Observation, expected: Observation) {
        self.checked += 1;
        if observed != expected {
            self.flipped.push(id.to_string());
        }
    }

    /// Record one row visited by a check that has no verdict to compare.
    fn visit(&mut self) {
        self.checked += 1;
    }

    /// A run whose whole row set was consulted without a per-row verdict.
    fn visited_all(mut self) -> Self {
        self.checked = self.rows;
        self
    }

    /// Panic with the per-row guidance when a case contradicted its row.
    fn expect_no_flips(&self, registered: &str, guidance: &str) {
        assert!(
            self.flipped.is_empty(),
            "{}",
            self.flipped
                .iter()
                .map(|id| format!("row `{id}` is registered `{registered}`; {guidance}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Read and parse the disposition table from disk.
fn table() -> Vec<Row> {
    let path = repo_root().join(TABLE);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let rows = parse_table(&text);
    assert!(
        !rows.is_empty(),
        "{} registers no disposition rows",
        path.display()
    );
    rows
}

fn parse_table(text: &str) -> Vec<Row> {
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
        "malformed disposition row (expected 8 pipe-split parts for six columns): {line}"
    );
    let id = cells[1];
    if id.is_empty() || id == "Case id" || id.bytes().all(|byte| byte == b'-') {
        return None;
    }
    let disposition = match cells[5] {
        "gap" => Disposition::Gap,
        "closed" => Disposition::Closed,
        other => panic!("row `{id}`: disposition must be bare `gap` or `closed`, got `{other}`"),
    };
    Some(Row {
        id: id.to_string(),
        disposition,
        evidence: cells[6].to_string(),
    })
}

/// Resolve the case registered for a row, naming the row when it is missing.
fn case_for<'a>(cases: &'a [Case], id: &str) -> &'a Case {
    cases
        .iter()
        .find(|case| case.id == id)
        .unwrap_or_else(|| panic!("row `{id}` has no registered case in `cases::all()`"))
}

/// Every `gap` row's case must still observe the limitation.
pub fn gap_rows_still_show_the_limitation(cases: &[Case]) -> Report {
    let rows = table();
    let gap_rows: Vec<&Row> = rows
        .iter()
        .filter(|row| row.disposition == Disposition::Gap)
        .collect();
    let mut report = Report::new(gap_rows.len());
    for row in gap_rows {
        report.record(
            &row.id,
            (case_for(cases, &row.id).observe)(),
            Observation::Limitation,
        );
    }
    report.expect_no_flips(
        "gap",
        "its case no longer shows the limitation — flip the row to `closed` with the fixing evidence",
    );
    eprintln!("gap rows observed as limitations: {}", report.checked);
    report
}

/// Every `closed` row's case must still observe the fixed behavior.
pub fn closed_rows_still_show_the_fix(cases: &[Case]) -> Report {
    let rows = table();
    let closed_rows: Vec<&Row> = rows
        .iter()
        .filter(|row| row.disposition == Disposition::Closed)
        .collect();
    let mut report = Report::new(closed_rows.len());
    for row in closed_rows {
        report.record(
            &row.id,
            (case_for(cases, &row.id).observe)(),
            Observation::Cleared,
        );
    }
    report.expect_no_flips(
        "closed",
        "its case shows the limitation again — record the regression and flip the row back to `gap`",
    );
    eprintln!("closed rows observed as fixes: {}", report.checked);
    report
}

/// `closed` rows cite fixing evidence; `gap` rows name the missing capability.
pub fn closed_rows_cite_fixing_evidence(_cases: &[Case]) -> Report {
    let rows = table();
    let mut report = Report::new(rows.len());
    let mut closed = 0;
    for row in &rows {
        report.visit();
        let evidence = row.evidence.trim();
        assert!(
            !evidence.is_empty(),
            "row `{}` ({:?}) carries no sealing evidence",
            row.id,
            row.disposition
        );
        match row.disposition {
            Disposition::Closed => {
                assert!(
                    is_fixing_evidence(evidence),
                    "closed row `{}` must cite a commit or a git-tracked (index or HEAD) \
                     artifact path, got `{evidence}`",
                    row.id
                );
                closed += 1;
            }
            Disposition::Gap => assert!(
                names_missing_capability(evidence),
                "gap row `{}` must name the missing capability (`Missing: …`), \
                 not cite a fixing commit or artifact path; got `{evidence}`",
                row.id
            ),
        }
    }
    eprintln!("closed rows citing fixing evidence: {closed}");
    report
}

/// Every declared fixture document must exist under the gap-suite root.
fn assert_fixtures_exist(case: &Case) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gap_suite");
    for fixture in case.fixtures {
        let path = root.join(fixture);
        assert!(
            path.is_file(),
            "case `{}`: fixture {} is missing",
            case.id,
            path.display()
        );
    }
}

/// Bidirectional registration: every case has a row and every row a case.
pub fn every_case_is_registered(cases: &[Case]) -> Report {
    let rows = table();
    let mut report = Report::new(rows.len());
    for row in &rows {
        let case = case_for(cases, &row.id);
        assert_fixtures_exist(case);
        report.visit();
    }
    let row_ids: std::collections::BTreeSet<&str> =
        rows.iter().map(|row| row.id.as_str()).collect();
    let case_ids: std::collections::BTreeSet<&str> = cases.iter().map(|case| case.id).collect();
    assert_eq!(row_ids.len(), rows.len(), "duplicate row id in the table");
    assert_eq!(
        case_ids.len(),
        cases.len(),
        "duplicate case id in `cases::all()`"
    );
    let unregistered: Vec<&&str> = case_ids.difference(&row_ids).collect();
    assert!(
        unregistered.is_empty(),
        "cases without a disposition row: {unregistered:?}"
    );
    let uncased: Vec<&&str> = row_ids.difference(&case_ids).collect();
    assert!(
        uncased.is_empty(),
        "disposition rows without a registered case: {uncased:?}"
    );
    report.visited_all()
}

/// The docs gate: run the script over the real docs, then prove it rejects a
/// synthetic claim with no row id and accepts the same claim with one.
pub fn limitation_claims_name_a_disposition_row(_cases: &[Case]) -> Report {
    let rows = table();
    let clean = run_claim_gate(None);
    assert!(
        clean.status.success(),
        "the docs gate fails at HEAD:\n{}",
        gate_text(&clean)
    );

    let unregistered = "nothing can express a coproduct here";
    let rejected = run_claim_gate(Some(unregistered));
    assert_eq!(
        rejected.status.code(),
        Some(1),
        "the gate accepts a limitation claim with no row id:\n{}",
        gate_text(&rejected)
    );
    let text = gate_text(&rejected);
    assert!(
        text.contains("no disposition row id") && text.contains("nothing can express"),
        "the gate rejection names the claim and the missing row id:\n{text}"
    );

    let row_id = rows
        .last()
        .map(|row| row.id.clone())
        .expect("a registered row id");
    let cited = format!("{unregistered} (row `{row_id}`)");
    let accepted = run_claim_gate(Some(&cited));
    assert!(
        accepted.status.success(),
        "the gate rejects a claim that names a registered row:\n{}",
        gate_text(&accepted)
    );
    Report::new(rows.len()).visited_all()
}

fn run_claim_gate(synthetic: Option<&str>) -> Output {
    let script = repo_root().join("scripts/check-expressibility-claims.sh");
    let mut command = Command::new("bash");
    command.arg(&script);
    if let Some(line) = synthetic {
        command.env("CLAIM_GATE_SYNTHETIC", line);
    }
    command
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", script.display()))
}

fn gate_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// `just ci` must keep invoking the claim gate recipe.
pub fn gate_is_wired_into_ci(_cases: &[Case]) -> Report {
    let path = repo_root().join("justfile");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let recipe = format!("{CLAIM_GATE_RECIPE}:");
    assert!(
        text.lines().any(|line| line.trim() == recipe),
        "justfile defines no `{CLAIM_GATE_RECIPE}` recipe"
    );
    let body = recipe_body(&text, "ci").unwrap_or_else(|| panic!("justfile has no `ci` recipe"));
    assert!(
        body.iter().any(|line| {
            let line = line.trim();
            line.starts_with(&format!("just {CLAIM_GATE_RECIPE}"))
                || line.starts_with(&CLAIM_GATE_RECIPE)
        }),
        "the `ci` recipe never invokes `{CLAIM_GATE_RECIPE}`:\n{}",
        body.join("\n")
    );
    Report::new(table().len()).visited_all()
}

/// The indented line body of a top-level recipe, `None` when it is absent.
fn recipe_body(text: &str, name: &str) -> Option<Vec<String>> {
    let header = format!("{name}:");
    let mut lines = text.lines();
    lines.find(|line| line.trim_end() == header)?;
    Some(
        lines
            .take_while(|line| line.is_empty() || line.starts_with(' ') || line.starts_with('\t'))
            .map(str::to_string)
            .collect(),
    )
}
