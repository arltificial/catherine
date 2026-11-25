//! Purpose: the shared harness for the CLI's `rules --project` acceptance
//! and refusal binaries - one spelling of the live-binary probe and of the
//! hermetic projection fixture, so the split test files cannot drift. It
//! also hosts the plumbing the other CLI acceptance binaries share.
//!
//! Responsibilities: [`run`]/[`code`]/[`stdout`]/[`stderr`]/[`parse_json`]
//! probe the real `catherine` binary; [`try_run`]/[`envelope`] are their
//! propagating forms for binaries whose tests return `Result`; [`lost`]
//! reads the migration envelope's `data.lost`; [`scratch`] writes a
//! hermetic `.ct.json` to the temp directory; [`projection_doc!`] builds
//! the inline document (capture rows projected onto `source`/`edition`,
//! optional `batch` action arrow, optional null cells, optional document
//! `rule_presentation`) as one JSON string.
//!
//! Rationale: each integration-test binary compiles this module on its
//! own, so helpers one binary does not use are dead code by design (the
//! `catherine-core/tests/common` precedent). The fixture builder is split
//! into per-part functions to stay inside the test-role complexity caps,
//! and [`projection_doc!`] takes the builder's four arguments positionally
//! so the fixture keeps one spelling at every call site.
#![allow(dead_code)]
use serde_json::{json, Map, Value};
use std::process::{Command, Output};
/// Spawn the `catherine` binary with the given arguments.
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
        .expect("spawn catherine binary")
}
/// `run` for binaries whose tests return `Result`: the spawn error
/// (never observed in practice) propagates instead of panicking here.
pub fn try_run(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
}
pub fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}
pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}
/// Parse one JSON envelope from `stream`, naming the stream and carrying
/// the raw text on failure - the panicking [`parse_json`]'s diagnostics,
/// spelled for `?`.
pub fn envelope(stream: &str, text: &str) -> Result<Value, Box<dyn std::error::Error>> {
    serde_json::from_str(text)
        .map_err(|error| format!("{stream} is one JSON envelope: {error}\n{text}").into())
}
pub fn parse_json(text: &str, what: &str) -> Value {
    serde_json::from_str(text)
        .unwrap_or_else(|error| panic!("{what} is one JSON envelope: {error}\n{text}"))
}
/// The envelope's findings array (the one payload shape every rules run
/// carries).
pub fn findings(value: &Value) -> &Vec<Value> {
    value["data"]["findings"]
        .as_array()
        .unwrap_or_else(|| panic!("findings array"))
}
/// The finding kinds in payload order.
pub fn kinds(value: &Value) -> Vec<String> {
    findings(value)
        .iter()
        .map(|finding| finding["kind"].as_str().expect("kind").to_string())
        .collect()
}
/// The migration envelope's `data.lost` array - the kernel
/// `MigrationFinding` encoding, verbatim.
pub fn lost(value: &Value) -> &Vec<Value> {
    value["data"]["lost"]
        .as_array()
        .unwrap_or_else(|| panic!("data.lost array"))
}
/// Write a hermetic fixture to the temp directory (per process, so
/// parallel test binaries cannot collide) and return its path.
pub fn scratch(name: &str, contents: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "catherine-rules-projection-{}-{name}",
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("write hermetic fixture");
    path.display().to_string()
}
/// One hermetic projection scenario's declared parts: the capture rows,
/// the `(row, arrow)` cells left dangling, and the action-arrow `batch`
/// rows. The bundle form is what keeps every builder helper inside the
/// test-role two-parameter cap.
pub struct ProjectionParts<'a> {
    pub captures: &'a [(&'a str, &'a str, &'a str)],
    pub null_cells: &'a [(&'a str, &'a str)],
    pub batches: &'a [(&'a str, &'a str)],
}
/// Build the scenario's document JSON with the builder's four arguments,
/// positional as ever: captures, null cells, batch rows, and whether the
/// document carries a `rule_presentation` (the two-presentation-sources
/// refusal fixture is the one that does).
#[macro_export]
macro_rules! projection_doc {
    ($captures:expr, $null_cells:expr, $batches:expr, $with_presentation:expr $(,)?) => {
        $crate::common::projection_json(
            &$crate::common::ProjectionParts {
                captures: $captures,
                null_cells: $null_cells,
                batches: $batches,
            },
            $with_presentation,
        )
    };
}
/// Build the scenario's document (no `rule_presentation` key unless
/// requested) as one JSON string.
pub fn projection_json(parts: &ProjectionParts, with_presentation: bool) -> String {
    let doc = json!({
        "format_version": "1.2",
        "schema": {
            "objects": objects(parts.batches),
            "arrows": arrows(parts.batches),
            "equations": [],
        },
        "instance": {
            "rows": rows(parts),
            "tables": tables(parts),
        },
    });
    let presented = if with_presentation {
        present(doc, document_presentation())
    } else {
        doc
    };
    serde_json::to_string(&presented).unwrap_or_else(|error| panic!("serialize fixture: {error}"))
}
/// The document with `rule_presentation` attached (an owning helper, so
/// the builder's bindings stay immutable).
fn present(mut doc: Value, presentation: Value) -> Value {
    doc["rule_presentation"] = presentation;
    doc
}
/// The declared objects: the two projection columns plus `edition`, and
/// the `batch` object when an action arrow is in play.
fn objects(batches: &[(&str, &str)]) -> Value {
    if batches.is_empty() {
        return json!(["capture", "edition", "source"]);
    }
    json!(["batch", "capture", "edition", "source"])
}
/// The declared arrows: the two projection columns, the not-sourced
/// control arrow (`source_edition`), and the action arrow when in play.
fn arrows(batches: &[(&str, &str)]) -> Value {
    let base = vec![
        json!({"name": "capture_source", "src": "capture", "tgt": "source"}),
        json!({"name": "capture_edition", "src": "capture", "tgt": "edition"}),
        json!({"name": "source_edition", "src": "source", "tgt": "edition"}),
    ];
    let batch = if batches.is_empty() {
        Vec::new()
    } else {
        vec![json!({"name": "capture_batch", "src": "capture", "tgt": "batch"})]
    };
    Value::Array(base.into_iter().chain(batch).collect())
}
/// The instance rows: capture names as supplied, value objects sorted and
/// deduplicated (the parser's dense-id order), plus the `batch` row when
/// the scenario declares an action arrow.
fn rows(parts: &ProjectionParts) -> Value {
    let declared: Vec<(String, Value)> = [
        ("capture".to_string(), json!(names(parts.captures))),
        (
            "source".to_string(),
            json!(sorted(parts.captures.iter().map(|(_, source, _)| *source))),
        ),
        (
            "edition".to_string(),
            json!(sorted(
                parts.captures.iter().map(|(_, _, edition)| *edition)
            )),
        ),
    ]
    .into_iter()
    .chain(batch_rows(parts.batches))
    .collect();
    Value::Object(declared.into_iter().collect())
}
/// The `batch` row when the scenario declares an action arrow.
fn batch_rows(batches: &[(&str, &str)]) -> Vec<(String, Value)> {
    if batches.is_empty() {
        return Vec::new();
    }
    vec![(
        "batch".to_string(),
        json!(sorted(batches.iter().map(|(_, batch)| *batch))),
    )]
}
/// The instance tables, each cell minus the scenario's declared nulls.
fn tables(parts: &ProjectionParts) -> Value {
    let declared: Vec<(String, Value)> = [
        (
            "capture_source".to_string(),
            table(
                parts
                    .captures
                    .iter()
                    .map(|(name, source, _)| (*name, *source)),
                &nulls_on(parts.null_cells, "capture_source"),
            ),
        ),
        (
            "capture_edition".to_string(),
            table(
                parts
                    .captures
                    .iter()
                    .map(|(name, _, edition)| (*name, *edition)),
                &nulls_on(parts.null_cells, "capture_edition"),
            ),
        ),
    ]
    .into_iter()
    .chain(batch_table(parts))
    .collect();
    Value::Object(declared.into_iter().collect())
}
/// The `capture_batch` table when the scenario declares an action arrow.
fn batch_table(parts: &ProjectionParts) -> Vec<(String, Value)> {
    if parts.batches.is_empty() {
        return Vec::new();
    }
    vec![(
        "capture_batch".to_string(),
        table(
            parts.batches.iter().map(|(name, batch)| (*name, *batch)),
            &nulls_on(parts.null_cells, "capture_batch"),
        ),
    )]
}
/// One instance table: row name -> target row name, skipping the cells
/// the scenario leaves dangling.
fn table<'a>(cells: impl Iterator<Item = (&'a str, &'a str)>, nulls: &[(&str, &str)]) -> Value {
    let kept: Map<String, Value> = cells
        .filter(|(row, _)| !nulls.iter().any(|(null_row, _)| null_row == row))
        .map(|(row, target)| (row.to_string(), json!(target)))
        .collect();
    Value::Object(kept)
}
/// The scenario's dangling cells on one arrow.
fn nulls_on<'a>(null_cells: &'a [(&'a str, &'a str)], arrow: &str) -> Vec<(&'a str, &'a str)> {
    null_cells
        .iter()
        .filter(|(_, null_arrow)| *null_arrow == arrow)
        .copied()
        .collect()
}
fn names<'a>(captures: &[(&'a str, &'a str, &'a str)]) -> Vec<&'a str> {
    captures.iter().map(|(name, _, _)| *name).collect()
}
/// Sorted, deduplicated declared names.
fn sorted<'a>(items: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut names: Vec<&str> = items.collect();
    names.sort_unstable();
    names.dedup();
    names
}
/// A valid document `rule_presentation` (the two-presentation-sources
/// refusal fixture).
fn document_presentation() -> Value {
    json!({
        "domain": "capture",
        "columns": [{"name": "n", "type": "int"}],
        "rules": [
            {"name": "one", "predicate": {"kind": "cmp", "column": "n", "op": "eq", "value": 1},
             "action": "one"}
        ],
        "policy": "DeclaredDisjoint"
    })
}
