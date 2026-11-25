//! Purpose: the findings-taxonomy properties (hegel, never snapshots;
//! openspec change `add-findings-taxonomy`, task 6) — serde JSON
//! round-trip losslessness, byte-identical serialization of the *check
//! output* across repeated runs, the total order holding on check output,
//! and kernel label/kind soundness over arbitrary well-typed models, plus
//! every finding kind (analysis kinds included — representable values,
//! though this crate produces none) round-tripping from arbitrary
//! witnesses. Assertions are structural and by parsed value — never byte
//! snapshots.
//!
//! Responsibilities: hold the four drawn properties and their per-arm
//! assertion helpers, and import the witness generators they draw from
//! the sibling `generators` module — labels, idents, paths, domain
//! questions and the six-way finding matrix. The model generator itself
//! is the shared `common::generators::arb_case`, not a local copy.
//!
//! Rationale: the witness generators have exactly one consumer (this
//! file), so they live beside it rather than in the shared module, next
//! to the properties that pin their shape. The draws mirror the
//! strategies they replaced, 1:1 — 3-way labels, `0..8` row/id indices,
//! 1..=3-ident paths, at-most-40-char question text and equal weight
//! over the six kinds — so the port explores the space the properties
//! were written against. `Finding`'s order is kind-first, label last;
//! the tiebreak property exists to keep that exact shape honest.

mod common;
#[path = "findings_hegel/generators.rs"]
mod generators;

use common::fixture;
use common::generators::arb_case;
use generators::arb_finding;

use catherine_core::checks::check;
use catherine_core::findings::{
    CoproductWitness, DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind,
    KeyWitness, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Schema};

use hegel::TestCase;

/// Any serializable value as JSON text, or the test fails loudly.
fn to_json<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_string(value) {
        Ok(text) => text,
        Err(error) => panic!("value does not serialize: {error}"),
    }
}

/// JSON text deserialized, or the test fails loudly.
fn from_json<T: serde::de::DeserializeOwned>(text: &str) -> T {
    match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => panic!("value does not deserialize: {error}"),
    }
}

/// Findings round-trip through serde JSON losslessly, and repeated
/// checks — separate invocations over fresh builds — serialize
/// byte-identically; the emitted list is in the taxonomy's total
/// order (kind, then witness ids).
#[hegel::test]
fn findings_round_trip_and_serialize_stably(tc: TestCase) {
    let case = tc.draw(arb_case());
    tc.note(&format!("case: {case:?}"));
    let (schema, instance) = fixture::assemble(&case);
    let built = TypedInstance::build(&schema, &instance);
    let findings = check(&schema, &built);
    let first = to_json(&findings);
    let rebuilt = TypedInstance::build(&schema, &instance);
    let again = to_json(&check(&schema, &rebuilt));
    assert_eq!(first, again, "repeated checks serialize byte-identically");
    let back: Vec<Finding> = from_json(&first);
    assert_eq!(findings, back, "serde round-trip is lossless");
    assert!(
        findings.windows(2).all(|w| w[0] <= w[1]),
        "check output is in the taxonomy's total order"
    );
}

/// Every arbitrary finding — all five kinds, all three labels —
/// round-trips through serde JSON losslessly and serializes
/// deterministically (same value, same bytes, twice).
#[hegel::test]
fn every_kind_round_trips_and_serializes_deterministically(tc: TestCase) {
    let finding = tc.draw(arb_finding());
    tc.note(&format!("finding: {finding:?}"));
    let first = to_json(&finding);
    let back: Finding = from_json(&first);
    assert_eq!(finding, back, "serde round-trip is lossless");
    let second = to_json(&finding);
    assert_eq!(first, second, "serialization is deterministic");
}

/// The total order is kind-first, label last: with different kinds,
/// swapping the labels leaves the order unchanged (the label never
/// overrides kind order); with equal kinds — same witnesses —
/// swapping the labels swaps the order (the label is the tiebreak).
#[hegel::test]
fn label_is_only_a_tiebreak(tc: TestCase) {
    let a = tc.draw(arb_finding());
    let b = tc.draw(arb_finding());
    tc.note(&format!("a: {a:?}\nb: {b:?}"));
    let (la, lb) = (a.label, b.label);
    let a2 = Finding {
        label: lb,
        kind: a.kind.clone(),
    };
    let b2 = Finding {
        label: la,
        kind: b.kind.clone(),
    };
    if a.kind != b.kind {
        assert_eq!(
            a.cmp(&b),
            a2.cmp(&b2),
            "with different kinds, the labels do not affect the order"
        );
    } else {
        assert_eq!(
            a2.cmp(&b2),
            b.cmp(&a),
            "with equal kinds, swapping labels swaps the order — the label is the tiebreak"
        );
    }
}

