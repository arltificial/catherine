//! Purpose: acceptance tests for the `catherine rules --project` bridge
//! (openspec change `add-rules-projection`, step 1) — an instance of
//! stored rows projected onto a rule table and analyzed by the shipped
//! rule lens, with no new finding kinds and no envelope change.
//!
//! Responsibilities: spawn the real binary (`CARGO_BIN_EXE_catherine`)
//! against hermetic inline `.ct.json` documents written to the temp
//! directory (collision, clean, sparse, null-cell, empty-projection and
//! two-presentation-sources fixtures) and against the committed grounded
//! slice (`fixtures/probes/capture-selector/capture-slice.ct.json`);
//! assert exactly one `executed` violation per colliding row pair with
//! both rows in the witness, the observed-tuple domain's silence on
//! combinations no row has, the declared-`--values` gap, the
//! `--action` arrow, and every labeled refusal
//! (`bad_argument`, `projection_incomplete`, `empty_projection`).
//!
//! Rationale: the projection's claim lives outside the model hash and
//! the ledger, so the only honest test is an end-to-end run of the real
//! binary: the fixture is the evidence. Assertions are structural
//! (`serde_json::Value`) and numeric (exit codes), never snapshots;
//! witness pairs are resolved through the fixture's own row order, so a
//! renamed row cannot silently satisfy the test.

use serde_json::{json, Map, Value};
use std::process::{Command, Output};

const CAPTURE_SLICE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/probes/capture-selector/capture-slice.ct.json"
);

/// Spawn the `catherine` binary with the given arguments.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
        .expect("spawn catherine binary")
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn parse_json(text: &str, what: &str) -> Value {
    serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("{what} is one JSON envelope: {e}\n{text}"))
}

/// Write a hermetic fixture to the temp directory (per process, so
/// parallel test binaries cannot collide) and return its path.
fn scratch(name: &str, contents: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "catherine-cli-projection-{}-{name}.ct.json",
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("write hermetic fixture");
    path.to_string_lossy().into_owned()
}

/// One hermetic projection document: `capture` rows onto the string
/// value objects `source` and `edition` (plus `batch` when `batches` is
/// non-empty). `null_cells` names `(row, arrow)` pairs whose table cell
/// is left absent — the parser's explicit dangling value.
fn projection_doc(
    captures: &[(&str, &str, &str)],
    null_cells: &[(&str, &str)],
    batches: &[(&str, &str)],
    with_presentation: bool,
) -> String {
    let mut sources: Vec<&str> = captures.iter().map(|(_, source, _)| *source).collect();
    sources.sort_unstable();
    sources.dedup();
    let mut editions: Vec<&str> = captures.iter().map(|(_, _, edition)| *edition).collect();
    editions.sort_unstable();
    editions.dedup();
    let mut batches_names: Vec<&str> = batches.iter().map(|(_, batch)| *batch).collect();
    batches_names.sort_unstable();
    batches_names.dedup();

    let mut objects: Vec<&str> = vec!["capture", "edition", "source"];
    if !batches.is_empty() {
        objects.push("batch");
        objects.sort_unstable();
    }
    let mut arrows = vec![
        json!({"name": "capture_source", "src": "capture", "tgt": "source"}),
        json!({"name": "capture_edition", "src": "capture", "tgt": "edition"}),
        json!({"name": "source_edition", "src": "source", "tgt": "edition"}),
    ];
    if !batches.is_empty() {
        arrows.push(json!({"name": "capture_batch", "src": "capture", "tgt": "batch"}));
    }

    let mut source_cells = Map::new();
    let mut edition_cells = Map::new();
    for (name, source, edition) in captures {
        let is_null = |arrow: &str| {
            null_cells
                .iter()
                .any(|(row, null_arrow)| row == name && *null_arrow == arrow)
        };
        if !is_null("capture_source") {
            source_cells.insert((*name).to_string(), json!(source));
        }
        if !is_null("capture_edition") {
            edition_cells.insert((*name).to_string(), json!(edition));
        }
    }
    let mut batch_cells = Map::new();
    for (name, batch) in batches {
        let is_null = null_cells
            .iter()
            .any(|(row, arrow)| row == name && *arrow == "capture_batch");
        if !is_null {
            batch_cells.insert((*name).to_string(), json!(batch));
        }
    }

    let mut rows = Map::new();
    rows.insert(
        "capture".to_string(),
        json!(captures
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<Vec<_>>()),
    );
    rows.insert("source".to_string(), json!(sources));
    rows.insert("edition".to_string(), json!(editions));
    if !batches.is_empty() {
        rows.insert("batch".to_string(), json!(batches_names));
    }
    let mut tables = Map::new();
    tables.insert("capture_source".to_string(), Value::Object(source_cells));
    tables.insert("capture_edition".to_string(), Value::Object(edition_cells));
    if !batches.is_empty() {
        tables.insert("capture_batch".to_string(), Value::Object(batch_cells));
    }

    let mut doc = json!({
        "format_version": "1.2",
        "schema": {"objects": objects, "arrows": arrows, "equations": []},
        "instance": {"rows": rows, "tables": tables},
    });
    if with_presentation {
        doc["rule_presentation"] = json!({
            "domain": "capture",
            "columns": [{"name": "n", "type": "int"}],
            "rules": [
                {"name": "one", "predicate": {"kind": "cmp", "column": "n", "op": "eq", "value": 1},
                 "action": "one"}
            ],
            "policy": "DeclaredDisjoint"
        });
    }
    serde_json::to_string(&doc).expect("serialize fixture")
}

