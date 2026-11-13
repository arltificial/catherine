//! Purpose: the grounded acceptance test of `catherine rules --project`
//! (openspec change `add-rules-projection`, step 1) over the committed
//! capture slice — exactly the three ambiguous selectors, each carrying
//! the two capture rows behind it, no gap on the 30 undeclared zero-match
//! selectors, and one gap when an absent input is declared.
//!
//! Responsibilities: compute the expected collision pairs from the
//! fixture's own rows and dense id order (never a hardcoded capture name),
//! run the real binary against
//! `fixtures/probes/capture-selector/capture-slice.ct.json` twice (plain
//! and with the pre-registered `--values` declaration), and assert exit 3,
//! the finding classes, and the witness pairs.
//!
//! Rationale: the projection is evidence about this snapshot of rows
//! projected this way, so the fixture is the authority; resolving witness
//! indices through the fixture's sorted row names makes a renamed row fail
//! loudly. The hermetic semantics and the refusals keep their own binaries
//! so every file stays inside the file-size gate.

mod common;

use common::{code, findings, kinds, parse_json, run, stderr, stdout};
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Output;

const CAPTURE_SLICE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/probes/capture-selector/capture-slice.ct.json"
);

/// Colliding capture rows, keyed by their `(source, edition)` selector and
/// holding dense row ids.
type CollisionGroups = BTreeMap<(String, String), Vec<usize>>;

fn read_slice() -> Value {
    let text = std::fs::read_to_string(CAPTURE_SLICE).expect("read the committed slice");
    parse_json(&text, "capture slice")
}

/// One named column cell per capture row.
fn cell(value: &Value, table: &str, capture: &str) -> String {
    value["instance"]["tables"][table][capture]
        .as_str()
        .unwrap_or_else(|| panic!("fixture has a `{table}` cell for `{capture}`"))
        .to_string()
}

/// The slice's capture rows in declared file order.
fn capture_names(slice: &Value) -> Vec<String> {
    slice["instance"]["rows"]["backfill_captures"]
        .as_array()
        .expect("capture rows")
        .iter()
        .map(|name| name.as_str().expect("row name").to_string())
        .collect()
}

/// The parser's dense ids: capture names sorted.
fn dense_captures(slice: &Value) -> Vec<String> {
    let mut names = capture_names(slice);
    names.sort();
    names
}

fn dense_id(dense: &[String], capture: &str) -> usize {
    dense
        .iter()
        .position(|row| row == capture)
        .expect("dense id")
}

/// Each capture's projected selector and dense id, grouped.
fn collision_groups(slice: &Value, dense: &[String]) -> CollisionGroups {
    let mut groups: CollisionGroups = BTreeMap::new();
    for capture in capture_names(slice) {
        let selector = (
            cell(slice, "capture_source", &capture),
            cell(slice, "capture_edition_date", &capture),
        );
        groups
            .entry(selector)
            .or_default()
            .push(dense_id(dense, &capture));
    }
    groups
}

/// The selectors shared by more than one capture (canonical order).
fn ambiguous_selectors(groups: &CollisionGroups) -> Vec<(String, String)> {
    groups
        .iter()
        .filter(|(_, rows)| rows.len() > 1)
        .map(|(selector, _)| selector.clone())
        .collect()
}

/// Every pair of captures behind an ambiguous selector (canonical order).
fn ambiguous_pairs(groups: &CollisionGroups) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(usize, usize)> = groups
        .values()
        .filter(|rows| rows.len() > 1)
        .flat_map(|rows| pairs_of(rows))
        .collect();
    pairs.sort();
    pairs
}

fn pairs_of(rows: &[usize]) -> Vec<(usize, usize)> {
    rows.iter()
        .enumerate()
        .flat_map(|(index, left)| rows[index + 1..].iter().map(move |right| (*left, *right)))
        .collect()
}

