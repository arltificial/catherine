//! Purpose: the unknown-field parse contract (catherine-bak): a field the
//! accepted format version does not define is a labeled `unknown_field`
//! error at its line, never silently dropped.
//!
//! Responsibilities: walk every raw document node (top level, schema,
//! arrow, equation, key, coproduct, instance, instance map, schema map,
//! rule presentation, predicate) with one otherwise-valid document each,
//! and pin the code, the offending line, and the message naming the field;
//! keep the empty/absent sections legal so strictness never rejects a
//! well-formed document.
//!
//! Rationale: the parser used to ignore unknown keys as deliberate
//! additive-minor policy. An external black-box probe showed the cost: a
//! misspelled `equations` key made a violated model report clean at exit
//! 0. Additive minors above `KNOWN_MINOR` are already rejected by the
//! version gate, so inside an accepted version an unknown field can only
//! be a mistake — the no-silent-drop principle wins.

use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::parse::parse;

/// The valid baseline every case perturbs: objects `A`/`B`/`C`, arrows
/// `f: A->B` and `g: A->C`, one equation, one key, one coproduct.
const BASE: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["A", "B", "C"],
    "arrows": [
      {"name": "f", "src": "A", "tgt": "B"},
      {"name": "g", "src": "A", "tgt": "C"}
    ],
    "equations": [
      {"lhs": ["f"], "rhs": ["f"], "rationale": "reflexive", "status": "accepted"}
    ],
    "keys": [{"object": "A", "arrows": ["f"], "status": "accepted"}],
    "coproducts": [{"object": "C", "arrows": ["g"], "status": "accepted"}]
  },
  "instance": {
    "rows": {"A": ["a1"], "B": ["b1"], "C": ["c1"]},
    "tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}
  }
}"#;

/// Replace `needle` with `replacement` in the baseline — each case adds
/// exactly one unknown field.
fn perturbs(needle: &str, replacement: &str) -> String {
    assert!(BASE.contains(needle), "baseline carries {needle}");
    BASE.replacen(needle, replacement, 1)
}

/// One case: the perturbed document and the unknown field it carries.
struct Case {
    document: String,
    field: &'static str,
}

fn cases() -> Vec<Case> {
    let mut all = document_cases();
    all.extend(declared_section_cases());
    all
}

/// The document's own nodes: top level, schema, arrow, equation, key,
/// coproduct, instance.
fn document_cases() -> Vec<Case> {
    vec![
        Case {
            document: perturbs(
                r#"{
  "format_version": "1.5","#,
                r#"{
  "format_version": "1.5",
  "notes": "an unknown top-level field","#,
            ),
            field: "notes",
        },
        Case {
            document: perturbs(
                r#""objects": ["A", "B", "C"],"#,
                r#""objects": ["A", "B", "C"],
    "mystery_section": true,"#,
            ),
            field: "mystery_section",
        },
        Case {
            document: perturbs(
                r#"{"name": "f", "src": "A", "tgt": "B"}"#,
                r#"{"name": "f", "src": "A", "tgt": "B", "label": "the f arrow"}"#,
            ),
            field: "label",
        },
        Case {
            document: perturbs(
                r#"{"lhs": ["f"], "rhs": ["f"], "rationale": "reflexive", "status": "accepted"}"#,
                r#"{"lhs": ["f"], "rhs": ["f"], "justification": "reflexive", "status": "accepted"}"#,
            ),
            field: "justification",
        },
        Case {
            document: perturbs(
                r#"{"object": "A", "arrows": ["f"], "status": "accepted"}"#,
                r#"{"object": "A", "arrows": ["f"], "status": "accepted", "rationale": "why"}"#,
            ),
            field: "rationale",
        },
        Case {
            document: perturbs(
                r#"{"object": "C", "arrows": ["g"], "status": "accepted"}"#,
                r#"{"object": "C", "arrows": ["g"], "status": "accepted", "note": "coverage"}"#,
            ),
            field: "note",
        },
        Case {
            document: perturbs(
                r#""tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}"#,
                r#""tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}},
    "claimed_extra": true"#,
            ),
            field: "claimed_extra",
        },
    ]
}

/// The additive sections and the predicate language: instance map, schema
/// map, and a predicate field its `kind` does not define.
fn declared_section_cases() -> Vec<Case> {
    let rule = r#"{"format_version": "1.5",
  "schema": {"objects": ["A"], "arrows": [], "equations": []},
  "instance": {"rows": {"A": []}, "tables": {}},
  "rule_presentation": {
    "domain": "A",
    "columns": [{"name": "x", "type": "int"}],
    "rules": [{"name": "r", "predicate": {"kind": "cmp", "column": "x", "op": "gt", "value": 1, "extra": 2}, "action": "a"}]
  }
}"#;
    vec![
        Case {
            document: perturbs(
                r#"    "tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}
  }"#,
                r#"    "tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}
  },
  "instance_map": {"maps": [{"object": "A", "rows": {}, "note": "x"}]}"#,
            ),
            field: "note",
        },
        Case {
            document: perturbs(
                r#"    "tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}
  }"#,
                r#"    "tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}
  },
  "schema_map": {"objects": {}, "arrows": {}, "extra": true}"#,
            ),
            field: "extra",
        },
        Case {
            document: rule.to_string(),
            field: "extra",
        },
    ]
}

/// The accepted format version defines the document's fields; anything
/// else is `unknown_field` at the line where it appears.
#[test]
fn unknown_fields_are_rejected_at_their_nodes() {
    for case in cases() {
        let field = case.field;
        let expected_line = case
            .document
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains(&format!("\"{field}\"")))
            .map(|(index, _)| index + 1)
            .last()
            .unwrap_or_else(|| panic!("case document spells `{field}`"));
        match parse(&case.document) {
            Err(ParseError::Semantic(error)) => {
                assert_eq!(
                    error.code,
                    ErrorCode::UnknownField,
                    "`{field}` is an unknown_field rejection: {error:?}"
                );
                assert_eq!(
                    error.line, expected_line,
                    "the error points at the line of `{field}`: {error:?}"
                );
                assert!(error.column >= 1, "the column is 1-based: {error:?}");
                assert!(
                    error.message.contains(field),
                    "the message names `{field}`: {error:?}"
                );
            }
            Ok(_) => panic!("unknown field `{field}` was silently accepted"),
            Err(other) => panic!("`{field}` must be unknown_field, got {other:?}"),
        }
    }
}

/// Strictness must not reject a well-formed document — the baseline above
/// is the guard: any over-rejection breaks every case's premise.
#[test]
fn a_well_formed_document_still_parses() {
    let model = parse(BASE).expect("the baseline document is well formed");
    assert_eq!(model.instance.rows.len(), 3, "all three objects carry rows");
}

/// An empty or absent instance section stays legal: strictness is about
/// undefined fields, not about empty values.
#[test]
fn empty_instance_section_stays_legal() {
    let empty: String = BASE.replace(
        r#""rows": {"A": ["a1"], "B": ["b1"], "C": ["c1"]},
    "tables": {"f": {"a1": "b1"}, "g": {"a1": "c1"}}"#,
        r#""rows": {},
    "tables": {}"#,
    );
    let model = parse(&empty).expect("an empty instance parses");
    assert!(model.instance.rows.values().all(Vec::is_empty));
}