/// One named column cell per capture row, in the given order.
fn cell(value: &Value, table: &str, capture: &str) -> String {
    value["instance"]["tables"][table][capture]
        .as_str()
        .unwrap_or_else(|| panic!("fixture has a `{table}` cell for `{capture}`"))
        .to_string()
}

fn findings(value: &Value) -> &Vec<Value> {
    value["data"]["findings"]
        .as_array()
        .expect("findings array")
}

fn kinds(value: &Value) -> Vec<String> {
    findings(value)
        .iter()
        .map(|finding| finding["kind"].as_str().expect("kind").to_string())
        .collect()
}

/// The labeled error of a run that must refuse: exit 1, code, message.
fn refusal(out: &Output, expected_code: &str) -> String {
    assert_eq!(code(out), 1, "refusals are the build-failure exit class");
    assert_eq!(stdout(out), "", "errors never write stdout");
    let envelope = parse_json(&stderr(out), "stderr");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["envelope_kind"], "error");
    assert_eq!(envelope["data"]["code"], expected_code);
    envelope["data"]["message"]
        .as_str()
        .expect("message")
        .to_string()
}

/// Two rows sharing both projected columns are one overlap per shared
/// input: exactly one `executed` violation, and the human render names
/// both capture rows.
#[test]
fn projection_collision_emits_one_violation_with_both_rows() {
    let doc = projection_doc(
        &[
            ("cap_a", "src_a", "ed_1"),
            ("cap_b", "src_a", "ed_1"),
            ("cap_c", "src_b", "ed_2"),
        ],
        &[],
        &[],
        false,
    );
    let path = scratch("collision", &doc);

    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
        "--json",
    ]);
    assert_eq!(code(&out), 3, "overlap under DeclaredDisjoint → violation");
    assert_eq!(stderr(&out), "", "the error stream stays empty");

    let value = parse_json(&stdout(&out), "stdout");
    assert_eq!(value["data"]["outcome"], "analyzed");
    let found = findings(&value);
    assert_eq!(found.len(), 1, "one finding: {found:?}");
    assert_eq!(found[0]["label"], "executed");
    assert_eq!(found[0]["kind"], "violation");
    assert_eq!(
        found[0]["witness"]["lhs"], 0,
        "rule indices are the dense row order"
    );
    assert_eq!(found[0]["witness"]["rhs"], 1);
    assert_eq!(
        kinds(&value),
        vec!["violation".to_string()],
        "no gap: the domain is the observed tuple set"
    );

    let human = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
    ]);
    assert_eq!(code(&human), 3, "the exit decision is mode-independent");
    let text = stdout(&human);
    assert!(
        text.contains("`cap_a`") && text.contains("`cap_b`"),
        "the witness names both rows: {text}"
    );
}

