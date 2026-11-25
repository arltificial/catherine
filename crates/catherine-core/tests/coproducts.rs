//! Purpose: the coproduct contract (format 1.5, openspec change
//! `add-coproduct-exhaustiveness`): `schema.coproducts` declares closed
//! variant sets over injection arrows; the section round-trips,
//! canonicalizes, is rejected at its node when malformed, executes
//! coverage and disjointness as existing-taxonomy violations, and is
//! hash-neutral when absent.
//!
//! Responsibilities: pin the canonical `(object, arrows, status)` order,
//! the repeated-declaration rule, the new minor (1.5 known, 1.6 rejected),
//! every `invalid_coproduct` class at its node (both non-injection
//! disjuncts: the wrong target and a source at the coproduct object), the
//! absent/empty hash guard, the two witness shapes — uncovered row
//! (coverage) and a row in both injections (disjointness) — their exact
//! JSON encodings, the artifact's `arrows` constraints, the proposed
//! label, and the unchanged equation-witness encoding.
//!
//! Rationale: coproducts are schema constraints (Decision 1): presence
//! moves the model hash, absence must not; coverage/disjointness are
//! existing-label violations (Decision 2); the grammar is `keys` again.

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::checks::check;
use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::findings::{
    CoproductWitness, DanglingWitness, EpistemicLabel, Finding, FindingKind, ViolationWitness,
};
use catherine_core::hash::model_hash;
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Coproduct, Ident, Model, RowId};
use catherine_core::parse::parse;
/// A format template with one `__SECTION__` slot (the coproduct section or
/// nothing) and the instance slots the check tests fill.
const DOC: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["C", "V1", "V2"],
    "arrows": [
      {"name": "i1", "src": "V1", "tgt": "C"},
      {"name": "i2", "src": "V2", "tgt": "C"},
      {"name": "f", "src": "C", "tgt": "V1"},
      {"name": "j", "src": "V1", "tgt": "V2"},
      {"name": "loop", "src": "C", "tgt": "C"}
    ],
    "equations": []__SECTION__
  },
  "instance": {
    "rows": {__ROWS__},
    "tables": {"i1": {__I1__}, "i2": {__I2__}}
  }
}"#;

/// The same schema without the direction-control arrows: every declared
/// arrow needs a table, so the check fixtures stay free of dangling cells.
const CHECK_DOC: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["C", "V1", "V2"],
    "arrows": [
      {"name": "i1", "src": "V1", "tgt": "C"},
      {"name": "i2", "src": "V2", "tgt": "C"}
    ],
    "equations": []__SECTION__
  },
  "instance": {
    "rows": {__ROWS__},
    "tables": {"i1": {__I1__}, "i2": {__I2__}}
  }
}"#;

const ACCEPTED: &str = r#"{"object": "C", "arrows": ["i1", "i2"], "status": "accepted"}"#;
const PROPOSED: &str = r#"{"object": "C", "arrows": ["i1", "i2"], "status": "proposed"}"#;
const EMPTY_ROWS: &str = r#""C": [], "V1": [], "V2": []"#;

/// Fill a template's four slots.
fn fill(template: &str, slots: (&str, &str, &str, &str)) -> String {
    let (section, rows, i1, i2) = slots;
    template
        .replace("__SECTION__", section)
        .replace("__ROWS__", rows)
        .replace("__I1__", i1)
        .replace("__I2__", i2)
}

/// The coproduct section carrying the given raw declarations.
fn section(raw: &str) -> String {
    format!(",\n    \"coproducts\": [\n      {raw}\n    ]")
}

fn parsed(text: &str) -> Model {
    match parse(text) {
        Ok(model) => model,
        Err(error) => panic!("the document fails to parse: {error:?}"),
    }
}

fn coproduct(arrows: &[&str], status: ConstraintStatus) -> Coproduct {
    Coproduct {
        object: Ident("C".to_string()),
        arrows: arrows
            .iter()
            .map(|arrow| Ident(arrow.to_string()))
            .collect(),
        status,
    }
}

const ORDER_ONE: &str = r#"{"object": "C", "arrows": ["i2", "i1"], "status": "proposed"},
           {"object": "C", "arrows": ["i1"], "status": "accepted"},
           {"object": "C", "arrows": ["i1"], "status": "accepted"}"#;
const ORDER_TWO: &str = r#"{"object": "C", "arrows": ["i1"], "status": "accepted"},
           {"object": "C", "arrows": ["i2", "i1"], "status": "proposed"},
           {"object": "C", "arrows": ["i1"], "status": "accepted"}"#;

