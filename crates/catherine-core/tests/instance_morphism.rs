//! Purpose: the instance-morphism format contract (format 1.4, openspec
//! change `add-instance-morphism`): an additive `instance_map` section
//! declares a claimed, total row mapping from this document's instance to
//! another instance of the same schema; the section round-trips,
//! canonicalizes, is total at parse, and is hash-neutral when absent.
//!
//! Responsibilities: pin the canonical `(object, rows)` order and the
//! name-blind canonical form (a renamed source instance expressing the
//! same claim keeps its identity), every structural rejection of a
//! malformed claim at its stable code and offending node, the
//! absent/empty hash guard (a pre-existing corpus fixture keeps its
//! canonical bytes and `model_hash`), and the construction-time split:
//! a target name that no instance declares is *data*, preserved verbatim
//! by parse — the labeled `unknown_map_target` belongs to the operation
//! (`crates/catherine-cli/tests/naturality.rs` pins it), never to the
//! format.
//!
//! Rationale: openspec change `add-instance-morphism`, design Decisions
//! 1 and 2 — the map is sealed model evidence (presence moves the hash,
//! absence must not), totality is required because a partial map has no
//! decidable reading without an implicit identity, and target resolution
//! happens once at operation construction. Assertions are structural:
//! byte-level pins are limited to the hash-neutrality guard (the keys
//! precedent) and to the canonical object order the spec names.

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use serde_json::Value;

#[path = "instance_morphism/rejection.rs"]
mod rejection;
use rejection::rejection;

const SMALL_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/parity/small/model.ct.json"
);

/// The pre-change identity of `fixtures/parity/small/model.ct.json`,
/// computed before the instance-map section existed (the same value the
/// schema-map and rule-presentation suites pin). If this moves, the
/// additive 1.4 section is not hash-neutral and every sealed ledger
/// `model_hashes` entry is invalidated.
const SMALL_PRE_CHANGE_HASH: &str =
    "sha256:fb033e3ccba19e8b558522062fea7d086a1bfc95e48ec04e50993cf6072e1311";

/// The canonical bytes of the same fixture: dense per-object row names,
/// sorted sections, one line. Pinned beside the hash so a canonical-shape
/// regression is visible as bytes, not just as a digest.
const SMALL_PRE_CHANGE_CANONICAL: &str = concat!(
    r#"{"format_version":"1.0","instance":{"rows":{"Constraint":["_0"],"Intent":["_0"]},"#,
    r#""tables":{"traces_to":{"_0":"_0"}}},"schema":{"arrows":[{"name":"traces_to","src":"Constraint","tgt":"Intent"}],"#,
    r#""equations":[],"objects":["Constraint","Intent"]}}"#
);

/// A total 1.4 map: `account` and `owner` rows onto the target instance's
/// `w*`/`p*` names, one arrow so the map's semantics stay legible.
const MAPPED_DOC: &str = r#"{
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

/// The same model declared in the opposite presentation order: the `maps`
/// array is reversed and each `rows` object's keys are reversed.
const MAPPED_REORDERED: &str = r#"{
  "format_version": "1.4",
  "instance_map": {
    "maps": [
      {"object": "owner", "rows": {"o2": "p2", "o1": "p1"}},
      {"object": "account", "rows": {"a2": "w2", "a1": "w1"}}
    ]
  },
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["a1", "a2"], "owner": ["o1", "o2"]},
    "tables": {"account_owner": {"a1": "o1", "a2": "o2"}}
  }
}"#;

/// The same claim under an order-reversing renaming of the *source* rows:
/// presentation names are changed in the instance, the tables, and the
/// map's source keys alike, so the document expresses an identical model.
const MAPPED_RENAMED: &str = r#"{
  "format_version": "1.4",
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["omega", "alpha"], "owner": ["o1", "o2"]},
    "tables": {"account_owner": {"omega": "o2", "alpha": "o1"}}
  },
  "instance_map": {
    "maps": [
      {"object": "account", "rows": {"omega": "w2", "alpha": "w1"}},
      {"object": "owner", "rows": {"o1": "p1", "o2": "p2"}}
    ]
  }
}"#;

/// The same instance with no `instance_map` at all — the pre-1.4 shape.
const UNMAPPED_DOC: &str = r#"{
  "format_version": "1.4",
  "schema": {
    "objects": ["account", "owner"],
    "arrows": [{"name": "account_owner", "src": "account", "tgt": "owner"}],
    "equations": []
  },
  "instance": {
    "rows": {"account": ["a1", "a2"], "owner": ["o1", "o2"]},
    "tables": {"account_owner": {"a1": "o1", "a2": "o2"}}
  }
}"#;

/// A schema with no instance rows at all: an empty `maps` list is total
/// (vacuously) and must be as invisible as an absent section.
const NO_ROWS_MAP: &str = r#"{
  "format_version": "1.4",
  "schema": {"objects": ["account"], "arrows": [], "equations": []},
  "instance": {"rows": {}, "tables": {}},
  "instance_map": {"maps": []}
}"#;

