//! Purpose: the instance-shape parse contract (catherine-3en, arc review
//! MEDIUM-005): an `instance` section this build cannot read is refused as
//! a labeled `syntax` error naming the unrecognized keys, never reported
//! as an empty instance.
//!
//! Responsibilities: pin the legacy per-object instance shape (the
//! `{"N": [{"id": …}]}` spelling with no `rows`/`tables`) at
//! `ParseError::Syntax` with a message naming the shape and the offending
//! keys, the two legal shapes (`{}`/empty `rows`+`tables`, and
//! unrecognized keys beside a populated section), and the additive
//! tolerance that the leftover keys stay outside the model (hash-neutral).
//!
//! Rationale: unknown keys are deliberately ignored (additive minors add
//! optional fields), so the parser must distinguish that tolerance from a
//! totally unrecognized shape. Reading the legacy shape as an empty
//! instance checks clean and exits 0 — a class-A fail-open the negative
//! matrix (`crates/catherine-cli/tests/negative_matrix.rs`,
//! `gap_suite/wrong_shape_instance.ct.json`) and the gap-suite row
//! `vacuous-clean-instances` now pin from the other side.

use catherine_core::error::ParseError;
use catherine_core::hash::model_hash;
use catherine_core::model::Ident;
use catherine_core::parse::parse;

/// A minimal 1.3 document whose `instance` section is supplied verbatim.
fn document(instance: &str) -> String {
    format!(
        r#"{{
  "format_version": "1.3",
  "schema": {{
    "objects": ["N"],
    "arrows": [{{"name": "f", "src": "N", "tgt": "N"}}],
    "equations": [{{"lhs": ["f", "f"], "rhs": ["f"], "rationale": "idempotent", "status": "accepted"}}]
  }},
  "instance": {instance}
}}"#
    )
}

fn syntax_message(text: &str) -> String {
    match parse(text) {
        Err(ParseError::Syntax { message, .. }) => message,
        other => panic!("the legacy instance shape is a syntax rejection, got {other:?}"),
    }
}

/// The reported defect: a legacy per-object instance is refused with a
/// message that names the missing keys and the keys it saw instead.
#[test]
fn legacy_per_object_instance_is_refused_naming_the_keys() {
    let message = syntax_message(&document(
        r#"{"N": [{"id": "a", "f": "b"}, {"id": "b", "f": "a"}]}"#,
    ));
    assert!(
        message.contains("no `rows` or `tables`"),
        "the message names the missing keys: {message}"
    );
    assert!(
        message.contains("unrecognized keys: `N`"),
        "the message names the unrecognized key: {message}"
    );
}

/// Several unrecognized keys are all named, in canonical (sorted) order.
#[test]
fn every_unrecognized_key_is_named() {
    let message = syntax_message(&document(r#"{"M": [], "N": [{"id": "a"}]}"#));
    assert!(
        message.contains("unrecognized keys: `M`, `N`"),
        "every unrecognized key is named: {message}"
    );
}

/// An empty instance section stays legal — an empty instance is a present,
/// readable value, not an unreadable shape.
#[test]
fn empty_instance_section_stays_legal() {
    let model = parse(&document("{}")).expect("an empty instance section parses");
    assert!(
        model.instance.rows.values().all(Vec::is_empty),
        "no object carries rows"
    );
    let empty_tables = parse(&document(r#"{"rows": {}, "tables": {}}"#))
        .expect("explicitly empty rows and tables parse");
    assert!(empty_tables.instance.values.values().all(Vec::is_empty));
}

/// The additive-minor tolerance is unchanged: a future optional key beside
/// a recognizable section parses and never reaches the model.
#[test]
fn unrecognized_keys_beside_a_recognized_section_stay_ignored() {
    let base = document(r#"{"rows": {"N": ["a"]}, "tables": {"f": {"a": "a"}}}"#);
    let extended = document(
        r#"{"rows": {"N": ["a"]}, "tables": {"f": {"a": "a"}}, "future_optional": {"any": true}}"#,
    );
    let base_model = parse(&base).expect("the recognized section parses");
    let extended_model = parse(&extended).expect("an additive extra beside content parses");
    assert_eq!(
        base_model, extended_model,
        "an unrecognized key with no semantics is not model data"
    );
    assert_eq!(
        model_hash(&base_model),
        model_hash(&extended_model),
        "the ignored key is hash-neutral"
    );
    let n = Ident("N".to_string());
    assert_eq!(base_model.instance.rows[&n].len(), 1);
    assert_eq!(base_model.instance.values[&Ident("f".to_string())].len(), 1);
}