/// The grounded acceptance command, optionally with extra flags.
fn capture_slice_run(extra: &[&str]) -> Output {
    let mut args = vec![
        "rules",
        CAPTURE_SLICE,
        "--project",
        "backfill_captures",
        "--columns",
        "capture_source,capture_edition_date",
        "--json",
    ];
    args.extend_from_slice(extra);
    run(&args)
}

fn violation_pair(finding: &Value) -> (usize, usize) {
    let left = finding["witness"]["lhs"].as_u64().expect("lhs") as usize;
    let right = finding["witness"]["rhs"].as_u64().expect("rhs") as usize;
    (left, right)
}

/// Three `executed` violations whose witness pairs are exactly the
/// ambiguous selectors' captures — and nothing else.
fn assert_three_violations(value: &Value, expected: &[(usize, usize)]) {
    let found = findings(value);
    assert_eq!(found.len(), 3, "exactly one finding per ambiguous selector");
    let labels: Vec<&str> = found
        .iter()
        .map(|finding| finding["label"].as_str().expect("label"))
        .collect();
    assert_eq!(labels, vec!["executed"; 3]);
    let observed_kinds: Vec<&str> = found
        .iter()
        .map(|finding| finding["kind"].as_str().expect("kind"))
        .collect();
    assert_eq!(
        observed_kinds,
        vec!["violation"; 3],
        "undeclared zero-match selectors are not inputs"
    );
    let mut observed: Vec<(usize, usize)> = found.iter().map(violation_pair).collect();
    observed.sort();
    assert_eq!(
        observed, expected,
        "each witness pair is the two captures behind an ambiguous selector"
    );
}

/// Declaring an absent input adds exactly one gap naming it.
fn assert_declared_gap(out: &Output) {
    assert_eq!(code(out), 3, "violation outranks underconstraint");
    let value = parse_json(&stdout(out), "stdout");
    assert_eq!(findings(&value).len(), 4, "three overlaps plus one gap");
    assert_eq!(
        kinds(&value),
        vec![
            "violation".to_string(),
            "violation".to_string(),
            "violation".to_string(),
            "underconstraint".to_string(),
        ],
        "one gap and no other class"
    );
    let gap = findings(&value)
        .iter()
        .find(|finding| finding["kind"] == "underconstraint")
        .expect("the declared absent input is a gap");
    assert_eq!(gap["label"], "hypothesized");
    let class = gap["witness"]["detail"]["class"]
        .as_str()
        .expect("gap class");
    assert!(
        class.contains("src_nacional_primera") && class.contains("ed_2026_11_07"),
        "the gap names the declared input: {class}"
    );
}

/// The grounded acceptance: exactly the three ambiguous selectors of the
/// committed slice, each carrying the two capture rows behind it; no gap
/// on the 30 undeclared zero-match selectors; declaring an absent input
/// adds the one gap.
#[test]
fn projection_capture_slice_acceptance_three_violations() {
    let slice = read_slice();
    let groups = collision_groups(&slice, &dense_captures(&slice));
    assert_eq!(
        ambiguous_selectors(&groups),
        vec![
            (
                "src_nacional_primera".to_string(),
                "ed_2026_11_03".to_string()
            ),
            (
                "src_nacional_segunda".to_string(),
                "ed_2026_11_13".to_string()
            ),
            ("src_pba_JUDICIAL".to_string(), "ed_2026_11_03".to_string()),
        ],
        "the pre-registered ambiguous selectors"
    );

    let out = capture_slice_run(&[]);
    assert_eq!(code(&out), 3, "three violations → violation class");
    assert_eq!(stderr(&out), "");
    let value = parse_json(&stdout(&out), "stdout");
    assert_three_violations(&value, &ambiguous_pairs(&groups));

    let declared = capture_slice_run(&[
        "--values",
        "capture_source=src_nacional_primera",
        "--values",
        "capture_edition_date=ed_2026_11_07",
    ]);
    assert_declared_gap(&declared);
}
