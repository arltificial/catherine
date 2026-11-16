//! Purpose: the analysis-level classification of naturality squares
//! (catherine-ke1): a square whose two compositions are both undefined is
//! neither clean nor a witnessed break — it is reported in
//! [`NaturalityOutcome::undefined`] so the caller can warn instead of
//! claiming a verified clean. A square undefined on one side remains a
//! `violation` with a null witness side.
//!
//! Responsibilities: parse real 1.4 documents around one dangling
//! `account_owner` cell and assert the three classifications structurally:
//! both sides null → exactly one [`UndefinedSquare`] and no finding; one
//! side null → one `violation`/`NaturalityWitness` with the null side; both
//! sides defined and equal → totally clean.
//!
//! Rationale: silences are the defect — the same-schema map is executed
//! over every arrow row, and a square with no composition on either side
//! used to vanish from the sweep. The classifier keeps the existing
//! `violation` kind and witness shape; only the third case is new, and it
//! is returned beside the findings rather than folded into them.

use catherine_analysis::naturality::{naturality, NaturalityOutcome, UndefinedSquare};
use catherine_core::findings::{FindingKind, ViolationWitness};
use catherine_core::model::{Ident, RowId};

/// A source instance whose `account_owner` cell at `a2` dangles; the
/// declared map is total.
const SOURCE_DANGLING: &str = r#"{
  "format_version": "1.4",
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["a1", "a2"], "owner": ["o1", "o2"]},
    "tables": {"account_owner": {"a1": "o1"}}
  },
  "instance_map": {
    "maps": [
      {"object": "account", "rows": {"a1": "w1", "a2": "w2"}},
      {"object": "owner", "rows": {"o1": "p1", "o2": "p2"}}
    ]
  }
}"#;

/// The same source with its `a2` cell defined.
const SOURCE_DEFINED: &str = r#"{
  "format_version": "1.4",
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["a1", "a2"], "owner": ["o1", "o2"]},
    "tables": {"account_owner": {"a1": "o1", "a2": "o2"}}
  },
  "instance_map": {
    "maps": [
      {"object": "account", "rows": {"a1": "w1", "a2": "w2"}},
      {"object": "owner", "rows": {"o1": "p1", "o2": "p2"}}
    ]
  }
}"#;

/// The target whose matching `w2` cell also dangles: the `a2→w2` square
/// has no composition on either side.
const TARGET_DANGLING: &str = r#"{
  "format_version": "1.4",
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["w1", "w2"], "owner": ["p1", "p2"]},
    "tables": {"account_owner": {"w1": "p1"}}
  }
}"#;

/// The same target with its `w2` cell defined.
const TARGET_DEFINED: &str = r#"{
  "format_version": "1.4",
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["w1", "w2"], "owner": ["p1", "p2"]},
    "tables": {"account_owner": {"w1": "p1", "w2": "p2"}}
  }
}"#;

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// Execute the declared map over the given source/target documents.
fn outcome(source_text: &str, target_text: &str) -> NaturalityOutcome {
    let (source, source_names) =
        catherine_core::parse::parse_with_rows(source_text).expect("the source parses");
    let (target, target_names) =
        catherine_core::parse::parse_with_rows(target_text).expect("the target parses");
    naturality(&source, &target, &source_names, &target_names).expect("the map resolves")
}

/// Both compositions undefined: exactly one undefined square, no finding
/// — the square is a non-answer, never a clean.
#[test]
fn undefined_square_is_reported_not_clean() {
    let outcome = outcome(SOURCE_DANGLING, TARGET_DANGLING);
    assert!(
        outcome.findings.is_empty(),
        "an undefined square is not a witnessed break: {:?}",
        outcome.findings
    );
    assert_eq!(
        outcome.undefined,
        vec![UndefinedSquare {
            arrow: ident("account_owner"),
            source_row: RowId(1),
        }],
        "the `account_owner` square at row `a2` is undefined on both sides"
    );
}

/// One side undefined: a witnessed break with a null `lhs` and the
/// resolved target row as `rhs`.
#[test]
fn one_sided_undefined_is_a_witnessed_violation() {
    let outcome = outcome(SOURCE_DANGLING, TARGET_DEFINED);
    assert!(
        outcome.undefined.is_empty(),
        "a one-sided undefined square is decidable: {:?}",
        outcome.undefined
    );
    assert_eq!(outcome.findings.len(), 1, "exactly one square breaks");
    match &outcome.findings[0].kind {
        FindingKind::Violation(ViolationWitness::Naturality(witness)) => {
            assert_eq!(witness.arrow, ident("account_owner"));
            assert_eq!(witness.source_row, RowId(1));
            assert_eq!(witness.lhs, None, "the source cell dangles");
            assert_eq!(witness.rhs, Some(RowId(1)), "the target maps to `p2`");
        }
        other => panic!("a break is a naturality violation, not {other:?}"),
    }
}

/// Both compositions defined and equal: the fully executed sweep is clean
/// in both channels.
#[test]
fn defined_commuting_square_is_clean() {
    let outcome = outcome(SOURCE_DEFINED, TARGET_DEFINED);
    assert!(
        outcome.findings.is_empty(),
        "the defined map commutes: {:?}",
        outcome.findings
    );
    assert!(
        outcome.undefined.is_empty(),
        "every square was verified: {:?}",
        outcome.undefined
    );
}
