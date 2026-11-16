//! Purpose: RED contracts for model composition at a seam (openspec change
//! `add-model-composition`, beads `catherine-0qz`): the kernel `glue`
//! operation over named declarations, its conflict codes, and the deferred
//! post-glue equation resolution that makes cross-document seam claims
//! legal.
//!
//! Responsibilities: pin the composition API (`parse_document`, `glue`),
//! the merge rules (objects/arrows/rows identified by name, equations and
//! keys kept as an enforced multiset), the labeled conflict codes with
//! document + node attribution, the empty-interface flag, and the
//! span-preserving `unknown_arrow` raised at glue time.
//!
//! Rationale: design Decisions 2 and 3 — gluing is by declared name and
//! conflicts are labeled errors, never last-writer-wins; equation names
//! resolve once over the pushout, so a single-document glue reproduces
//! today's parse-time `unknown_arrow` (code, position, message) exactly.
//! Assertions are structural (kernel values, codes, positions), never
//! snapshots.

use catherine_core::checks;
use catherine_core::composition::{glue, parse_document};
use catherine_core::error::ParseError;
use catherine_core::findings::{Finding, FindingKind, ViolationWitness};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Instance, RowId, Schema};
use catherine_core::parse::parse;

/// Two documents declaring the same object, arrow, and row names; the
/// overlap agrees (B presents the same sets in a different order).
const AGREE_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Target"],"arrows":[{"name":"link","src":"Order","tgt":"Target"}],"equations":[]},"instance":{"rows":{"Order":["o1","o2"],"Target":["t1","t2"]},"tables":{"link":{"o1":"t1","o2":"t2"}}}}"#;

const AGREE_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Target","Order"],"arrows":[{"name":"link","src":"Order","tgt":"Target"}],"equations":[]},"instance":{"rows":{"Order":["o2","o1"],"Target":["t2","t1"]},"tables":{"link":{"o2":"t2","o1":"t1"}}}}"#;

/// The same arrow `f` declared with different endpoints on each side.
const ENDPOINTS_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Alpha","Beta"],"arrows":[{"name":"f","src":"Alpha","tgt":"Beta"}],"equations":[]},"instance":{"rows":{},"tables":{}}}"#;

const ENDPOINTS_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Alpha","Beta"],"arrows":[{"name":"f","src":"Beta","tgt":"Alpha"}],"equations":[]},"instance":{"rows":{},"tables":{}}}"#;

/// The same row `h1` valued differently in the commonly declared `link`.
const ROWS_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Holder","Target"],"arrows":[{"name":"link","src":"Holder","tgt":"Target"}],"equations":[]},"instance":{"rows":{"Holder":["h1"],"Target":["t1","t2"]},"tables":{"link":{"h1":"t1"}}}}"#;

const ROWS_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Holder","Target"],"arrows":[{"name":"link","src":"Holder","tgt":"Target"}],"equations":[]},"instance":{"rows":{"Holder":["h1"],"Target":["t1","t2"]},"tables":{"link":{"h1":"t2"}}}}"#;

/// Two documents with an agreeing overlap, each declaring one accepted
/// equation the shared row violates (opposite orientations).
const EQUATIONS_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Item","Tag"],"arrows":[{"name":"tag_of","src":"Item","tgt":"Tag"},{"name":"tag_alt","src":"Item","tgt":"Tag"}],"equations":[{"lhs":["tag_of"],"rhs":["tag_alt"],"rationale":"a","status":"accepted"}]},"instance":{"rows":{"Item":["i1"],"Tag":["t1","t2"]},"tables":{"tag_of":{"i1":"t1"},"tag_alt":{"i1":"t2"}}}}"#;

const EQUATIONS_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Item","Tag"],"arrows":[{"name":"tag_of","src":"Item","tgt":"Tag"},{"name":"tag_alt","src":"Item","tgt":"Tag"}],"equations":[{"lhs":["tag_alt"],"rhs":["tag_of"],"rationale":"b","status":"accepted"}]},"instance":{"rows":{"Item":["i1"],"Tag":["t1","t2"]},"tables":{"tag_of":{"i1":"t1"},"tag_alt":{"i1":"t2"}}}}"#;