/// Scenario "Coproducts round-trip and canonicalize": input order is
/// presentation, the parsed schema carries the declarations in canonical
/// `(object, arrows, status)` order, and a repeated declaration is kept.
#[test]
fn coproducts_round_trip_and_canonicalize() {
    let first = parsed(&fill(DOC, (&section(ORDER_ONE), EMPTY_ROWS, "", "")));
    let second = parsed(&fill(DOC, (&section(ORDER_TWO), EMPTY_ROWS, "", "")));
    assert_eq!(
        first.schema.coproducts,
        vec![
            coproduct(&["i1"], ConstraintStatus::Accepted),
            coproduct(&["i1"], ConstraintStatus::Accepted),
            coproduct(&["i2", "i1"], ConstraintStatus::Proposed),
        ],
        "canonical (object, arrows, status) order; a repeated declaration is kept"
    );
    assert_eq!(first.schema.coproducts, second.schema.coproducts);
    assert_eq!(
        canonical_bytes(&first),
        canonical_bytes(&second),
        "input order never reaches the canonical bytes"
    );
    let Ok(round_tripped) = parse(&serialize(&first)) else {
        panic!("canonical bytes fail to parse")
    };
    assert_eq!(round_tripped, first, "serialize/parse is a fixed point");
}

/// Scenario "Malformed coproduct is rejected at its node": each defect is
/// `invalid_coproduct` at the offending node, never a silent repair.
#[test]
fn malformed_coproduct_is_rejected_at_its_node() {
    let cases = [
        (
            r#"{"object": "Nope", "arrows": ["i1"]}"#,
            18,
            "undeclared object",
        ),
        (
            r#"{"object": "C", "arrows": ["q"]}"#,
            34,
            "undeclared arrow",
        ),
        (
            r#"{"object": "C", "arrows": ["f"]}"#,
            34,
            "arrow not an injection V_k -> C",
        ),
        (
            r#"{"object": "C", "arrows": ["j"]}"#,
            34,
            "arrow targeted away from the coproduct object",
        ),
        (
            r#"{"object": "C", "arrows": ["loop"]}"#,
            34,
            "arrow sourced at the coproduct object",
        ),
        (r#"{"object": "C", "arrows": []}"#, 33, "empty arrow list"),
        (
            r#"{"object": "C", "arrows": ["i1", "i1"]}"#,
            40,
            "duplicate arrow name",
        ),
    ];
    assert_eq!(cases.len(), 7, "every malformed-coproduct class is pinned");
    for (raw, column, what) in cases {
        assert_invalid((raw, column), what);
    }
}

/// One malformed declaration: the pinned node (line 14, six columns of
/// indent) and the stable code.
fn assert_invalid(drawn: (&str, usize), what: &str) {
    let (raw, column) = drawn;
    let text = fill(DOC, (&section(raw), EMPTY_ROWS, "", ""));
    let Err(ParseError::Semantic(error)) = parse(&text) else {
        panic!("{what}: not a semantic rejection");
    };
    assert_eq!(
        error.code,
        ErrorCode::InvalidCoproduct,
        "{what}: {}",
        error.message
    );
    assert_eq!(
        (error.line, error.column),
        (14, column),
        "{what}: the offending node"
    );
    assert!(
        error.message.contains("coproduct"),
        "{what}: the message names the coproduct: {}",
        error.message
    );
}

/// Scenario "Absent coproducts are hash-neutral": the pre-change corpus
/// fixture keeps its canonical bytes and hash, and an empty section is
/// omitted exactly like an absent one.
#[test]
fn absent_coproducts_are_hash_neutral() {
    let Ok(text) = std::fs::read_to_string(SMALL_FIXTURE) else {
        panic!("the corpus fixture is missing")
    };
    let Ok(model) = parse(&text) else {
        panic!("the pre-change corpus fixture fails to parse")
    };
    assert_eq!(
        canonical_bytes(&model),
        SMALL_PRE_CHANGE_CANONICAL,
        "an absent section is invisible to canonical bytes"
    );
    assert_eq!(
        model_hash(&model),
        SMALL_PRE_CHANGE_HASH,
        "an absent section is invisible to the model hash"
    );
    assert!(
        !canonical_bytes(&model).contains("coproducts"),
        "an absent section is never emitted"
    );

    let empty = parsed(&fill(DOC, (&section(""), EMPTY_ROWS, "", "")));
    let absent = parsed(&fill(DOC, ("", EMPTY_ROWS, "", "")));
    assert_eq!(canonical_bytes(&empty), canonical_bytes(&absent));
    assert_eq!(model_hash(&empty), model_hash(&absent));
    assert!(
        !canonical_bytes(&empty).contains("coproducts"),
        "an empty coproduct list is omitted from canonical bytes"
    );
}

/// Scenario "Current major parses": 1.6 is the known minor; 1.7 is not.
#[test]
fn current_major_parses_a_1_5_document() {
    let text = fill(DOC, (&section(ACCEPTED), EMPTY_ROWS, "", ""));
    let Ok(model) = parse(&text) else {
        panic!("1.5 is not an accepted additive minor")
    };
    assert_eq!(model.format_version.minor, 5);

    let too_new = text.replace("\"format_version\": \"1.5\"", "\"format_version\": \"1.7\"");
    let Err(ParseError::Semantic(error)) = parse(&too_new) else {
        panic!("an unknown minor is accepted")
    };
    assert_eq!(error.code, ErrorCode::UnsupportedVersion);
    assert_eq!((error.line, error.column), (2, 21));
    assert!(
        error.message.contains("known version is 1.6"),
        "the message states the cap: {}",
        error.message
    );
}

/// Parse -> build -> check over the check template.
fn findings(drawn: (&str, &str, &str, &str)) -> Vec<Finding> {
    let (coproducts, rows, i1, i2) = drawn;
    let text = fill(CHECK_DOC, (&section(coproducts), rows, i1, i2));
    let Ok(model) = parse(&text) else {
        panic!("the check document fails to parse")
    };
    let built = TypedInstance::build(&model.schema, &model.instance);
    check(&model.schema, &built)
}

fn coproduct_finding(drawn: (EpistemicLabel, usize), injections: &[&str]) -> Finding {
    let (label, row) = drawn;
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: 0,
            row: RowId(row),
            injections: injections
                .iter()
                .map(|name| Ident(name.to_string()))
                .collect(),
        })),
    }
}

