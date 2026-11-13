//! Unit tests for the projection translation: dense row order, the
//! conjunctive predicate shape, the identity/`--action` action, the exact
//! observed-tuple domain unioned with `--values`, and the not-sourced
//! arrow refusal. The end-to-end surface lives in
//! `tests/cli_rules_projection*.rs`.

use super::*;
use catherine_core::model::{CmpOp, ColumnType, Ident, Literal, Policy, Predicate};
use std::collections::BTreeMap;

/// Two captures colliding on `(source, edition)`, plus one distinct row;
/// every cell resolves.
fn fixture() -> (Model, RowNames) {
    let doc = r#"{
        "format_version": "1.0",
        "schema": {
            "objects": ["capture", "source", "edition", "batch"],
            "arrows": [
                {"name": "capture_source", "src": "capture", "tgt": "source"},
                {"name": "capture_edition", "src": "capture", "tgt": "edition"},
                {"name": "capture_batch", "src": "capture", "tgt": "batch"},
                {"name": "source_edition", "src": "source", "tgt": "edition"}
            ],
            "equations": []
        },
        "instance": {
            "rows": {
                "capture": ["cap_c", "cap_b", "cap_a"],
                "source": ["src_a", "src_b"],
                "edition": ["ed_1", "ed_2"],
                "batch": ["b1", "b2"]
            },
            "tables": {
                "capture_source": {"cap_a": "src_a", "cap_b": "src_a", "cap_c": "src_b"},
                "capture_edition": {"cap_a": "ed_1", "cap_b": "ed_1", "cap_c": "ed_2"},
                "capture_batch": {"cap_a": "b1", "cap_b": "b1", "cap_c": "b2"}
            }
        }
    }"#;
    catherine_core::parse::parse_with_rows(doc).expect("fixture parses")
}

fn projection<'a>(columns: &'a str, action: Option<&'a str>) -> Projection<'a> {
    Projection {
        object: "capture",
        columns: Some(columns),
        action,
    }
}

fn rule_names(presentation: &RulePresentation) -> Vec<&str> {
    presentation
        .rules
        .iter()
        .map(|rule| rule.name.0.as_str())
        .collect()
}

fn first_predicate() -> Predicate {
    Predicate::And(vec![
        Predicate::Cmp {
            column: Ident("capture_source".to_string()),
            op: CmpOp::Eq,
            value: Literal::String("src_a".to_string()),
        },
        Predicate::Cmp {
            column: Ident("capture_edition".to_string()),
            op: CmpOp::Eq,
            value: Literal::String("ed_1".to_string()),
        },
    ])
}

fn assert_columns(presentation: &RulePresentation) {
    assert_eq!(
        presentation.columns,
        BTreeMap::from([
            (Ident("capture_source".to_string()), ColumnType::String),
            (Ident("capture_edition".to_string()), ColumnType::String),
        ])
    );
}

/// Rules follow dense row order; predicates are conjunctive equalities in
/// the given column order; the action is identity by default and the
/// arrow target otherwise.
#[test]
fn rows_become_rules_in_dense_order() {
    let (model, rows) = fixture();
    let (presentation, _) = project(
        &model,
        &rows,
        &projection("capture_source,capture_edition", None),
        &[],
    )
    .expect("projection succeeds");
    assert_eq!(rule_names(&presentation), vec!["cap_a", "cap_b", "cap_c"]);
    assert_eq!(presentation.policy, Some(Policy::DeclaredDisjoint));
    assert_eq!(presentation.rules[0].action, "cap_a");
    assert_eq!(
        presentation.rules[0].predicate,
        first_predicate(),
        "predicate order follows --columns order"
    );
    assert_columns(&presentation);
}

/// `--action` replaces the identity action with the arrow's target row
/// name.
#[test]
fn the_action_arrow_projects_target_row_names() {
    let (model, rows) = fixture();
    let (presentation, _) = project(
        &model,
        &rows,
        &projection("capture_source", Some("capture_batch")),
        &[],
    )
    .expect("projection succeeds");
    let actions: Vec<&str> = presentation
        .rules
        .iter()
        .map(|rule| rule.action.as_str())
        .collect();
    assert_eq!(actions, vec!["b1", "b1", "b2"]);
}

/// The observed tuple domain is deduplicated and canonically sorted;
/// a declared product input is unioned in.
#[test]
fn the_domain_is_the_observed_set_plus_any_declaration() {
    let (model, rows) = fixture();
    let (presentation, domain) = project(
        &model,
        &rows,
        &projection("capture_source,capture_edition", None),
        &[],
    )
    .expect("projection succeeds");
    let observed = vec![
        vec![
            Literal::String("ed_1".to_string()),
            Literal::String("src_a".to_string()),
        ],
        vec![
            Literal::String("ed_2".to_string()),
            Literal::String("src_b".to_string()),
        ],
    ];
    assert_eq!(domain.inputs(), observed, "deduped, in column-key order");

    let (_, declared) = project(
        &model,
        &rows,
        &projection("capture_source,capture_edition", None),
        &[
            "capture_source=src_a,src_b".to_string(),
            "capture_edition=ed_1,ed_2".to_string(),
        ],
    )
    .expect("projection succeeds");
    assert_eq!(
        declared.inputs().len(),
        4,
        "the declared product is unioned"
    );
    assert!(presentation
        .columns
        .contains_key(&Ident("capture_source".into())));
}

/// A schema arrow is not automatically a projection arrow: it must start
/// at the projected object.
#[test]
fn an_arrow_sourced_elsewhere_is_refused() {
    let (model, rows) = fixture();
    let bad = Projection {
        object: "capture",
        columns: Some("source_edition"),
        action: None,
    };
    let error = project(&model, &rows, &bad, &[]).expect_err("not sourced at capture");
    assert_eq!(error.code, "bad_argument");
    assert!(error.message.contains("source_edition"));
}
