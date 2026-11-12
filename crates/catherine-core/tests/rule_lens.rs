//! Purpose: the rule-lens phase-A parse contract — format 1.2 documents
//! carry an optional `rule_presentation` section (input domain X as typed
//! attribute columns, rules with decidable predicates and required
//! actions, an explicit resolution policy), and its presence never moves
//! an existing document's identity.
//!
//! Responsibilities: pin the hash-stability guard (a pre-existing corpus
//! fixture's `model_hash`, computed before the section landed, asserted
//! unchanged — the section is skip-serialized when absent), the four
//! golden fixtures (one per rule-table bug class) parsing with their
//! documented policy and rule order, the closed predicate language, the
//! labeled validation errors naming the offending element, the additive
//! minor gate (1.2 known, 1.3 rejected), and the section's serialize/parse
//! fixed point.
//!
//! Rationale: openspec change `add-rule-lens` phase A (design.md
//! Decisions 1/2/3) — no implicit precedence, a required total action, and
//! a tiny decidable predicate language. The pinned hash is the P6 ledger's
//! identity contract: content addressing must not shift for documents that
//! predate the rule lens.

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::error::{ErrorCode, ParseError, SemanticError};
use catherine_core::hash::model_hash;
use catherine_core::model::{
    CmpOp, ColumnType, Ident, Literal, Policy, Predicate, RulePresentation, Version,
};
use catherine_core::parse::parse;

const SMALL_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/parity/small/model.ct.json"
);

/// The pre-change identity of `fixtures/parity/small/model.ct.json`,
/// computed before the rule-presentation section existed. If this value
/// moves, an existing document's canonical bytes moved — the ledger's
/// content-addressing is broken and the change must not ship.
const SMALL_PRE_CHANGE_HASH: &str =
    "sha256:fb033e3ccba19e8b558522062fea7d086a1bfc95e48ec04e50993cf6072e1311";

/// A minimal 1.2 document with a placeholder for the presentation section.
const DOC: &str = r#"{
  "format_version": "1.2",
  "schema": {"objects": ["Input"], "arrows": [], "equations": []},
  "instance": {"rows": {}, "tables": {}},
  "rule_presentation": __SECTION__
}"#;

/// A well-formed single-rule presentation over one int column.
const VALID: &str = r#"{
  "domain": "Input",
  "columns": [{"name": "age", "type": "int"}],
  "rules": [{"name": "kid", "predicate": {"kind": "cmp", "column": "age", "op": "lt", "value": 18}, "action": "kid"}],
  "policy": "PriorityOrder"
}"#;

fn doc(section: &str) -> String {
    DOC.replace("__SECTION__", section)
}

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/rule_lens/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(path).expect("golden fixture exists")
}

fn presentation(model: &catherine_core::model::Model) -> &RulePresentation {
    model
        .rule_presentation
        .as_ref()
        .expect("the fixture declares a rule presentation")
}

/// The labeled semantic rejection of one malformed presentation.
fn rejection(section: &str) -> SemanticError {
    match parse(&doc(section)) {
        Err(ParseError::Semantic(semantic)) => semantic,
        other => panic!("expected a labeled rejection, got {other:?}"),
    }
}

#[test]
fn existing_documents_hash_as_before_the_rule_presentation_section() {
    let text = std::fs::read_to_string(SMALL_FIXTURE).expect("corpus fixture exists");
    let model = parse(&text).expect("corpus fixture parses");
    assert!(
        model.rule_presentation.is_none(),
        "a pre-rule document carries no rule presentation"
    );
    assert_eq!(
        model_hash(&model),
        SMALL_PRE_CHANGE_HASH,
        "the rule-presentation section must be invisible to existing documents' identity"
    );
    assert!(
        !canonical_bytes(&model).contains("rule_presentation"),
        "an absent section is skip-serialized, never emitted empty"
    );
}

