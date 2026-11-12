//! Purpose: the Δ migration phase-A parse contract — schema maps are
//! optional additive sections (`format_version` 1.1) parsed with totality
//! (a partial map is a labeled error naming the unmapped element), and
//! their presence never moves an existing document's identity.
//!
//! Responsibilities: pin the hash-stability guard (a pre-existing corpus
//! fixture's `model_hash`, computed before the schema-map section landed,
//! asserted unchanged — the section is skip-serialized when absent), the
//! total-map parse shape, the partial-map rejection with its label and
//! node position, assignment-target declaredness, and the section's
//! serialize/parse fixed point.
//!
//! Rationale: openspec change `add-delta-migration` phase A — maps are
//! declared, never executed (pullback/reindex is phase B), and dropped
//! elements are explicit `null` assignments, never silent defaults. The
//! pinned hash is the P6 ledger's identity contract: content addressing
//! must not shift for documents that predate the axis.

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::hash::model_hash;
use catherine_core::model::Ident;
use catherine_core::parse::parse;

const SMALL_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/parity/small/model.ct.json"
);

/// The pre-change identity of `fixtures/parity/small/model.ct.json`,
/// computed before the schema-map section existed. If this value moves,
/// an existing document's canonical bytes moved — the ledger's
/// content-addressing is broken and the change must not ship.
const SMALL_PRE_CHANGE_HASH: &str =
    "sha256:fb033e3ccba19e8b558522062fea7d086a1bfc95e48ec04e50993cf6072e1311";

/// A total map over a two-object source: `Blob` survives (its arrow
/// remaps onto a target self-loop), `Temp` is dropped explicitly.
const TOTAL_MAP_DOC: &str = r#"{
  "format_version": "1.1",
  "schema": {
    "objects": ["Blob", "Temp"],
    "arrows": [{"name": "keep", "src": "Blob", "tgt": "Blob"},
               {"name": "temp_only", "src": "Temp", "tgt": "Temp"}],
    "equations": []
  },
  "instance": {
    "rows": {"Blob": ["_0", "_1"], "Temp": ["_0"]},
    "tables": {"keep": {"_0": "_0", "_1": null}, "temp_only": {"_0": "_0"}}
  },
  "schema_map": {
    "target_schema": {
      "objects": ["Blob"],
      "arrows": [{"name": "keep", "src": "Blob", "tgt": "Blob"}],
      "equations": []
    },
    "objects": {"Blob": "Blob", "Temp": null},
    "arrows": {"keep": ["keep"], "temp_only": null}
  }
}"#;

#[test]
fn existing_documents_hash_as_before_the_schema_map_section() {
    let text = std::fs::read_to_string(SMALL_FIXTURE).expect("corpus fixture exists");
    let model = parse(&text).expect("corpus fixture parses");
    assert!(
        model.schema_map.is_none(),
        "a pre-map document carries no schema map"
    );
    assert_eq!(
        model_hash(&model),
        SMALL_PRE_CHANGE_HASH,
        "the schema-map section must be invisible to existing documents' identity"
    );
    assert!(
        !canonical_bytes(&model).contains("schema_map"),
        "an absent section is skip-serialized, never emitted empty"
    );
}

#[test]
fn a_total_schema_map_parses_with_target_and_explicit_drops() {
    let model = parse(TOTAL_MAP_DOC).expect("a total map parses");
    let Some(map) = &model.schema_map else {
        panic!("the 1.1 section parses into the model");
    };
    assert_eq!(
        map.target
            .objects
            .keys()
            .map(|k| k.0.as_str())
            .collect::<Vec<_>>(),
        vec!["Blob"],
        "the target schema carries only its declared objects"
    );
    assert_eq!(
        map.objects
            .get(&Ident("Blob".to_string()))
            .and_then(|a| a.as_ref()),
        Some(&Ident("Blob".to_string())),
        "Blob survives into the target"
    );
    assert_eq!(
        map.objects.get(&Ident("Temp".to_string())),
        Some(&None),
        "Temp is dropped explicitly, never silently"
    );
    assert_eq!(
        map.arrows.get(&Ident("temp_only".to_string())),
        Some(&None),
        "the dropped object's arrow is dropped explicitly alongside it"
    );
}

#[test]
fn a_partial_schema_map_is_rejected_naming_the_unmapped_element() {
    // Drop the Blob assignment: the map no longer covers every declared
    // source object — the labeled rejection names the unmapped element.
    let partial = TOTAL_MAP_DOC.replace(r#""Blob": "Blob","#, "");
    let error = parse(&partial).expect_err("a partial map is a build error");
    let ParseError::Semantic(semantic) = error else {
        panic!("the partial-map rejection is a labeled semantic error");
    };
    assert_eq!(semantic.code, ErrorCode::PartialSchemaMap);
    assert!(
        semantic.message.contains("Blob"),
        "the rejection names the unmapped element: {}",
        semantic.message
    );
}

#[test]
fn a_mapped_document_is_a_serialize_parse_fixed_point() {
    let model = parse(TOTAL_MAP_DOC).expect("total map parses");
    let bytes = serialize(&model);
    assert!(
        bytes.contains("schema_map"),
        "the map section is emitted when present"
    );
    let reparsed = parse(&bytes).expect("the serialized document parses");
    assert_eq!(reparsed, model, "serialize/parse is a fixed point");
    assert_eq!(model_hash(&model), model_hash(&reparsed));
}
