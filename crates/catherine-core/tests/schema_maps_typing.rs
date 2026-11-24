//! Purpose: the schema map's composable-path endpoint typing (wave 2,
//! catherine-kpn.7) — mapped arrow paths are checked against the mapped
//! objects' images in the target schema at parse time.
//!
//! Responsibilities: pin the labeled refusal (`partial_schema_map`
//! naming the arrow, its objects, and the offending path element) for
//! the source-endpoint mismatch, the non-composable path, and the
//! target-endpoint mismatch, and pin that a well-typed map still parses
//! with its mapping carried as declared.
//!
//! Rationale: split out of `schema_maps.rs` at the file-line gate — the
//! phase-A target pins the section's totality and identity contract,
//! this target pins the typing obligation the parser now refuses
//! instead of deferring to execution.

use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::model::{Ident, Path};
use catherine_core::parse::parse;

/// A 1.1 document whose schema map sends `f: A -> B` to `h: Y -> Y`
/// under `A -> X, B -> Y`: the mapped path's source endpoint does not
/// match the source object's image — the non-functorial shape
/// catherine-kpn.7 refuses at parse time.
const NONFUNCTORIAL_MAP_DOC: &str = r#"{
  "format_version": "1.1",
  "schema": {
    "objects": ["A", "B"],
    "arrows": [{"name": "f", "src": "A", "tgt": "B"}],
    "equations": []
  },
  "instance": {
    "rows": {"A": ["_0"], "B": ["_0"]},
    "tables": {"f": {"_0": "_0"}}
  },
  "schema_map": {
    "target_schema": {
      "objects": ["X", "Y"],
      "arrows": [{"name": "g", "src": "X", "tgt": "Y"},
                 {"name": "h", "src": "Y", "tgt": "Y"}],
      "equations": []
    },
    "objects": {"A": "X", "B": "Y"},
    "arrows": {"f": ["h"]}
  }
}"#;

/// The same document shape with the arrow corrected to `g: X -> Y`: the
/// well-typed map parses unchanged — endpoint typing refuses only the
/// ill-typed mapping.
const WELL_TYPED_MAP_DOC: &str = r#"{
  "format_version": "1.1",
  "schema": {
    "objects": ["A", "B"],
    "arrows": [{"name": "f", "src": "A", "tgt": "B"}],
    "equations": []
  },
  "instance": {
    "rows": {"A": ["_0"], "B": ["_0"]},
    "tables": {"f": {"_0": "_0"}}
  },
  "schema_map": {
    "target_schema": {
      "objects": ["X", "Y"],
      "arrows": [{"name": "g", "src": "X", "tgt": "Y"},
                 {"name": "h", "src": "Y", "tgt": "Y"}],
      "equations": []
    },
    "objects": {"A": "X", "B": "Y"},
    "arrows": {"f": ["g"]}
  }
}"#;

/// The endpoint mismatch is a labeled parse refusal at the arrow's node:
/// the code is `partial_schema_map`, and the message names the offending
/// arrow, its source object, and both images (catherine-kpn.7).
#[test]
fn a_nonfunctorial_schema_map_is_a_labeled_refusal_naming_the_arrow_and_objects() {
    let ParseError::Semantic(error) = parse(NONFUNCTORIAL_MAP_DOC).expect_err("non-functorial map refused") else {
        panic!("the endpoint mismatch is a labeled semantic error");
    };
    assert_eq!(error.code, ErrorCode::PartialSchemaMap);
    for name in ["f", "A", "B", "X", "Y", "h"] {
        assert!(
            error.message.contains(name),
            "the refusal names `{name}`: {}",
            error.message
        );
    }
    assert!(
        error.message.contains("endpoint") || error.message.contains("functorial"),
        "the refusal names the defect class: {}",
        error.message
    );
}

/// The composable-path mismatch: a two-element path whose elements do
/// not compose (`h`'s target `Y` is not `g`'s source `X`) is refused
/// with the same labeled code, naming the non-composable boundary.
#[test]
fn a_non_composable_mapped_path_is_a_labeled_refusal() {
    // `A -> Y` types the source endpoint (`h` starts at `Y`); the path
    // then fails to compose: `h`'s target `Y` is not `g`'s source `X`.
    let document = NONFUNCTORIAL_MAP_DOC
        .replace(r#""A": "X""#, r#""A": "Y""#)
        .replace(r#""f": ["h"]"#, r#""f": ["h", "g"]"#);
    let ParseError::Semantic(error) = parse(&document).expect_err("non-composable path refused") else {
        panic!("the composition mismatch is a labeled semantic error");
    };
    assert_eq!(error.code, ErrorCode::PartialSchemaMap);
    assert!(
        error.message.contains("does not compose") && error.message.contains("f"),
        "the refusal names the non-composable path: {}",
        error.message
    );
}

/// The target-endpoint mismatch: the path composes but lands short of
/// the target object's image is refused with the same labeled code.
#[test]
fn a_target_endpoint_mismatch_is_a_labeled_refusal() {
    let document = NONFUNCTORIAL_MAP_DOC
        .replace(r#""objects": {"A": "X", "B": "Y"}"#, r#""objects": {"A": "X", "B": "X"}"#)
        .replace(r#""f": ["h"]"#, r#""f": ["g"]"#);
    let ParseError::Semantic(error) = parse(&document).expect_err("target mismatch refused") else {
        panic!("the target-endpoint mismatch is a labeled semantic error");
    };
    assert_eq!(error.code, ErrorCode::PartialSchemaMap);
    assert!(
        error.message.contains("target endpoint") && error.message.contains("B"),
        "the refusal names the target-endpoint mismatch: {}",
        error.message
    );
}

/// The well-typed counterpart parses unchanged — endpoint typing
/// refuses only the ill-typed mapping, never a functorial map.
#[test]
fn a_well_typed_schema_map_still_parses() {
    let model = parse(WELL_TYPED_MAP_DOC).expect("a well-typed map parses");
    let map = model.schema_map.as_ref().expect("the section parses");
    assert_eq!(
        map.arrows.get(&Ident("f".to_string())),
        Some(&Some(path(&["g"]))),
        "the well-typed mapping is carried as declared"
    );
}

/// A path from declared arrow names.
fn path(elements: &[&str]) -> Path {
    Path(elements.iter().map(|e| Ident(e.to_string())).collect())
}
