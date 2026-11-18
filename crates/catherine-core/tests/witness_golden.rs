//! Purpose: the remaining violation-witness golden encodings (openspec
//! change `add-pullback-checks`, finding
//! `every-finding-kind-has-a-golden-fixture`): one real kernel run per
//! witness shape still without a golden fixture — the accepted-equation
//! failure, the accepted-key collision, and the accepted-coproduct
//! coverage/disjointness pair.
//!
//! Responsibilities: pin each fixture's exact `Finding` vector (kind,
//! witness ids, `executed` label), the witness's exact serialized field
//! set (`equation`/`row`/`lhs`/`rhs`, `key`/`rows`,
//! `coproduct`/`row` with `injections` omitted when empty), and a
//! lossless serde round-trip — structurally, never from output text.
//!
//! Rationale: the violation witness is an untagged union whose
//! disambiguation rests on the disjoint field sets, so the encoding is a
//! contract, not a detail — `equation`'s four fields stay exactly as
//! before, and the coproduct witness's omitted `injections` distinguishes
//! coverage from disjointness without the schema. Row ids are read back
//! from the parsed model's dense tables, never guessed, and the inline
//! documents stay at the minimum minor that declares their section
//! (1.3 for keys, 1.5 for coproducts).

use catherine_core::checks::check;
use catherine_core::findings::{
    CoproductWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, KeyWitness,
    ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Model, RowId};
use catherine_core::parse::parse;

/// Format 1.3: one accepted key over `capture`'s arrow pair; both rows
/// carry the same tuple, so the injectivity claim fails exactly once.
const KEY_DOC: &str = r#"{
  "format_version": "1.3",
  "schema": {
    "objects": ["capture", "value"],
    "arrows": [
      {"name": "capture_source", "src": "capture", "tgt": "value"},
      {"name": "capture_edition", "src": "capture", "tgt": "value"}
    ],
    "equations": [],
    "keys": [
      {"object": "capture", "arrows": ["capture_source", "capture_edition"], "status": "accepted"}
    ]
  },
  "instance": {
    "rows": {"capture": ["c1", "c2"], "value": ["v1"]},
    "tables": {
      "capture_source": {"c1": "v1", "c2": "v1"},
      "capture_edition": {"c1": "v1", "c2": "v1"}
    }
  }
}"#;

/// Format 1.5: one accepted coproduct `C = V1 + V2`; `c1` lies in both
/// injection images (disjointness fails) and `c4` in neither (coverage
/// fails), while `c2` and `c3` are covered exactly once.
const COPRODUCT_DOC: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["C", "V1", "V2"],
    "arrows": [
      {"name": "i1", "src": "V1", "tgt": "C"},
      {"name": "i2", "src": "V2", "tgt": "C"}
    ],
    "equations": [],
    "coproducts": [
      {"object": "C", "arrows": ["i1", "i2"], "status": "accepted"}
    ]
  },
  "instance": {
    "rows": {"C": ["c1", "c2", "c3", "c4"], "V1": ["v1", "v2"], "V2": ["w1", "w2"]},
    "tables": {
      "i1": {"v1": "c1", "v2": "c2"},
      "i2": {"w1": "c1", "w2": "c3"}
    }
  }
}"#;

/// Parse → build → check, the kernel's one pipeline.
fn checked(text: &str) -> (Model, Vec<Finding>) {
    let model = parse(text).expect("the fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    let findings = check(&model.schema, &built);
    (model, findings)
}

/// The dense id of an object's `index`-th parsed row — never a guessed
/// name → id map.
fn row(model: &Model, object: &str, index: usize) -> RowId {
    model.instance.rows[&Ident(object.to_string())][index]
}

/// The resolved value of an arrow on a source row, read from the parsed
/// tables.
fn value(model: &Model, arrow: &str, src: RowId) -> RowId {
    model.instance.values[&Ident(arrow.to_string())][src.0].expect("cell is resolved")
}

fn violation(witness: ViolationWitness) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(witness),
    }
}

/// Assert the finding's label, kind, and exact witness field set, then
/// return the serialized witness — structural, never output-text
/// assertions.
fn assert_encoding(
    finding: &Finding,
    label: &str,
    kind: &str,
    fields: &[&str],
) -> serde_json::Value {
    let json = serde_json::to_value(finding).expect("finding serializes");
    assert_eq!(json["label"].as_str(), Some(label), "the epistemic label");
    assert_eq!(json["kind"].as_str(), Some(kind), "the finding kind");
    let witness = json["witness"].clone();
    let mut actual: Vec<String> = witness
        .as_object()
        .expect("the witness is an object")
        .keys()
        .cloned()
        .collect();
    actual.sort_unstable();
    let mut expected: Vec<String> = fields.iter().map(|field| field.to_string()).collect();
    expected.sort_unstable();
    assert_eq!(
        actual, expected,
        "the witness field set is the shape's contract"
    );
    witness
}

