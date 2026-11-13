//! Purpose: the hermetic semantic half of the `catherine rules --project`
//! acceptance surface (openspec change `add-rules-projection`, step 1) —
//! collisions, the observed-tuple domain, the declared `--values` gap, a
//! clean projection, and the `--action` arrow.
//!
//! Responsibilities: spawn the real binary (`CARGO_BIN_EXE_catherine`)
//! against inline `.ct.json` documents written to the temp directory;
//! assert exactly one `executed` violation per colliding row pair with
//! both rows in the witness, the observed-tuple domain's silence on
//! combinations no row has, the declared-`--values` gap, and the `--action`
//! arrow.
//!
//! Rationale: the projection's claim lives outside the model hash and the
//! ledger, so the only honest test is an end-to-end run of the real
//! binary: the fixture is the evidence. Assertions are structural
//! (`serde_json::Value`) and numeric (exit codes), never snapshots. The
//! grounded slice acceptance and the refusal surface keep their own
//! binaries (`cli_rules_projection_acceptance.rs`,
//! `cli_rules_projection_errors.rs`) so every file stays inside the
//! file-size gate.

mod common;

use common::{code, findings, kinds, parse_json, projection_doc, run, scratch, stderr, stdout};
use serde_json::Value;

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
        serde_json::json!([1]),
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