/// The grounded acceptance: exactly the three ambiguous selectors of the
/// committed slice, each carrying the two capture rows behind it; no gap
/// on the 30 undeclared zero-match selectors; declaring an absent input
/// adds the one gap.
#[test]
fn projection_capture_slice_acceptance_three_violations() {
    let text = std::fs::read_to_string(CAPTURE_SLICE).expect("read the committed slice");
    let slice = parse_json(&text, "capture slice");
    let captures: Vec<String> = slice["instance"]["rows"]["backfill_captures"]
        .as_array()
        .expect("capture rows")
        .iter()
        .map(|name| name.as_str().expect("row name").to_string())
        .collect();
    let mut dense = captures.clone();
    dense.sort();
    let index = |capture: &str| {
        dense
            .iter()
            .position(|row| row == capture)
            .expect("dense id")
    };

    let mut groups: std::collections::BTreeMap<(String, String), Vec<usize>> =
        std::collections::BTreeMap::new();
    for capture in &captures {
        let key = (
            cell(&slice, "capture_source", capture),
            cell(&slice, "capture_edition_date", capture),
        );
        groups.entry(key).or_default().push(index(capture));
    }
    let mut expected_pairs: Vec<(usize, usize)> = Vec::new();
    let mut expected_selectors: Vec<(String, String)> = Vec::new();
    for (selector, rows) in &groups {
        if rows.len() > 1 {
            expected_selectors.push(selector.clone());
            for left in rows {
                for right in rows {
                    if left < right {
                        expected_pairs.push((*left, *right));
                    }
                }
            }
        }
    }
    expected_pairs.sort();
    assert_eq!(
        expected_selectors,
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

    let out = run(&[
        "rules",
        CAPTURE_SLICE,
        "--project",
        "backfill_captures",
        "--columns",
        "capture_source,capture_edition_date",
        "--json",
    ]);
    assert_eq!(code(&out), 3, "three violations → violation class");
    assert_eq!(stderr(&out), "");
    let value = parse_json(&stdout(&out), "stdout");
    let found = findings(&value);
    assert_eq!(found.len(), 3, "exactly one finding per ambiguous selector");
    let mut observed_pairs = Vec::new();
    for finding in found {
        assert_eq!(finding["label"], "executed");
        assert_eq!(finding["kind"], "violation");
        let left = finding["witness"]["lhs"].as_u64().expect("lhs") as usize;
        let right = finding["witness"]["rhs"].as_u64().expect("rhs") as usize;
        observed_pairs.push((left, right));
    }
    observed_pairs.sort();
    assert_eq!(
        observed_pairs, expected_pairs,
        "each witness pair is the two captures behind an ambiguous selector"
    );
    assert!(
        !kinds(&value).contains(&"underconstraint".to_string()),
        "undeclared zero-match selectors are not inputs"
    );

    let declared = run(&[
        "rules",
        CAPTURE_SLICE,
        "--project",
        "backfill_captures",
        "--columns",
        "capture_source,capture_edition_date",
        "--values",
        "capture_source=src_nacional_primera",
        "--values",
        "capture_edition_date=ed_2026_11_07",
        "--json",
    ]);
    assert_eq!(code(&declared), 3, "violation outranks underconstraint");
    let value = parse_json(&stdout(&declared), "stdout");
    assert_eq!(findings(&value).len(), 4, "three overlaps plus one gap");
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

/// A projection of rows covering only part of a larger conceivable set
/// says nothing about the combinations no row has: the domain is the
/// observed tuple set.
#[test]
fn projection_undeclared_absent_input_is_silent() {
    let doc = projection_doc(
        &[
            ("cap_a", "src_a", "ed_1"),
            ("cap_b", "src_b", "ed_2"),
            ("cap_c", "src_a", "ed_2"),
        ],
        &[],
        &[],
        false,
    );
    let path = scratch("sparse", &doc);

    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
        "--json",
    ]);
    assert_eq!(code(&out), 0, "no findings over the observed tuples");
    assert_eq!(stderr(&out), "");
    let value = parse_json(&stdout(&out), "stdout");
    assert_eq!(findings(&value), &Vec::<Value>::new());
    assert!(
        !kinds(&value).contains(&"underconstraint".to_string()),
        "(src_b, ed_1) is not a projection input unless declared"
    );
}

