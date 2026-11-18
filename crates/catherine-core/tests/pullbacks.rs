//! Purpose: the pullback declaration contract (format 1.6, openspec change
//! `add-pullback-checks`): `schema.pullbacks` declares a cospan of arrows
//! with one common target and two legs from a single apex object; the
//! section round-trips, canonicalizes, is rejected at its node when
//! malformed, and is hash-neutral when absent.
//!
//! Responsibilities: pin the canonical `(apex, legs, cospan, status)`
//! order, the repeated-declaration rule, the 1.6 minor and the accepted
//! status default, every `invalid_pullback` class at its offending node
//! (undeclared apex/arrow, leg sourcing, leg/cospan composability, leg
//! and cospan list lengths, unequal cospan targets, repeated arrow), the
//! absent/empty hash guard, and the continued rejection of a `cones`
//! namespace.
//!
//! Rationale: a pullback is a schema constraint: presence moves the model
//! hash, absence must not, and a malformed declaration is never repaired
//! (the `invalid_coproduct` discipline). The declaration grammar mirrors
//! `coproducts`, so one canonicalization and one error path serves both.

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::hash::model_hash;
use catherine_core::model::{ConstraintStatus, Ident, Model, Pullback};
use catherine_core::parse::parse;

/// A format 1.6 template with one `__SECTION__` slot (the pullback
/// section or nothing). Every arrow the malformed cases name is declared
/// here: `p1`/`p2`/`r1`/`r2` are apex legs, `f`/`g`/`h` share target
/// `C`, `q` is a third apex leg targeting `C` (not composable with `g`),
/// and `k` is a cospan-shaped arrow whose target differs from `f`'s.
const DOC: &str = r#"{
  "format_version": "1.6",
  "schema": {
    "objects": ["A", "B", "C", "P", "Q"],
    "arrows": [
      {"name": "f", "src": "A", "tgt": "C"},
      {"name": "g", "src": "B", "tgt": "C"},
      {"name": "p1", "src": "P", "tgt": "A"},
      {"name": "p2", "src": "P", "tgt": "B"},
      {"name": "r1", "src": "Q", "tgt": "A"},
      {"name": "r2", "src": "Q", "tgt": "B"},
      {"name": "q", "src": "P", "tgt": "C"},
      {"name": "h", "src": "A", "tgt": "C"},
      {"name": "k", "src": "B", "tgt": "A"}
    ],
    "equations": []__SECTION__
  },
  "instance": {
    "rows": {"A": [], "B": [], "C": [], "P": [], "Q": []},
    "tables": {
      "f": {}, "g": {}, "p1": {}, "p2": {}, "r1": {}, "r2": {},
      "q": {}, "h": {}, "k": {}
    }
  }
}"#;

/// Fill the template's section slot.
fn fill(section: &str) -> String {
    DOC.replace("__SECTION__", section)
}

/// The pullback section carrying the given raw declarations.
fn section(raw: &str) -> String {
    format!(",\n    \"pullbacks\": [\n      {raw}\n    ]")
}

fn parsed(text: &str) -> Model {
    parse(text).expect("the document parses")
}

fn pullback(apex: &str, legs: &[&str], cospan: &[&str], status: ConstraintStatus) -> Pullback {
    Pullback {
        apex: Ident(apex.to_string()),
        legs: legs.iter().map(|name| Ident(name.to_string())).collect(),
        cospan: cospan.iter().map(|name| Ident(name.to_string())).collect(),
        status,
    }
}

const ORDER_ONE: &str = r#"{"apex": "Q", "legs": ["r2", "r1"], "cospan": ["g", "f"], "status": "proposed"},
      {"apex": "P", "legs": ["p2", "p1"], "cospan": ["g", "f"], "status": "accepted"},
      {"apex": "P", "legs": ["p2", "p1"], "cospan": ["g", "f"], "status": "accepted"}"#;