/// Kernel findings are label- and kind-sound over arbitrary
/// well-typed models: labels only executed/hypothesized and mirroring
/// equation status, witnesses in scope, and none of the analysis
/// kinds (their producers are P4's).
#[hegel::test]
fn kernel_findings_are_label_and_kind_sound(tc: TestCase) {
    let case = tc.draw(arb_case());
    tc.note(&format!("case: {case:?}"));
    let (schema, instance) = fixture::assemble(&case);
    let built = TypedInstance::build(&schema, &instance);
    for finding in check(&schema, &built) {
        assert!(
            matches!(
                finding.label,
                EpistemicLabel::Executed | EpistemicLabel::Hypothesized
            ),
            "kernel labels are executed/hypothesized only, got {:?}",
            finding.label
        );
        assert_kernel_witness_sound((&schema, &built), &finding);
    }
}

/// One kernel finding's witnesses stay sound: the arm for each kind
/// delegates to the small assertion helpers below, and the arms with no
/// kernel producer refuse their witnesses.
fn assert_kernel_witness_sound(inputs: (&Schema, &TypedInstance), finding: &Finding) {
    match &finding.kind {
        FindingKind::Dangling(w) => assert_dangling_witness_sound(inputs, (finding.label, w)),
        FindingKind::Violation(ViolationWitness::Equation(w)) => {
            assert_equation_witness_sound(inputs, (finding.label, w))
        }

        FindingKind::Violation(ViolationWitness::Naturality(_)) => {
            panic!("check never emits naturality witnesses");
        }

        FindingKind::Violation(ViolationWitness::MigrationNaturality(_)) => {
            panic!("check never emits migration-naturality witnesses");
        }

        FindingKind::Violation(ViolationWitness::Coproduct(w)) => {
            assert_coproduct_witness_sound(inputs, (finding.label, w))
        }

        FindingKind::Violation(ViolationWitness::Key(w)) => {
            assert_key_witness_sound(inputs, (finding.label, w))
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

/// A dangling witness is executed, names a declared arrow, and names a
/// row of that arrow's source object.
fn assert_dangling_witness_sound(
    inputs: (&Schema, &TypedInstance),
    finding: (EpistemicLabel, &DanglingWitness),
) {
    let (schema, built) = inputs;
    let (label, w) = finding;
    assert_eq!(label, EpistemicLabel::Executed, "dangling is executed");
    assert!(
        built.tables().contains_key(&w.arrow),
        "dangling arrow declared"
    );
    let n = built
        .rows()
        .get(&schema.arrows[&w.arrow].src)
        .map(Vec::len)
        .unwrap_or(0);
    assert!(w.src.0 < n, "dangling src row in the arrow's source space");
}

/// An equation witness names a valid equation, a row of the equation's
/// domain object, and composed values of the path's target; the label
/// mirrors the equation's status.
fn assert_equation_witness_sound(
    inputs: (&Schema, &TypedInstance),
    finding: (EpistemicLabel, &EquationWitness),
) {
    let (schema, built) = inputs;
    let (label, w) = finding;
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
    let tgt = built
        .rows()
        .get(&schema.arrows[&eq.lhs.0[eq.lhs.0.len() - 1]].tgt)
        .map(Vec::len)
        .unwrap_or(0);
    assert!(
        w.lhs.0 < tgt && w.rhs.0 < tgt,
        "composed values are rows of the path's target"
    );
}

/// A coproduct witness names a valid coproduct and a row of the
/// coproduct's object; the label mirrors the coproduct's status.
fn assert_coproduct_witness_sound(
    inputs: (&Schema, &TypedInstance),
    finding: (EpistemicLabel, &CoproductWitness),
) {
    let (schema, built) = inputs;
    let (label, w) = finding;
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

/// A key witness names a valid key and ascending rows of the key's
/// object; the label mirrors the key's status.
fn assert_key_witness_sound(
    inputs: (&Schema, &TypedInstance),
    finding: (EpistemicLabel, &KeyWitness),
) {
    let (schema, built) = inputs;
    let (label, w) = finding;
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
