//! Purpose: the in-scope witness assertions for `checks_hegel.rs` — one
//! helper per finding kind, each asserting the witness names declared
//! schema elements and rows of the right instance spaces. Split out of
//! `checks_hegel.rs` (via `#[path]`) to keep both files inside the
//! file-size gate; behavior is unchanged.

use catherine_core::findings::{
    CoproductWitness, DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind,
    KeyWitness, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Schema};

/// to the small assertion helpers below, in the same order as before.
pub(crate) fn assert_witness_in_scope(schema: &Schema, built: &TypedInstance, finding: &Finding) {
    match &finding.kind {
        FindingKind::Dangling(w) => {
            assert_dangling_witness_in_scope(schema, built, finding.label, w)
        }
        FindingKind::Violation(ViolationWitness::Equation(w)) => {
            assert_equation_witness_in_scope(schema, built, finding.label, w)
        }

        FindingKind::Violation(ViolationWitness::Naturality(_)) => {
            panic!("check never emits naturality witnesses");
        }

        FindingKind::Violation(ViolationWitness::MigrationNaturality(_)) => {
            panic!("check never emits migration-naturality witnesses");
        }

        FindingKind::Violation(ViolationWitness::Coproduct(w)) => {
            assert_coproduct_witness_in_scope(schema, built, finding.label, w)
        }

        FindingKind::Violation(ViolationWitness::Key(w)) => {
            assert_key_witness_in_scope(schema, built, finding.label, w)
        }
        // The generated fixture declares no pullbacks
        // (`fixture::assemble` always fills `pullbacks` with
        // `Vec::new()`), so the pullback pass has nothing to
        // evaluate; this arm stays honest by refusing a witness
        // the fixture cannot justify.
        FindingKind::Violation(ViolationWitness::Pullback(_)) => {
            panic!(
                "the generated fixture declares no pullbacks, so the check emits no pullback witness"
            );
        }
        FindingKind::Redundant(_)
        | FindingKind::Contradiction(_)
        | FindingKind::Underconstraint(_)
        // The granularity lint is CLI-side over the loaded model
        // (ticket `catherine-kw9.3`); the kernel `check` never emits it.
        | FindingKind::Granularity(_) => {
            panic!(
                "the analysis kinds have no kernel producer (P4's), got {:?}",
                finding.kind
            );
        }
    }
}

/// A dangling witness names a declared arrow and a row of that arrow's
/// source object.
pub(crate) fn assert_dangling_witness_in_scope(
    schema: &Schema,
    built: &TypedInstance,
    label: EpistemicLabel,
    w: &DanglingWitness,
) {
    assert_eq!(label, EpistemicLabel::Executed, "dangling is executed");
    let cells = built.tables().get(&w.arrow);
    assert!(cells.is_some(), "dangling arrow `{w:?}` declared");
    let n = built
        .rows()
        .get(&schema.arrows[&w.arrow].src)
        .map(Vec::len)
        .unwrap_or(0);
    assert!(w.src.0 < n, "dangling src row in the arrow's source space");
}

/// An equation violation names a valid equation, a row of the equation's
/// domain and composed values of its path's target; the label mirrors the
/// equation's status.
pub(crate) fn assert_equation_witness_in_scope(
    schema: &Schema,
    built: &TypedInstance,
    label: EpistemicLabel,
    w: &EquationWitness,
) {
    assert!(w.equation < schema.equations.len(), "equation index valid");
    let eq = &schema.equations[w.equation];
    assert_eq!(
        label == EpistemicLabel::Hypothesized,
        eq.status == ConstraintStatus::Proposed,
        "the label mirrors the equation's status"
    );
    let lead = &schema.arrows[&eq.lhs.0[0]];
    let n = built.rows().get(&lead.src).map(Vec::len).unwrap_or(0);
    assert!(w.row.0 < n, "row is a row of the equation's domain object");
    let tgt_space = built
        .rows()
        .get(&schema.arrows[&eq.lhs.0[eq.lhs.0.len() - 1]].tgt)
        .map(Vec::len)
        .unwrap_or(0);
    assert!(
        w.lhs.0 < tgt_space && w.rhs.0 < tgt_space,
        "composed values are rows of the path's target"
    );
}

/// A coproduct violation names a valid coproduct and a row of its object;
/// the label mirrors the coproduct's status.
pub(crate) fn assert_coproduct_witness_in_scope(
    schema: &Schema,
    built: &TypedInstance,
    label: EpistemicLabel,
    w: &CoproductWitness,
) {
    assert!(
        w.coproduct < schema.coproducts.len(),
        "coproduct index valid"
    );
    let coproduct = &schema.coproducts[w.coproduct];
    let n = built
        .rows()
        .get(&coproduct.object)
        .map(Vec::len)
        .unwrap_or(0);
    assert!(
        w.row.0 < n,
        "witness row is a row of the coproduct's object"
    );
    assert_eq!(
        label == EpistemicLabel::Hypothesized,
        coproduct.status == ConstraintStatus::Proposed,
        "the label mirrors the coproduct's status"
    );
}

/// A key violation names a valid key, ascending rows of the key's object;
/// the label mirrors the key's status.
pub(crate) fn assert_key_witness_in_scope(
    schema: &Schema,
    built: &TypedInstance,
    label: EpistemicLabel,
    w: &KeyWitness,
) {
    assert!(w.key < schema.keys.len(), "key index valid");
    let key = &schema.keys[w.key];
    let n = built.rows().get(&key.object).map(Vec::len).unwrap_or(0);
    assert!(
        w.rows[0].0 < n && w.rows[1].0 < n,
        "witness rows are rows of the key's object"
    );
    assert!(w.rows[0] < w.rows[1], "witness rows ascending");
    assert_eq!(
        label == EpistemicLabel::Hypothesized,
        key.status == ConstraintStatus::Proposed,
        "the label mirrors the key's status"
    );
}