/// The same empty instance with no section.
const NO_ROWS_UNMAPPED: &str = r#"{
  "format_version": "1.4",
  "schema": {"objects": ["account"], "arrows": [], "equations": []},
  "instance": {"rows": {}, "tables": {}}
}"#;

/// An incomplete map: `account` maps `a1` but omits `a2`. The per-object
/// map entry for `account` sits on line 14; the `rows` object that fails
/// coverage starts at column 37.
const INCOMPLETE_MAP_DOC: &str = r#"{
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
      {"object": "account", "rows": {"a1": "w1"}},
      {"object": "owner", "rows": {"o1": "p1", "o2": "p2"}}
    ]
  }
}"#;

/// The smallest total map — one object entry per object, one row each —
/// so every rejection below is exactly one edit away from a claim that
/// parses. All offending nodes sit on the one-line `instance_map` row.
const MINIMAL_MAP: &str = r#"{
  "format_version": "1.4",
  "schema": {"objects": ["account", "owner"], "arrows": [], "equations": []},
  "instance": {"rows": {"account": ["a1"], "owner": ["o1"]}, "tables": {}},
  "instance_map": {"maps": [{"object": "account", "rows": {"a1": "t1"}}, {"object": "owner", "rows": {"o1": "t2"}}]}
}"#;

fn parsed(text: &str, what: &str) -> catherine_core::model::Model {
    parse(text).unwrap_or_else(|error| panic!("{what} parses: {error:?}"))
}

/// Scenario "Map round-trips and canonicalizes": input order is
/// presentation — the parsed map carries the canonical `(object, rows)`
/// order, canonical bytes are independent of the input order and of the
/// source rows' presentation names, and serialize/parse is a fixed point.
#[test]
fn instance_map_round_trips_and_canonicalizes() {
    let first = parsed(MAPPED_DOC, "a total 1.4 instance map");
    let second = parsed(MAPPED_REORDERED, "the reordered declaration");
    let renamed = parsed(MAPPED_RENAMED, "the renamed source instance");

    assert_eq!(
        first.format_version.minor, 4,
        "1.4 is the additive minor the instance-map section lands in"
    );

    assert_eq!(first, second, "input order never reaches the model");
    assert_eq!(
        canonical_bytes(&first),
        canonical_bytes(&second),
        "input order never reaches the canonical bytes"
    );
    assert_eq!(
        canonical_bytes(&first),
        canonical_bytes(&renamed),
        "the source rows' presentation names are not part of the claim's identity"
    );
    assert_eq!(
        model_hash(&first),
        model_hash(&renamed),
        "a renamed source instance expressing the same claim keeps its hash"
    );

    // Canonical `(object, rows)` order: objects ascending, each entry's
    // rows keyed by the document's rows. Target names are external, so
    // they are carried as declared.
    let canonical: Value = serde_json::from_str(&serialize(&first))
        .unwrap_or_else(|error| panic!("canonical serialization is not JSON: {error}"));
    let maps = canonical["instance_map"]["maps"]
        .as_array()
        .unwrap_or_else(|| panic!("the instance_map section does not survive serialization"));
    assert_eq!(maps.len(), 2, "one entry per mapped object");
    assert_eq!(maps[0]["object"], "account", "objects in ascending order");
    assert_eq!(maps[1]["object"], "owner", "objects in ascending order");
    let account_rows = maps[0]["rows"]
        .as_object()
        .unwrap_or_else(|| panic!("the account rows are not an object"));
    assert_eq!(account_rows.len(), 2, "every account row is mapped");
    let mut targets: Vec<&str> = account_rows
        .values()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("a target row name is not a string"))
        })
        .collect();
    targets.sort_unstable();
    assert_eq!(targets, vec!["w1", "w2"], "target names as declared");

    let reparsed = parsed(&serialize(&first), "canonical bytes parse");
    assert_eq!(reparsed, first, "serialize/parse is a fixed point");

    let unmapped = parsed(UNMAPPED_DOC, "the map-free document");
    assert_ne!(
        model_hash(&first),
        model_hash(&unmapped),
        "a declared map is sealed evidence: its presence moves the model hash"
    );
}

/// Scenario "Absent map is hash-neutral": a pre-existing document under
/// the new minor keeps its exact canonical bytes and hash, and an empty
/// map over an empty instance is omitted just like an absent one.
#[test]
fn absent_instance_map_hashes_as_before() {
    let model = parsed(
        &std::fs::read_to_string(SMALL_FIXTURE).unwrap_or_else(|error| {
            panic!("corpus fixture {SMALL_FIXTURE} is unreadable: {error}")
        }),
        "the pre-1.4 corpus fixture parses",
    );
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
        !canonical_bytes(&model).contains("instance_map"),
        "an absent section is never emitted empty"
    );

    let empty = parsed(NO_ROWS_MAP, "an empty map over an empty instance");
    let bare = parsed(NO_ROWS_UNMAPPED, "the same instance without the section");
    assert_eq!(
        canonical_bytes(&empty),
        canonical_bytes(&bare),
        "an empty map list is omitted from canonical bytes"
    );
    assert_eq!(
        model_hash(&empty),
        model_hash(&bare),
        "an empty map list is hash-neutral"
    );
    assert!(
        !canonical_bytes(&empty).contains("instance_map"),
        "an empty section is never emitted"
    );
}

