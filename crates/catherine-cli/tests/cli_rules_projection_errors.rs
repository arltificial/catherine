//! Purpose: the refusal half of the `catherine rules --project` acceptance
//! surface (openspec change `add-rules-projection`, step 1) — a null
//! projected cell, an empty projection, two presentation sources, and
//! every malformed arrow/column spelling.
//!
//! Responsibilities: spawn the real binary (`CARGO_BIN_EXE_catherine`)
//! against hermetic inline documents and assert the labeled error class
//! (exit 1, error envelope on stderr, exact code, a message naming the
//! offending object/row/arrow) for `projection_incomplete`,
//! `empty_projection`, and `bad_argument`.
//!
//! Rationale: a projection never silently skips a row, never reports a
//! vacuous clean run, and never prefers one presentation source over
//! another — each is a refusal with its owned code, so the failure the
//! bridge exists to prevent stays visible. The semantic half lives in
//! `tests/cli_rules_projection.rs` (the file split keeps both binaries
//! inside the file-size gate).

mod common;

use common::{code, parse_json, projection_doc, run, scratch, stderr, stdout};
use std::process::Output;

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
/// `--columns`, an empty or duplicate column entry, and a multi-arrow
/// `--action`.
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