const ORDER_TWO: &str = r#"{"apex": "P", "legs": ["p2", "p1"], "cospan": ["g", "f"], "status": "accepted"},
      {"apex": "Q", "legs": ["r2", "r1"], "cospan": ["g", "f"], "status": "proposed"},
      {"apex": "P", "legs": ["p2", "p1"], "cospan": ["g", "f"], "status": "accepted"}"#;

/// Scenario "Pullbacks round-trip and canonicalize": input order is
/// presentation, the parsed schema carries the declarations in canonical
/// `(apex, legs, cospan, status)` order, a repeated declaration is kept,
/// and every reordering serializes to the same bytes.
#[test]
fn pullbacks_round_trip_and_canonicalize() {
    let first = parsed(&fill(&section(ORDER_ONE)));
    let second = parsed(&fill(&section(ORDER_TWO)));
    let expected = vec![
        pullback("P", &["p2", "p1"], &["g", "f"], ConstraintStatus::Accepted),
        pullback("P", &["p2", "p1"], &["g", "f"], ConstraintStatus::Accepted),
        pullback("Q", &["r2", "r1"], &["g", "f"], ConstraintStatus::Proposed),
    ];
    assert_eq!(
        first.schema.pullbacks, expected,
        "canonical (apex, legs, cospan, status) order; a repeated declaration is kept"
    );
    assert_eq!(first.schema.pullbacks, second.schema.pullbacks);
    assert_eq!(
        canonical_bytes(&first),
        canonical_bytes(&second),
        "input order never reaches the canonical bytes"
    );
    assert_eq!(
        serialize(&first),
        serialize(&second),
        "serialize emits the same bytes for every presentation order"
    );
    assert_eq!(
        parse(&serialize(&first)).expect("canonical bytes parse"),
        first,
        "serialize/parse is a fixed point"
    );

    let apex = &first.schema.pullbacks[0];
    assert_eq!(apex.apex, Ident("P".to_string()), "the apex survives as-is");
    assert_eq!(
        apex.legs,
        vec![Ident("p2".to_string()), Ident("p1".to_string())],
        "leg order is presentation, never rewritten"
    );
    assert_eq!(
        apex.cospan,
        vec![Ident("g".to_string()), Ident("f".to_string())],
        "cospan order is presentation, never rewritten"
    );
    assert_eq!(apex.status, ConstraintStatus::Accepted, "status survives");
    assert!(
        canonical_bytes(&first).contains("\"pullbacks\":["),
        "a present section is emitted"
    );
}

/// Scenario "Absent pullbacks are hash-neutral": an absent section and an
/// empty one produce the same canonical bytes and hash, and neither emits
/// a `pullbacks` key.
#[test]
fn absent_pullbacks_are_hash_neutral() {
    let absent = parsed(&fill(""));
    let empty = parsed(&fill(",\n    \"pullbacks\": []"));
    assert!(absent.schema.pullbacks.is_empty());
    assert!(empty.schema.pullbacks.is_empty());
    assert_eq!(
        canonical_bytes(&empty),
        canonical_bytes(&absent),
        "an empty section is omitted exactly like an absent one"
    );
    assert_eq!(
        model_hash(&empty),
        model_hash(&absent),
        "absence is invisible to the model hash"
    );
    assert!(
        !canonical_bytes(&absent).contains("pullbacks"),
        "an absent section is never emitted"
    );
    assert!(
        !serialize(&empty).contains("pullbacks"),
        "an empty pullback list is omitted from canonical bytes"
    );
}

/// Scenario "Status defaults to accepted; proposed is a question": an
/// omitted status parses as accepted and an explicit proposed status is
/// preserved — the equation/key/coproduct status rule verbatim.
#[test]
fn pullback_status_defaults_to_accepted_and_proposed_fails_as_hypothesized() {
    let defaulted = parsed(&fill(&section(
        r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"]}"#,
    )));
    assert_eq!(
        defaulted.schema.pullbacks[0].status,
        ConstraintStatus::Accepted,
        "an omitted status is accepted"
    );

    let proposed = parsed(&fill(&section(
        r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"], "status": "proposed"}"#,
    )));
    assert_eq!(
        proposed.schema.pullbacks[0].status,
        ConstraintStatus::Proposed,
        "an explicit proposed status is preserved"
    );
}