/// Scenarios "Rogue variant is a violation with a witness" and "Dual
/// coverage is a violation naming both injections": an accepted
/// coproduct's executed violation names the uncovered `C` row, with an
/// injection pair exactly when that row is doubly covered.
#[test]
fn executed_coproduct_violations_pin_their_witnesses() {
    /// One scenario: the accepted document fragment, the supplied values,
    /// and the pinned witness.
    type Scenario<'a> = ((&'a str, &'a str, &'a str, &'a str), &'a [&'a str], &'a str);
    let cases: [Scenario; 2] = [
        (
            (
                ACCEPTED,
                r#""C": ["c1", "c2", "c3"], "V1": ["v1"], "V2": ["v2"]"#,
                r#""v1": "c1""#,
                r#""v2": "c2""#,
            ),
            &[],
            "the uncovered `c3` row is the only failure",
        ),
        (
            (
                ACCEPTED,
                r#""C": ["c1", "c2", "c3"], "V1": ["v1", "v2"], "V2": ["w1", "w2"]"#,
                r#""v1": "c1", "v2": "c3""#,
                r#""w1": "c3", "w2": "c2""#,
            ),
            &["i1", "i2"],
            "the doubly covered `c3` row names both injections",
        ),
    ];
    for (drawn, injections, what) in cases {
        let findings = findings(drawn);
        assert_eq!(
            findings,
            vec![coproduct_finding((EpistemicLabel::Executed, 2), injections)],
            "{what}: an accepted coproduct executes its coverage check"
        );
        assert_eq!(findings.len(), 1, "{what}: exactly one executed violation");
        assert_eq!(
            findings[0].label,
            EpistemicLabel::Executed,
            "{what}: the violation is executed, not hypothesized"
        );
    }
}

/// Scenario "Dangling injection cells are not double-reported": an
/// unresolved cell in a declared injection already carries its dangling
/// finding, so the incomplete declaration emits no coverage failure.
#[test]
fn dangling_injection_cell_is_not_double_reported() {
    let findings = findings((
        ACCEPTED,
        r#""C": ["c1", "c2", "c3"], "V1": ["v1", "v2"], "V2": ["w1"]"#,
        r#""v1": "c1""#,
        r#""w1": "c2""#,
    ));
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Dangling(DanglingWitness {
                arrow: Ident("i1".to_string()),
                src: RowId(1),
            }),
        }],
        "the dangling cell is reported once and never as a coverage failure"
    );
    assert_ne!(
        findings,
        Vec::<Finding>::new(),
        "the cell is still reported"
    );
    let nonempty = !findings.is_empty();
    assert!(nonempty, "the dangling cell is never silenced");
}

/// Scenario "Clean closed table is silent": every row is covered exactly
/// once, so the check emits nothing.
#[test]
fn clean_closed_table_is_silent() {
    let findings = findings((
        ACCEPTED,
        r#""C": ["c1", "c2"], "V1": ["v1"], "V2": ["w1"]"#,
        r#""v1": "c1""#,
        r#""w1": "c2""#,
    ));
    assert!(findings.is_empty(), "a clean closed table emits no finding");
}

#[path = "coproducts/proposed.rs"]
mod proposed;

/// The pre-change identity of `fixtures/parity/small/model.ct.json` (hash-neutral guard).
const SMALL_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/parity/small/model.ct.json"
);
const SMALL_PRE_CHANGE_HASH: &str =
    "sha256:fb033e3ccba19e8b558522062fea7d086a1bfc95e48ec04e50993cf6072e1311";
const SMALL_PRE_CHANGE_CANONICAL: &str = concat!(
    r#"{"format_version":"1.0","instance":{"rows":{"Constraint":["_0"],"Intent":["_0"]},"#,
    r#""tables":{"traces_to":{"_0":"_0"}}},"schema":{"arrows":[{"name":"traces_to","src":"Constraint","tgt":"Intent"}],"#,
    r#""equations":[],"objects":["Constraint","Intent"]}}"#
);
