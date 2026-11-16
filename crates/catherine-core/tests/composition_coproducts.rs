//! Purpose: coproduct declarations through model composition (openspec
//! change `add-coproduct-exhaustiveness`): the glue's enforced multiset
//! carries every input's coproducts, and each merged declaration keeps its
//! declaring-document attribution.
//!
//! Responsibilities: pin two documents' coproducts surviving `glue` in
//! canonical order, the per-declaration `CoproductSource` attribution
//! (declaring document plus the injection arrows' documents), and the
//! executed check emitting one coverage finding per declaration.
//!
//! Rationale: composition reuses the keys merge rule verbatim (design
//! Decision 3), so a joint run must execute both declarations and never
//! collapse one into the other; assertions are structural kernel values,
//! never snapshots.

use catherine_core::checks;
use catherine_core::composition::{glue, parse_document};
use catherine_core::findings::{Finding, FindingKind, ViolationWitness};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Instance, Schema};

/// Two documents declaring different accepted coproducts over the same
/// `Method` object and injections; each covers only one of the two rows.
const COPRODUCTS_A: &str = r#"{"format_version":"1.5","schema":{"objects":["Method","Card","Transfer"],"arrows":[{"name":"card_inj","src":"Card","tgt":"Method"},{"name":"transfer_inj","src":"Transfer","tgt":"Method"}],"equations":[],"coproducts":[{"object":"Method","arrows":["card_inj"],"status":"accepted"}]},"instance":{"rows":{"Method":["card","cash"],"Card":["card"],"Transfer":["transfer"]},"tables":{"card_inj":{"card":"card"},"transfer_inj":{"transfer":"cash"}}}}"#;

const COPRODUCTS_B: &str = r#"{"format_version":"1.5","schema":{"objects":["Method","Card","Transfer"],"arrows":[{"name":"card_inj","src":"Card","tgt":"Method"},{"name":"transfer_inj","src":"Transfer","tgt":"Method"}],"equations":[],"coproducts":[{"object":"Method","arrows":["transfer_inj"],"status":"accepted"}]},"instance":{"rows":{"Method":["card","cash"],"Card":["card"],"Transfer":["transfer"]},"tables":{"card_inj":{"card":"card"},"transfer_inj":{"transfer":"cash"}}}}"#;

/// Run the kernel's executed checks over a glued composition.
fn checked(schema: &Schema, instance: &Instance) -> Vec<Finding> {
    let typed = TypedInstance::build(schema, instance);
    checks::check(schema, &typed)
}

/// The coproduct `(index, row)` pairs the violation findings name.
fn coproduct_rows(findings: &[Finding]) -> Vec<(usize, usize)> {
    findings
        .iter()
        .filter_map(|finding| match &finding.kind {
            FindingKind::Violation(ViolationWitness::Coproduct(coproduct)) => {
                Some((coproduct.coproduct, coproduct.row.0))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn coproducts_from_both_documents_are_enforced() {
    let a = parse_document("a.ct.json", COPRODUCTS_A).expect("document a parses");
    let b = parse_document("b.ct.json", COPRODUCTS_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("the agreeing overlap glues");
    assert_eq!(
        glued.schema.coproducts.len(),
        2,
        "every coproduct declaration is kept (multiset, canonical order)"
    );
    assert_eq!(
        glued
            .coproduct_sources
            .iter()
            .map(|source| source.declaring.as_str())
            .collect::<Vec<_>>(),
        vec!["a.ct.json", "b.ct.json"],
        "each declaration is attributed to its declaring document"
    );
    assert_eq!(
        glued.coproduct_sources[0].arrows,
        vec!["a.ct.json".to_string(), "b.ct.json".to_string()],
        "the shared injection's declaring documents are attributed in order"
    );
    let merged = checked(&glued.schema, &glued.instance);
    assert_eq!(
        coproduct_rows(&merged),
        vec![(0, 1), (1, 0)],
        "both declarations execute: each names the row its sibling covers"
    );
    assert_eq!(
        merged.len(),
        2,
        "one coverage finding per declared coproduct"
    );
}