/// Scenario "Incomplete map is rejected at its node": every row of every
/// schema object must be covered. The uncovered `account` row is named,
/// and the error points at the per-object map node whose coverage is
/// short — never an implicit identity on the rest.
#[test]
fn incomplete_map_is_rejected_at_its_node() {
    let Err(ParseError::Semantic(semantic)) = parse(INCOMPLETE_MAP_DOC) else {
        panic!("an incomplete map is a labeled semantic rejection, never syntax or acceptance");
    };
    assert_eq!(
        semantic.code,
        ErrorCode::IncompleteInstanceMap,
        "the stable code for an uncovered row: {} ({}:{})",
        semantic.message,
        semantic.line,
        semantic.column
    );
    assert_eq!(
        (semantic.line, semantic.column),
        (14, 37),
        "the incomplete `account` map entry's `rows` object is the offending node"
    );
    assert!(
        semantic.message.contains("account") && semantic.message.contains("a2"),
        "the rejection names the object and the uncovered row: {}",
        semantic.message
    );
}

/// The five remaining structural rejections, each one edit away from
/// [`MINIMAL_MAP`]: a repeated entry object, an entry object the schema
/// never declares, a source key that is not a declared row of its object,
/// a target that is not a well-formed ident, and a schema object with no
/// entry at all. Expected positions are that document's layout: the
/// second entry's `object` value on the one-line map row, the cell's key
/// or value, and the `maps` array for a missing entry (no per-object node
/// exists to point at). `syntax` is the kernel's structural code for an
/// undeclared row key; the rest are semantic codes.
#[test]
fn malformed_maps_are_rejected_at_their_nodes() {
    let owner = r#"{"object": "owner", "rows": {"o1": "t2"}}"#;
    let cell = r#""o1": "t2""#;
    #[rustfmt::skip]
    let claims = [
        ("duplicate entry object", MINIMAL_MAP.replace(owner, r#"{"object": "account", "rows": {"a1": "t1"}}"#),
         "duplicate_ident", (5, 85), "duplicate instance map entry for object `account`"),
        ("undeclared entry object", MINIMAL_MAP.replace(r#"{"object": "owner", "#, r#"{"object": "ghost", "#),
         "invalid_ident", (5, 85), "names undeclared object `ghost`"),
        ("source key that is not a declared row", MINIMAL_MAP.replace(cell, r#""o9": "t2""#),
         "syntax", (5, 103), "keys row `o9`, which is not a row of `owner`"),
        ("target that is not an ident", MINIMAL_MAP.replace(cell, r#""o1": "no such row""#),
         "invalid_ident", (5, 109), "target `no such row` is not a well-formed row ident"),
        ("schema object with no map entry", MINIMAL_MAP.replace(&format!(", {owner}"), ""),
         "incomplete_instance_map", (5, 28), "no entry for object `owner`; its row `o1` is uncovered"),
    ];
    for (what, text, expected_code, expected_at, needle) in claims {
        let (code, at, message) = rejection(&text, what);
        assert_eq!(code, expected_code, "[{what}] stable code: {message}");
        assert_eq!(at, expected_at, "[{what}] offending node: {message}");
        assert!(
            message.contains(needle),
            "[{what}] the rejection names the defect: {message}"
        );
    }
}

/// Design Decision 2: target resolution happens once at operation
/// construction because map cells name rows of *another* document. The
/// kernel therefore preserves a target name it cannot resolve — the cell
/// is data, not a parse error and not a dropped default. The operation's
/// labeled `unknown_map_target` rejection is pinned in
/// `crates/catherine-cli/tests/naturality.rs` and the negative matrix.
#[test]
fn unknown_target_is_preserved_for_construction_time_resolution() {
    let unresolvable = MAPPED_DOC.replace(r#""a2": "w2""#, r#""a2": "w9""#);
    let model = parsed(
        &unresolvable,
        "a map naming a target row of another instance",
    );
    let bytes = canonical_bytes(&model);
    assert!(
        bytes.contains(r#""w9""#),
        "an unresolvable target name is preserved verbatim, never dropped: {bytes}"
    );
    let reparsed = parsed(&serialize(&model), "the serialized document parses");
    assert_eq!(reparsed, model, "serialize/parse is a fixed point");
    assert_ne!(
        model_hash(&model),
        model_hash(&parsed(MAPPED_DOC, "the resolvable map")),
        "the cell's target name is hashed model data"
    );
}