/// Two documents declaring different accepted keys over the same rows;
/// each key's tuple collides on `i1`/`i2`.
const KEYS_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Item","Tag"],"arrows":[{"name":"tag_of","src":"Item","tgt":"Tag"},{"name":"tag_alt","src":"Item","tgt":"Tag"}],"equations":[],"keys":[{"object":"Item","arrows":["tag_of"],"status":"accepted"}]},"instance":{"rows":{"Item":["i1","i2"],"Tag":["t1","t2"]},"tables":{"tag_of":{"i1":"t1","i2":"t1"},"tag_alt":{"i1":"t1","i2":"t1"}}}}"#;

const KEYS_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Item","Tag"],"arrows":[{"name":"tag_of","src":"Item","tgt":"Tag"},{"name":"tag_alt","src":"Item","tgt":"Tag"}],"equations":[],"keys":[{"object":"Item","arrows":["tag_alt"],"status":"accepted"}]},"instance":{"rows":{"Item":["i1","i2"],"Tag":["t1","t2"]},"tables":{"tag_of":{"i1":"t1","i2":"t1"},"tag_alt":{"i1":"t1","i2":"t1"}}}}"#;

/// A seam claim: the equation lives in the document that declares
/// `invoice_region`, and resolves `dispatch_region` from the sibling input.
const CROSS_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[{"lhs":["invoice_region"],"rhs":["dispatch_region"],"rationale":"SEAM","status":"accepted"}]},"instance":{"rows":{"Order":["o1"],"Region":["north","south"]},"tables":{"invoice_region":{"o1":"north"}}}}"#;

const CROSS_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"dispatch_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{"Order":["o1"],"Region":["north","south"]},"tables":{"dispatch_region":{"o1":"south"}}}}"#;

/// A seam equation whose second path element names no arrow at all.
const DANGLING: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[{"lhs":["invoice_region"],"rhs":["dispatch_region","ghost"],"rationale":"SEAM","status":"accepted"}]},"instance":{"rows":{},"tables":{}}}"#;

/// Today's parse-time `unknown_arrow`: the equation references `q`.
const UNKNOWN: &str = r#"{"format_version":"1.0","schema":{"objects":["A","B","C"],"arrows":[{"name":"f","src":"A","tgt":"B"},{"name":"g","src":"B","tgt":"C"}],"equations":[{"lhs":["f","q"],"rhs":["g"],"rationale":"demo","status":"accepted"}]},"instance":{"rows":{"A":[],"B":[],"C":[]},"tables":{"f":{},"g":{}}}}"#;

const DISJOINT_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Alpha"],"arrows":[],"equations":[]},"instance":{"rows":{"Alpha":["a1"]},"tables":{}}}"#;

const DISJOINT_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Beta"],"arrows":[],"equations":[]},"instance":{"rows":{"Beta":["b1"]},"tables":{}}}"#;

/// Run the kernel's executed checks over a glued composition.
fn checked(schema: &Schema, instance: &Instance) -> Vec<Finding> {
    let typed = TypedInstance::build(schema, instance);
    checks::check(schema, &typed)
}

/// The violation witnesses of a finding list, in canonical order.
fn violations(findings: &[Finding]) -> Vec<&ViolationWitness> {
    findings
        .iter()
        .filter_map(|finding| match &finding.kind {
            FindingKind::Violation(witness) => Some(witness),
            _ => None,
        })
        .collect()
}

/// The equation indices the violation findings name.
fn equation_ids(findings: &[Finding]) -> Vec<usize> {
    violations(findings)
        .into_iter()
        .filter_map(|witness| match witness {
            ViolationWitness::Equation(equation) => Some(equation.equation),
            _ => None,
        })
        .collect()
}

/// The key indices the violation findings name.
fn key_ids(findings: &[Finding]) -> Vec<usize> {
    violations(findings)
        .into_iter()
        .filter_map(|witness| match witness {
            ViolationWitness::Key(key) => Some(key.key),
            _ => None,
        })
        .collect()
}