/// Serialize → deserialize → equal.
fn round_trips(findings: &[Finding]) {
    let text = serde_json::to_string(findings).expect("findings serialize");
    let back: Vec<Finding> = serde_json::from_str(&text).expect("findings deserialize");
    assert_eq!(back.as_slice(), findings, "serde round-trip is lossless");
}

/// Scenario "Every finding kind has a golden fixture": the accepted
/// equation fixture fails on its one disagreeing row, carrying both
/// composed values; the encoding is exactly the four-field equation
/// witness, unchanged by the later untagged arms.
#[test]
fn equation_witness_golden_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/violation_accepted.json"
    );
    let text = std::fs::read_to_string(path).expect("the golden fixture exists");
    let (model, findings) = checked(&text);

    let failed = row(&model, "A", 1);
    let lhs = value(&model, "f", failed);
    let rhs = value(&model, "g", failed);
    let expected = violation(ViolationWitness::Equation(EquationWitness {
        equation: 0,
        row: failed,
        lhs,
        rhs,
    }));
    assert_eq!(
        findings,
        vec![expected],
        "the one disagreeing row is the only failure"
    );

    let witness = assert_encoding(
        &findings[0],
        "executed",
        "violation",
        &["equation", "row", "lhs", "rhs"],
    );
    assert_eq!(witness["equation"].as_u64(), Some(0));
    assert_eq!(witness["row"].as_u64(), Some(failed.0 as u64));
    assert_eq!(witness["lhs"].as_u64(), Some(lhs.0 as u64));
    assert_eq!(witness["rhs"].as_u64(), Some(rhs.0 as u64));
    assert!(
        witness.get("pullback").is_none(),
        "no square-shaped field leaks into the equation witness"
    );
    round_trips(&findings);
}

/// Scenario "Every finding kind has a golden fixture": the key fixture's
/// two rows share the tuple, so the accepted key emits exactly one
/// executed collision naming key 0 and both rows ascending.
#[test]
fn key_witness_golden_fixture() {
    let (model, findings) = checked(KEY_DOC);

    let rows = [row(&model, "capture", 0), row(&model, "capture", 1)];
    let expected = violation(ViolationWitness::Key(KeyWitness { key: 0, rows }));
    assert_eq!(
        findings,
        vec![expected],
        "the shared tuple collides exactly once"
    );

    let witness = assert_encoding(&findings[0], "executed", "violation", &["key", "rows"]);
    assert_eq!(witness["key"].as_u64(), Some(0));
    assert_eq!(witness["rows"][0].as_u64(), Some(rows[0].0 as u64));
    assert_eq!(witness["rows"][1].as_u64(), Some(rows[1].0 as u64));
    round_trips(&findings);
}

/// Scenario "Every finding kind has a golden fixture": the coproduct
/// fixture's `c1` overlaps both injections and `c4` lies in neither, so
/// the two failures carry different field sets — `injections` named for
/// the disjointness row, omitted for the coverage row.
#[test]
fn coproduct_witness_golden_fixture() {
    let (model, findings) = checked(COPRODUCT_DOC);

    let dual = row(&model, "C", 0);
    let uncovered = row(&model, "C", 3);
    let expected = vec![
        violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: 0,
            row: dual,
            injections: vec![Ident("i1".to_string()), Ident("i2".to_string())],
        })),
        violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: 0,
            row: uncovered,
            injections: Vec::new(),
        })),
    ];
    assert_eq!(
        findings, expected,
        "one row lies in both images, another in neither"
    );

    let witness = assert_encoding(
        &findings[0],
        "executed",
        "violation",
        &["coproduct", "row", "injections"],
    );
    assert_eq!(witness["coproduct"].as_u64(), Some(0));
    assert_eq!(witness["row"].as_u64(), Some(dual.0 as u64));
    assert_eq!(witness["injections"][0].as_str(), Some("i1"));
    assert_eq!(witness["injections"][1].as_str(), Some("i2"));

    let witness = assert_encoding(&findings[1], "executed", "violation", &["coproduct", "row"]);
    assert_eq!(witness["coproduct"].as_u64(), Some(0));
    assert_eq!(witness["row"].as_u64(), Some(uncovered.0 as u64));
    assert!(
        witness.get("injections").is_none(),
        "coverage omits injections"
    );
    round_trips(&findings);
}