#[test]
fn the_four_golden_fixtures_parse_with_their_declared_shape() {
    // (file, policy, rule names in priority order) — one per bug class;
    // the expected taxonomy mapping lives in fixtures/rule_lens/README.md.
    let cases: [(&str, Option<Policy>, &[&str]); 4] = [
        (
            "gap.ct.json",
            Some(Policy::PriorityOrder),
            &["kid", "adult"],
        ),
        (
            "overlap-disjoint.ct.json",
            Some(Policy::DeclaredDisjoint),
            &["small", "large"],
        ),
        ("shadow.ct.json", Some(Policy::FirstMatch), &["all", "even"]),
        (
            "derivable.ct.json",
            Some(Policy::FirstMatch),
            &["fizzbuzz", "fizz", "buzz", "plain"],
        ),
    ];
    for (file, policy, rules) in cases {
        let model = parse(&fixture(file)).expect("golden fixture parses");
        assert_eq!(model.format_version, Version { major: 1, minor: 2 });
        let declared = presentation(&model);
        assert_eq!(declared.domain, Ident("Input".to_string()));
        assert_eq!(declared.policy, policy, "{file}: declared policy");
        assert_eq!(
            declared
                .rules
                .iter()
                .map(|rule| rule.name.0.as_str())
                .collect::<Vec<_>>(),
            rules,
            "{file}: rule order is the priority chain"
        );
        assert!(
            declared.rules.iter().all(|rule| !rule.action.is_empty()),
            "{file}: every rule assigns an action"
        );
        let bytes = serialize(&model);
        let reparsed = parse(&bytes).expect("canonical bytes parse");
        assert_eq!(reparsed, model, "{file}: serialize/parse is a fixed point");
        assert_eq!(model_hash(&model), model_hash(&reparsed));
    }
}

#[test]
fn the_golden_fixtures_use_the_documented_predicate_language() {
    let gap = parse(&fixture("gap.ct.json")).expect("gap fixture parses");
    let adult = presentation(&gap).rules[1].predicate.clone();
    assert_eq!(
        adult,
        Predicate::And(vec![
            Predicate::Cmp {
                column: Ident("age".to_string()),
                op: CmpOp::Ge,
                value: Literal::Int(18),
            },
            Predicate::Cmp {
                column: Ident("age".to_string()),
                op: CmpOp::Lt,
                value: Literal::Int(65),
            },
        ]),
        "conjunction of typed comparisons"
    );
    let derivable = parse(&fixture("derivable.ct.json")).expect("derivable fixture parses");
    assert_eq!(
        presentation(&derivable).rules[0].predicate,
        Predicate::Mod {
            column: Ident("n".to_string()),
            modulus: 15,
            equals: 0,
        },
        "integer modulus test"
    );
    let overlap = parse(&fixture("overlap-disjoint.ct.json")).expect("overlap fixture parses");
    assert_eq!(
        presentation(&overlap).rules[0].predicate,
        Predicate::Cmp {
            column: Ident("amount".to_string()),
            op: CmpOp::Le,
            value: Literal::Int(100),
        },
        "typed comparison"
    );
}

#[test]
fn a_declared_presentation_parses_into_typed_columns_and_rules() {
    let model = parse(&doc(VALID)).expect("valid presentation parses");
    let declared = presentation(&model);
    assert_eq!(declared.domain, Ident("Input".to_string()));
    assert_eq!(
        declared.columns.get(&Ident("age".to_string())),
        Some(&ColumnType::Int)
    );
    assert_eq!(declared.policy, Some(Policy::PriorityOrder));
    assert_eq!(declared.rules.len(), 1);
    assert_eq!(declared.rules[0].action, "kid");
}