/// The 1-based line and character column of `needle`'s first occurrence.
fn position_of(text: &str, needle: &str) -> (usize, usize) {
    let offset = text.find(needle).expect("the needle occurs in the fixture");
    let before = &text[..offset];
    let line = before.matches('\n').count() + 1;
    let column = before.chars().rev().take_while(|c| *c != '\n').count() + 1;
    (line, column)
}

#[test]
fn shared_declarations_glue_by_name() {
    let a = parse_document("a.ct.json", AGREE_A).expect("document a parses");
    let b = parse_document("b.ct.json", AGREE_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("the agreeing overlap glues");
    assert_eq!(
        glued.schema.objects.len(),
        2,
        "shared objects collapse by name"
    );
    assert_eq!(
        glued.schema.arrows.len(),
        1,
        "the shared arrow is one declaration"
    );
    assert_eq!(
        glued
            .instance
            .rows
            .get(&Ident("Order".to_string()))
            .map(Vec::len),
        Some(2),
        "rows union by name, not by presentation position"
    );
    assert_eq!(
        glued.instance.values.get(&Ident("link".to_string())),
        Some(&vec![Some(RowId(0)), Some(RowId(1))]),
        "the shared arrow's values resolve into the merged row space"
    );
}

#[test]
fn agreeing_overlap_checks_clean() {
    let a = parse_document("a.ct.json", AGREE_A).expect("document a parses");
    let b = parse_document("b.ct.json", AGREE_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("the agreeing overlap glues");
    let merged = checked(&glued.schema, &glued.instance);
    assert!(
        merged.is_empty(),
        "the agreeing overlap emits no findings: {merged:?}"
    );
    assert!(
        !glued.empty_interface,
        "shared names are a non-empty interface"
    );
}

#[test]
fn disagreeing_arrow_endpoints_are_a_schema_conflict() {
    let a = parse_document("a.ct.json", ENDPOINTS_A).expect("document a parses");
    let b = parse_document("b.ct.json", ENDPOINTS_B).expect("document b parses");
    let error = glue(&[a, b])
        .err()
        .expect("disagreeing endpoints are refused");
    assert_eq!(error.code(), "schema_conflict", "stable conflict code");
    assert_eq!(
        error.documents(),
        vec!["a.ct.json", "b.ct.json"],
        "both documents named"
    );
    assert!(
        error.node().unwrap_or_default().contains('f'),
        "the offending node is named: {:?}",
        error.node()
    );
    assert!(
        error.message().contains('f'),
        "the message names the arrow: {}",
        error.message()
    );
}

#[test]
fn disagreeing_row_values_are_a_row_conflict() {
    let a = parse_document("a.ct.json", ROWS_A).expect("document a parses");
    let b = parse_document("b.ct.json", ROWS_B).expect("document b parses");
    let error = glue(&[a, b]).err().expect("disagreeing values are refused");
    assert_eq!(error.code(), "row_conflict", "stable conflict code");
    assert_eq!(
        error.documents(),
        vec!["a.ct.json", "b.ct.json"],
        "both documents named"
    );
    assert!(
        error.node().unwrap_or_default().contains("h1"),
        "the offending row is named: {:?}",
        error.node()
    );
    assert!(
        error.message().contains("link") && error.message().contains("h1"),
        "the message names the arrow and the row: {}",
        error.message()
    );
}

#[test]
fn equations_from_both_documents_are_enforced() {
    let a = parse_document("a.ct.json", EQUATIONS_A).expect("document a parses");
    let b = parse_document("b.ct.json", EQUATIONS_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("the agreeing overlap glues");
    let merged = checked(&glued.schema, &glued.instance);
    assert_eq!(
        equation_ids(&merged),
        vec![0, 1],
        "both declarations are enforced (multiset, canonical order)"
    );
    assert_eq!(
        violations(&merged).len(),
        2,
        "one violation per declared equation"
    );
}

#[test]
fn keys_from_both_documents_are_enforced() {
    let a = parse_document("a.ct.json", KEYS_A).expect("document a parses");
    let b = parse_document("b.ct.json", KEYS_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("the agreeing overlap glues");
    let merged = checked(&glued.schema, &glued.instance);
    assert_eq!(
        key_ids(&merged),
        vec![0, 1],
        "both key declarations are enforced (multiset, canonical order)"
    );
    assert_eq!(
        violations(&merged).len(),
        2,
        "one collision per declared key"
    );
}

#[test]
fn cross_document_equation_resolves_after_glue() {
    let a = parse_document("billing.ct.json", CROSS_A)
        .expect("a document references a sibling's arrows (syntax-only at document level)");
    let b = parse_document("dispatch.ct.json", CROSS_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("the names resolve over the pushout");
    let merged = checked(&glued.schema, &glued.instance);
    assert_eq!(
        equation_ids(&merged),
        vec![0],
        "the resolved seam equation is executed"
    );
    let witness = violations(&merged)[0];
    match witness {
        ViolationWitness::Equation(equation) => assert_eq!(
            (equation.row.0, equation.lhs.0, equation.rhs.0),
            (0, 0, 1),
            "o1 is invoiced north but dispatched south"
        ),
        other => panic!("the seam finding is an equation violation, got {other:?}"),
    }
}

#[test]
fn dangling_cross_document_reference_is_unknown_arrow_at_glue() {
    let a = parse_document("seam.ct.json", DANGLING).expect("syntax-only at document level");
    let b = parse_document("dispatch.ct.json", CROSS_B).expect("document b parses");
    let error = glue(&[a, b])
        .err()
        .expect("the dangling reference fails at glue time");
    assert_eq!(
        error.code(),
        "unknown_arrow",
        "same stable code as parse time"
    );
    assert_eq!(
        error.documents(),
        vec!["seam.ct.json"],
        "the declaring document is attributed"
    );
    assert_eq!(
        error.position(),
        Some(position_of(DANGLING, "\"ghost\"")),
        "the span survives deferral"
    );
    assert!(
        error
            .message()
            .contains("references undeclared arrow `ghost`"),
        "the kernel message shape: {}",
        error.message()
    );
    assert!(
        error.message().contains("[dispatch_region, ghost]"),
        "the full path is named: {}",
        error.message()
    );
}

#[test]
fn single_document_unknown_arrow_matches_the_parse_time_error() {
    let parse_error = parse(UNKNOWN)
        .err()
        .expect("today's parser rejects the dangling path");
    let ParseError::Semantic(semantic) = parse_error else {
        panic!("today's error is a semantic unknown_arrow");
    };
    let document = parse_document("d.ct.json", UNKNOWN).expect("syntax-only parse accepts it");
    let error = glue(&[document])
        .err()
        .expect("the singleton pushout resolves it");
    assert_eq!(error.code(), semantic.code.as_str(), "same stable code");
    assert_eq!(
        error.position(),
        Some((semantic.line, semantic.column)),
        "same offending node position"
    );
    assert_eq!(
        error.message(),
        semantic.message.as_str(),
        "same message shape"
    );
}

#[test]
fn disjoint_documents_flag_an_empty_interface() {
    let a = parse_document("a.ct.json", DISJOINT_A).expect("document a parses");
    let b = parse_document("b.ct.json", DISJOINT_B).expect("document b parses");
    let glued = glue(&[a, b]).expect("a disjoint union still glues");
    assert!(
        glued.empty_interface,
        "zero shared names is flagged, never silent"
    );
    assert_eq!(
        glued.schema.objects.len(),
        2,
        "the union keeps both schemas"
    );
    let merged = checked(&glued.schema, &glued.instance);
    assert!(
        merged.is_empty(),
        "a disjoint union has no findings: {merged:?}"
    );
}

#[test]
fn a_single_document_glue_preserves_the_parsed_model() {
    let model = parse(AGREE_A).expect("the document parses");
    let document = parse_document("solo.ct.json", AGREE_A).expect("the document parses");
    let glued = glue(&[document]).expect("a singleton pushout is accepted");
    assert_eq!(
        glued.schema, model.schema,
        "the merged schema is the document's schema"
    );
    assert_eq!(
        glued.instance, model.instance,
        "the merged instance is the document's instance"
    );
    assert!(
        !glued.empty_interface,
        "a single document is not a disjoint union"
    );
}