/// Scenario "Malformed pullback is rejected at its node": each defect is
/// `invalid_pullback` at the line/column of the offending JSON node,
/// never a silent repair. The baseline is otherwise valid; only the
/// declaration under test changes.
#[test]
fn malformed_pullback_is_rejected_at_its_node() {
    let cases = [
        (
            r#"{"apex": "Nope", "legs": ["p1", "p2"], "cospan": ["f", "g"]}"#,
            16,
            "undeclared object",
        ),
        (
            r#"{"apex": "P", "legs": ["zz", "p2"], "cospan": ["f", "g"]}"#,
            30,
            "undeclared arrow",
        ),
        (
            r#"{"apex": "P", "legs": ["f", "p2"], "cospan": ["f", "g"]}"#,
            30,
            "apex",
        ),
        (
            r#"{"apex": "P", "legs": ["p1"], "cospan": ["f", "g"]}"#,
            29,
            "exactly two",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "p2", "q"], "cospan": ["f", "g"]}"#,
            29,
            "exactly two",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "zz"]}"#,
            59,
            "undeclared arrow",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f"]}"#,
            53,
            "exactly two",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g", "h"]}"#,
            53,
            "exactly two",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "k"]}"#,
            59,
            "target",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "f"]}"#,
            59,
            "duplicate",
        ),
        (
            r#"{"apex": "P", "legs": ["p2", "p1"], "cospan": ["f", "g"]}"#,
            30,
            "targets `B`, not the source `A`",
        ),
        (
            r#"{"apex": "P", "legs": ["p1", "q"], "cospan": ["f", "g"]}"#,
            36,
            "targets `C`, not the source `B`",
        ),
    ];
    assert_eq!(cases.len(), 12, "every malformed-pullback class is pinned");
    for (raw, column, fragment) in cases {
        assert_invalid(raw, column, fragment);
    }
}

/// One malformed declaration: the stable code, the offending node's line
/// and column, and a message fragment naming the defect.
fn assert_invalid(raw: &str, column: usize, fragment: &str) {
    let text = fill(&section(raw));
    let expected_line = line_of(&text, raw);
    let ParseError::Semantic(error) = parse(&text).expect_err(raw) else {
        panic!("`{raw}` must be a semantic rejection");
    };
    assert_eq!(
        error.code,
        ErrorCode::InvalidPullback,
        "`{raw}`: {}",
        error.message
    );
    assert_eq!(
        (error.line, error.column),
        (expected_line, column),
        "`{raw}`: the error points at the offending node"
    );
    assert!(
        error.message.contains(fragment),
        "`{raw}`: the message names the defect (`{fragment}`): {}",
        error.message
    );
}

/// The 1-based line holding `needle` (unique in the document).
fn line_of(document: &str, needle: &str) -> usize {
    let lines: Vec<usize> = document
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(index, _)| index + 1)
        .collect();
    assert_eq!(lines.len(), 1, "the document spells `{needle}` once");
    lines[0]
}

/// Scenario "The cones field stays rejected": the capability is
/// declarable only through `schema.pullbacks`; a `cones` namespace is an
/// `unknown_field` at the field's line.
#[test]
fn cones_field_stays_rejected() {
    let text = fill(",\n    \"cones\": [{\"apex\": \"P\"}]");
    let expected_line = line_of(&text, "\"cones\"");
    let ParseError::Semantic(error) = parse(&text).expect_err("`cones` is not a field") else {
        panic!("a `cones` field is a semantic rejection");
    };
    assert_eq!(error.code, ErrorCode::UnknownField);
    assert_eq!(
        error.line, expected_line,
        "the error points at the `cones` node"
    );
    assert!(
        error.message.contains("cones"),
        "the message names `cones`: {}",
        error.message
    );
}