#[test]
fn an_absent_policy_stays_absent_never_defaulted() {
    let no_policy = VALID.replace(",\n  \"policy\": \"PriorityOrder\"", "");
    let model = parse(&doc(&no_policy)).expect("a policy-less table parses");
    assert_eq!(
        presentation(&model).policy,
        None,
        "no implicit precedence: an absent policy stays absent"
    );
    let bytes = serialize(&model);
    assert!(
        !bytes.contains("\"policy\""),
        "an undeclared policy is not emitted: {bytes}"
    );
    assert_eq!(parse(&bytes).expect("canonical bytes parse"), model);
    // An explicit null is the same non-declaration, never a default policy.
    let null_policy = VALID.replace(r#""PriorityOrder""#, "null");
    let model = parse(&doc(&null_policy)).expect("an explicit null policy parses");
    assert_eq!(presentation(&model).policy, None);
}

#[test]
fn column_order_is_presentation_but_rule_order_is_identity() {
    let first = r#"{
      "domain": "Input",
      "columns": [{"name": "n", "type": "int"}, {"name": "flag", "type": "bool"}],
      "rules": [
        {"name": "zero", "predicate": {"kind": "and", "args": [
          {"kind": "cmp", "column": "n", "op": "eq", "value": 0},
          {"kind": "cmp", "column": "flag", "op": "eq", "value": true}]}, "action": "zero"},
        {"name": "rest", "predicate": {"kind": "cmp", "column": "n", "op": "ge", "value": 1}, "action": "rest"}
      ]
    }"#;
    let columns_swapped = first.replace(
        r#"[{"name": "n", "type": "int"}, {"name": "flag", "type": "bool"}]"#,
        r#"[{"name": "flag", "type": "bool"}, {"name": "n", "type": "int"}]"#,
    );
    let a = parse(&doc(first)).expect("first presentation parses");
    let b = parse(&doc(&columns_swapped)).expect("column order is presentation");
    assert_eq!(
        canonical_bytes(&a),
        canonical_bytes(&b),
        "column declaration order never reaches the bytes (columns are keyed by name)"
    );
    assert_eq!(model_hash(&a), model_hash(&b));
    let rules_swapped = first.replace(
        r#"{"name": "zero", "predicate": {"kind": "and", "args": [
          {"kind": "cmp", "column": "n", "op": "eq", "value": 0},
          {"kind": "cmp", "column": "flag", "op": "eq", "value": true}]}, "action": "zero"},
        {"name": "rest", "predicate": {"kind": "cmp", "column": "n", "op": "ge", "value": 1}, "action": "rest"}"#,
        r#"{"name": "rest", "predicate": {"kind": "cmp", "column": "n", "op": "ge", "value": 1}, "action": "rest"},
        {"name": "zero", "predicate": {"kind": "and", "args": [
          {"kind": "cmp", "column": "n", "op": "eq", "value": 0},
          {"kind": "cmp", "column": "flag", "op": "eq", "value": true}]}, "action": "zero"}"#,
    );
    let c = parse(&doc(&rules_swapped)).expect("rule order is semantic, not sorted away");
    assert_ne!(
        model_hash(&a),
        model_hash(&c),
        "rule order IS the priority chain, so it is part of the identity"
    );
}

#[test]
fn the_additive_minor_gate_knows_1_2() {
    let too_new = doc(VALID).replace("\"format_version\": \"1.2\"", "\"format_version\": \"1.3\"");
    let ParseError::Semantic(semantic) = parse(&too_new).expect_err("1.3 is unknown") else {
        panic!("an unknown minor is a labeled rejection");
    };
    assert_eq!(semantic.code, ErrorCode::UnsupportedVersion);
    assert!(
        semantic.message.contains("known version is 1.2"),
        "the gate names the known version: {}",
        semantic.message
    );
}