/// A `--values` declaration that names a combination no row has is a
/// genuine gap: the declared scope, not a claim about the data.
#[test]
fn projection_declared_absent_input_yields_gap() {
    let doc = projection_doc(
        &[
            ("cap_a", "src_a", "ed_1"),
            ("cap_b", "src_b", "ed_2"),
            ("cap_c", "src_a", "ed_2"),
        ],
        &[],
        &[],
        false,
    );
    let path = scratch("declared-gap", &doc);

    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
        "--values",
        "capture_source=src_a,src_b",
        "--values",
        "capture_edition=ed_1,ed_2",
        "--json",
    ]);
    assert_eq!(code(&out), 6, "the only finding is a gap");
    assert_eq!(stderr(&out), "");
    let value = parse_json(&stdout(&out), "stdout");
    let found = findings(&value);
    assert_eq!(found.len(), 1, "one gap finding: {found:?}");
    assert_eq!(found[0]["label"], "hypothesized");
    assert_eq!(found[0]["kind"], "underconstraint");
    assert_eq!(found[0]["witness"]["case"], "uncovered_inputs");
    assert_eq!(
        found[0]["witness"]["detail"]["class"],
        "capture_edition = \"ed_1\", capture_source = \"src_b\"",
        "the declared class is named (columns in canonical order)"
    );
    assert_eq!(
        found[0]["witness"]["detail"]["examples"],
        json!([1]),
        "the gap's dense input index is the canonical tuple index"
    );
}

/// A clean projection (every row's tuple distinct) is exit 0 with zero
/// findings — the projection itself must not manufacture a diagnosis.
#[test]
fn projection_clean_rows_are_exit_zero() {
    let doc = projection_doc(
        &[
            ("cap_a", "src_a", "ed_1"),
            ("cap_b", "src_b", "ed_2"),
            ("cap_c", "src_b", "ed_1"),
        ],
        &[],
        &[],
        false,
    );
    let path = scratch("clean", &doc);

    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
        "--json",
    ]);
    assert_eq!(code(&out), 0);
    assert_eq!(stderr(&out), "");
    let value = parse_json(&stdout(&out), "stdout");
    assert_eq!(findings(&value), &Vec::<Value>::new());
    assert_eq!(kinds(&value), Vec::<String>::new());
}

/// `--action` names a non-identity action arrow; the projected rules take
/// that arrow's target row name and the analysis is unchanged.
#[test]
fn projection_action_arrow_is_accepted() {
    let doc = projection_doc(
        &[("cap_a", "src_a", "ed_1"), ("cap_b", "src_b", "ed_2")],
        &[],
        &[("cap_a", "b1"), ("cap_b", "b2")],
        false,
    );
    let path = scratch("action", &doc);

    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
        "--action",
        "capture_batch",
        "--json",
    ]);
    assert_eq!(code(&out), 0, "identity and --action are both valid");
    assert_eq!(stderr(&out), "");
    let value = parse_json(&stdout(&out), "stdout");
    assert_eq!(findings(&value), &Vec::<Value>::new());
}

