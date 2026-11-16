//! Purpose: the coproduct violation's serialized shape (openspec change
//! `add-coproduct-exhaustiveness`, the findings delta): the coverage and
//! disjointness witnesses encode exactly as the findings spec pins them,
//! and the equation-witness bytes stay unchanged by the added arm.
//!
//! Responsibilities: pin `{"coproduct": n, "row": r}` with `injections`
//! omitted for a coverage failure, the same shape plus the colliding
//! injection names for a disjointness failure, and the pre-coproduct
//! equation-witness JSON byte-for-byte.
//!
//! Rationale: consumers discriminate the untagged violation witness by its
//! field set, so the encoding is the contract — asserted structurally on
//! the serialized value, never through output text.

use catherine_core::findings::{
    CoproductWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, ViolationWitness,
};
use catherine_core::model::{Ident, RowId};

/// Scenario "Coproduct violation carries its row and injections": both
/// witness shapes serialize exactly as the findings spec pins them — the
/// coverage shape omits `injections`, the disjointness shape names the
/// colliding pair — so a consumer distinguishes them structurally.
#[test]
fn coproduct_witness_encodings_are_stable() {
    let uncovered = Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: 0,
            row: RowId(3),
            injections: Vec::new(),
        })),
    };
    assert_eq!(
        serde_json::to_string(&uncovered).expect("coverage witness serializes"),
        r#"{"label":"executed","kind":"violation","witness":{"coproduct":0,"row":3}}"#,
        "the coverage witness carries no empty injections field"
    );

    let dual = Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: 0,
            row: RowId(2),
            injections: vec![
                Ident("card_inj".to_string()),
                Ident("transfer_inj".to_string()),
            ],
        })),
    };
    assert_eq!(
        serde_json::to_string(&dual).expect("disjointness witness serializes"),
        r#"{"label":"executed","kind":"violation","witness":{"coproduct":0,"row":2,"injections":["card_inj","transfer_inj"]}}"#,
        "the disjointness witness names both colliding injections"
    );
}

/// The equation-witness encoding is unchanged by the added coproduct
/// witness arm: the serialized bytes are exactly the pre-coproduct shape.
#[test]
fn equation_violation_json_is_unchanged() {
    let finding = Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: 0,
            row: RowId(1),
            lhs: RowId(2),
            rhs: RowId(3),
        })),
    };
    let text = serde_json::to_string(&finding).expect("finding serializes");
    assert_eq!(
        text,
        r#"{"label":"executed","kind":"violation","witness":{"equation":0,"row":1,"lhs":2,"rhs":3}}"#
    );
}