#[test]
fn malformed_presentations_are_rejected_with_labeled_codes() {
    let cases: [(String, ErrorCode, &str); 8] = [
        (
            VALID.replace(r#", "action": "kid""#, ""),
            ErrorCode::MissingAction,
            "kid",
        ),
        (
            VALID.replace(r#""action": "kid""#, r#""action": "  ""#),
            ErrorCode::MissingAction,
            "kid",
        ),
        (
            VALID.replace(
                r#""policy": "PriorityOrder""#,
                r#""policy": "priorityOrder""#,
            ),
            ErrorCode::InvalidPolicy,
            "priorityOrder",
        ),
        (
            VALID.replace(r#""column": "age", "op""#, r#""column": "nope", "op""#),
            ErrorCode::UnknownColumn,
            "nope",
        ),
        (
            VALID.replace(r#""value": 18"#, r#""value": "eighteen""#),
            ErrorCode::InvalidPredicate,
            "int",
        ),
        (
            VALID.replace(r#""kind": "cmp""#, r#""kind": "regex""#),
            ErrorCode::InvalidPredicate,
            "regex",
        ),
        (
            VALID.replace(r#""domain": "Input""#, r#""domain": "Nope""#),
            ErrorCode::UndeclaredRuleDomain,
            "Nope",
        ),
        (
            VALID.replace(r#""type": "int""#, r#""type": "float""#),
            ErrorCode::InvalidColumnType,
            "float",
        ),
    ];
    for (section, code, needle) in cases {
        let semantic = rejection(&section);
        assert_eq!(semantic.code, code, "{}", semantic.message);
        assert!(
            semantic.message.contains(needle),
            "the rejection names the offending element `{needle}`: {}",
            semantic.message
        );
    }
}

#[test]
fn the_predicate_language_is_closed_and_well_typed() {
    // An empty boolean combination is rejected rather than materialized.
    let empty_args = r#"{
      "domain": "Input",
      "columns": [{"name": "n", "type": "int"}],
      "rules": [{"name": "r", "predicate": {"kind": "and", "args": []}, "action": "a"}]
    }"#;
    assert_eq!(rejection(empty_args).code, ErrorCode::InvalidPredicate);
    // `mod` is an int-column operation only.
    let mod_on_bool = r#"{
      "domain": "Input",
      "columns": [{"name": "b", "type": "bool"}],
      "rules": [{"name": "r", "predicate": {"kind": "mod", "column": "b", "modulus": 2, "equals": 0}, "action": "a"}]
    }"#;
    assert_eq!(rejection(mod_on_bool).code, ErrorCode::InvalidPredicate);
    // A modulus must be positive (non-negative remainder, no zero division).
    let zero_modulus = r#"{
      "domain": "Input",
      "columns": [{"name": "n", "type": "int"}],
      "rules": [{"name": "r", "predicate": {"kind": "mod", "column": "n", "modulus": 0, "equals": 0}, "action": "a"}]
    }"#;
    assert_eq!(rejection(zero_modulus).code, ErrorCode::InvalidPredicate);
    // Bool columns are two-valued: ordering comparisons are rejected.
    let bool_ordering = r#"{
      "domain": "Input",
      "columns": [{"name": "b", "type": "bool"}],
      "rules": [{"name": "r", "predicate": {"kind": "cmp", "column": "b", "op": "lt", "value": true}, "action": "a"}]
    }"#;
    assert_eq!(rejection(bool_ordering).code, ErrorCode::InvalidPredicate);
    // Duplicate rule and column names are duplicate idents.
    let duplicate_rule = r#"{
      "domain": "Input",
      "columns": [{"name": "n", "type": "int"}],
      "rules": [
        {"name": "r", "predicate": {"kind": "cmp", "column": "n", "op": "ge", "value": 0}, "action": "a"},
        {"name": "r", "predicate": {"kind": "cmp", "column": "n", "op": "lt", "value": 0}, "action": "b"}
      ]
    }"#;
    assert_eq!(rejection(duplicate_rule).code, ErrorCode::DuplicateIdent);
    let duplicate_column = r#"{
      "domain": "Input",
      "columns": [{"name": "n", "type": "int"}, {"name": "n", "type": "bool"}],
      "rules": []
    }"#;
    assert_eq!(rejection(duplicate_column).code, ErrorCode::DuplicateIdent);
}
