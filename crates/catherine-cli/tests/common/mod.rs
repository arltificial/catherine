//! Purpose: the shared harness for the CLI's `rules --project` acceptance
//! and refusal binaries — one spelling of the live-binary probe and of the
//! hermetic projection fixture, so the split test files cannot drift.
//!
//! Responsibilities: [`run`]/[`code`]/[`stdout`]/[`stderr`]/[`parse_json`]
//! probe the real `catherine` binary; [`scratch`] writes a hermetic
//! `.ct.json` to the temp directory; [`projection_doc`] builds the inline
//! document (capture rows projected onto `source`/`edition`, optional
//! `batch` action arrow, optional null cells, optional document
//! `rule_presentation`) as one JSON string.
//!
//! Rationale: each integration-test binary compiles this module on its own,
//! so helpers one binary does not use are dead code by design (the
//! `catherine-core/tests/common` precedent). The fixture builder is split
//! into per-part functions to stay inside the test-role complexity caps.

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

pub fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

pub fn parse_json(text: &str, what: &str) -> Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("{what} is one JSON envelope: {e}\n{text}"))
}

/// The envelope's findings array (the one payload shape every rules run
/// carries).
pub fn findings(value: &Value) -> &Vec<Value> {
    value["data"]["findings"]
        .as_array()
        .expect("findings array")
}

/// The finding kinds in payload order.
pub fn kinds(value: &Value) -> Vec<String> {
    findings(value)
        .iter()
        .map(|finding| finding["kind"].as_str().expect("kind").to_string())
        .collect()
}

/// Write a hermetic fixture to the temp directory (per process, so
/// parallel test binaries cannot collide) and return its path.
pub fn scratch(name: &str, contents: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "catherine-cli-projection-{}-{name}.ct.json",
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("write hermetic fixture");
    path.to_string_lossy().into_owned()
}

/// One hermetic projection document: `capture` rows onto the string value
/// objects `source` and `edition` (plus `batch` when `batches` is
/// non-empty). `null_cells` names `(row, arrow)` pairs whose table cell is
/// left absent — the parser's explicit dangling value.
pub fn projection_doc(
    captures: &[(&str, &str, &str)],
    null_cells: &[(&str, &str)],
    batches: &[(&str, &str)],
    with_presentation: bool,
) -> String {
    let mut doc = json!({
        "format_version": "1.2",
        "schema": {
            "objects": objects(batches),
            "arrows": arrows(batches),
            "equations": [],
        },
        "instance": {
            "rows": rows(captures, batches),
            "tables": tables(captures, null_cells, batches),
        },
    });
    if with_presentation {
        doc["rule_presentation"] = document_presentation();
    }
    serde_json::to_string(&doc).expect("serialize fixture")
}

/// The declared objects: the projection's two value objects, and `batch`
/// only when the scenario exercises an action arrow.
fn objects(batches: &[(&str, &str)]) -> Value {
    if batches.is_empty() {
        return json!(["capture", "edition", "source"]);
    }
    json!(["batch", "capture", "edition", "source"])
}

/// The declared arrows: the two projection columns, the not-sourced
/// control arrow (`source_edition`), and the action arrow when in play.
fn arrows(batches: &[(&str, &str)]) -> Value {
    let mut arrows = vec![
        json!({"name": "capture_source", "src": "capture", "tgt": "source"}),
        json!({"name": "capture_edition", "src": "capture", "tgt": "edition"}),
        json!({"name": "source_edition", "src": "source", "tgt": "edition"}),
    ];
    if !batches.is_empty() {
        arrows.push(json!({"name": "capture_batch", "src": "capture", "tgt": "batch"}));
    }
    Value::Array(arrows)
}

/// The instance rows: capture names as supplied, value objects sorted and
/// deduplicated (the parser's dense-id order).
fn rows(captures: &[(&str, &str, &str)], batches: &[(&str, &str)]) -> Value {
    let mut rows = Map::new();
    rows.insert("capture".to_string(), json!(names(captures)));
    rows.insert(
        "source".to_string(),
        json!(sorted(captures.iter().map(|(_, source, _)| *source))),
    );
    rows.insert(
        "edition".to_string(),
        json!(sorted(captures.iter().map(|(_, _, edition)| *edition))),
    );
    if !batches.is_empty() {
        rows.insert(
            "batch".to_string(),
            json!(sorted(batches.iter().map(|(_, batch)| *batch))),
        );
    }
    Value::Object(rows)
}

/// The instance tables, each cell minus the scenario's declared nulls.
fn tables(
    captures: &[(&str, &str, &str)],
    null_cells: &[(&str, &str)],
    batches: &[(&str, &str)],
) -> Value {
    let mut tables = Map::new();
    tables.insert(
        "capture_source".to_string(),
        cells(
            captures.iter().map(|(name, source, _)| (*name, *source)),
            null_cells,
            "capture_source",
        ),
    );
    tables.insert(
        "capture_edition".to_string(),
        cells(
            captures.iter().map(|(name, _, edition)| (*name, *edition)),
            null_cells,
            "capture_edition",
        ),
    );
    if !batches.is_empty() {
        tables.insert(
            "capture_batch".to_string(),
            cells(
                batches.iter().map(|(name, batch)| (*name, *batch)),
                null_cells,
                "capture_batch",
            ),
        );
    }
    Value::Object(tables)
}

/// One table: row name → target row name, skipping the cells the scenario
/// leaves dangling.
fn cells<'a>(
    cells: impl Iterator<Item = (&'a str, &'a str)>,
    null_cells: &[(&str, &str)],
    arrow: &str,
) -> Value {
    let mut table = Map::new();
    for (row, target) in cells {
        if !is_null(null_cells, row, arrow) {
            table.insert(row.to_string(), json!(target));
        }
    }
    Value::Object(table)
}

fn is_null(null_cells: &[(&str, &str)], row: &str, arrow: &str) -> bool {
    null_cells
        .iter()
        .any(|(null_row, null_arrow)| *null_row == row && *null_arrow == arrow)
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