/// A row that cannot be projected is a labeled error naming the row and
/// the arrow — never a silently dropped row.
#[test]
fn projection_null_cell_is_labeled_error_with_row_and_arrow() {
    let doc = projection_doc(
        &[("cap_a", "src_a", "ed_1"), ("cap_b", "src_b", "ed_2")],
        &[("cap_b", "capture_source")],
        &[],
        false,
    );
    let path = scratch("null-column", &doc);
    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source,capture_edition",
        "--json",
    ]);
    let message = refusal(&out, "projection_incomplete");
    assert!(
        message.contains("cap_b") && message.contains("capture_source"),
        "the refusal names the row and the arrow: {message}"
    );

    let doc = projection_doc(
        &[("cap_a", "src_a", "ed_1"), ("cap_b", "src_b", "ed_2")],
        &[("cap_b", "capture_batch")],
        &[("cap_a", "b1"), ("cap_b", "b2")],
        false,
    );
    let path = scratch("null-action", &doc);
    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source",
        "--action",
        "capture_batch",
        "--json",
    ]);
    let message = refusal(&out, "projection_incomplete");
    assert!(
        message.contains("cap_b") && message.contains("capture_batch"),
        "the null action cell names the row and the arrow: {message}"
    );
}

/// An empty projection supports no claim: a clean exit would be the
/// vacuous-clean defect.
#[test]
fn projection_empty_object_is_labeled_error_not_clean_run() {
    let doc = projection_doc(&[], &[], &[], false);
    let path = scratch("empty", &doc);
    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source",
        "--json",
    ]);
    let message = refusal(&out, "empty_projection");
    assert!(
        message.contains("capture"),
        "the refusal names the projected object: {message}"
    );
}

/// Two sources of the same presentation are a labeled refusal — never an
/// implicit preference for the document's `rule_presentation`.
#[test]
fn projection_refuses_document_with_rule_presentation() {
    let doc = projection_doc(&[("cap_a", "src_a", "ed_1")], &[], &[], true);
    let path = scratch("two-sources", &doc);
    let out = run(&[
        "rules",
        &path,
        "--project",
        "capture",
        "--columns",
        "capture_source",
        "--json",
    ]);
    let message = refusal(&out, "bad_argument");
    assert!(
        message.contains("rule_presentation"),
        "the refusal names the competing section: {message}"
    );
}

/// Every malformed projection invocation is a `bad_argument`: an arrow
/// not sourced at the projected object, an undeclared arrow, a missing
/// `--columns`, and an empty or duplicate column entry.
#[test]
fn projection_refuses_arrow_not_sourced_at_object() {
    let doc = projection_doc(&[("cap_a", "src_a", "ed_1")], &[], &[], false);
    let path = scratch("bad-arrows", &doc);

    let cases: &[(&[&str], &str)] = &[
        (
            &["--project", "capture", "--columns", "source_edition"],
            "source_edition",
        ),
        (&["--project", "capture", "--columns", "nope"], "nope"),
        (&["--project", "capture"], "columns"),
        (
            &["--project", "capture", "--columns", "capture_source,"],
            "empty",
        ),
        (
            &[
                "--project",
                "capture",
                "--columns",
                "capture_source,capture_source",
            ],
            "capture_source",
        ),
        (
            &[
                "--project",
                "capture",
                "--columns",
                "capture_source",
                "--action",
                "source_edition",
            ],
            "source_edition",
        ),
        (
            &[
                "--project",
                "capture",
                "--columns",
                "capture_source",
                "--action",
                "capture_source,capture_edition",
            ],
            "exactly one arrow",
        ),
        (
            &["--project", "missing_object", "--columns", "capture_source"],
            "missing_object",
        ),
        (&["--columns", "capture_source"], "--project"),
    ];
    for (args, needle) in cases {
        let mut full = vec!["rules", path.as_str()];
        full.extend_from_slice(args);
        full.push("--json");
        let out = run(&full);
        let message = refusal(&out, "bad_argument");
        assert!(
            message.to_lowercase().contains(&needle.to_lowercase()),
            "`{args:?}` names `{needle}`: {message}"
        );
    }
}
