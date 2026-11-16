//! Purpose: the composition agreement rules (openspec change
//! `add-model-composition`, ro5u findings DRAFT-001/CORR-001/CORR-002) —
//! a shared object's row sets, a shared cell's value including the
//! parser's `null`, the namespaced empty-interface predicate, and the
//! parse-order precedence of the deferred equation slot.
//!
//! Responsibilities: pin the two reviewer repros (two clean documents that
//! declare different row sets; a `null` cell against a value), the
//! agreeing `null`/`null` case, the schema-only document that claims no
//! rows, the namespaced interface intersections, and the deferred path's
//! first error for the same bytes the single-document parser sees.
//!
//! Rationale: an instance union meets two claims, so both are checked, not
//! unioned — otherwise a partial instance fabricates a `dangling` finding
//! no input entails (ro5u DRAFT-001). The equation slot keeps the parser's
//! own check over the document's arrows, so a deferred failure is resolved
//! by the same function over the pushout and the first error cannot depend
//! on the arity of the run (ro5u CORR-002). Assertions are structural
//! (kernel codes, positions, row ids), never snapshots.

use catherine_core::checks;
use catherine_core::composition::{glue, parse_document, CompositionError, Document};
use catherine_core::error::ParseError;
use catherine_core::findings::FindingKind;
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, RowId};
use catherine_core::parse::parse;

/// Two documents that each check clean alone but declare different row
/// sets on the shared object `Order` (a carries the extra row `o2`). The
/// union would fabricate a dangling `invoice_region` cell on `o2`.
const PARTIAL_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"ledger_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{"Order":["o1","o2"],"Region":["north"]},"tables":{"ledger_region":{"o1":"north","o2":"north"}}}}"#;

const PARTIAL_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{"Order":["o1"],"Region":["north"]},"tables":{"invoice_region":{"o1":"north"}}}}"#;

/// The same shared cell as an explicit `null` (the parser's dangling
/// value) in one document and a declared target row in the other.
const NULL_CELL: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{"Order":["o1"],"Region":["north"]},"tables":{"invoice_region":{"o1":null}}}}"#;

const VALUE_CELL: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{"Order":["o1"],"Region":["north"]},"tables":{"invoice_region":{"o1":"north"}}}}"#;

/// A document that declares the object's structure but no rows — the
/// seam-claim shape, which carries no row claim.
const ROW_LESS: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{},"tables":{}}}"#;

/// Two disjoint documents whose only common string is a row name `x` on
/// different objects.
const SAME_ROW_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Alpha"],"arrows":[{"name":"alpha_loop","src":"Alpha","tgt":"Alpha"}],"equations":[]},"instance":{"rows":{"Alpha":["x"]},"tables":{"alpha_loop":{"x":"x"}}}}"#;

const SAME_ROW_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Beta"],"arrows":[{"name":"beta_loop","src":"Beta","tgt":"Beta"}],"equations":[]},"instance":{"rows":{"Beta":["x"]},"tables":{"beta_loop":{"x":"x"}}}}"#;

/// Two disjoint documents whose only common string is `x`, one as an arrow
/// on `Beta` and one as a row on `Alpha` — different namespaces.
const CROSS_KIND_A: &str = r#"{"format_version":"1.4","schema":{"objects":["Alpha"],"arrows":[{"name":"loop","src":"Alpha","tgt":"Alpha"}],"equations":[]},"instance":{"rows":{"Alpha":["x"]},"tables":{"loop":{"x":"x"}}}}"#;

const CROSS_KIND_B: &str = r#"{"format_version":"1.4","schema":{"objects":["Beta"],"arrows":[{"name":"x","src":"Beta","tgt":"Beta"}],"equations":[]},"instance":{"rows":{"Beta":["y"]},"tables":{"x":{"y":"y"}}}}"#;

/// The parser's first error is equation 0's undeclared arrow; equation 1
/// carries an invalid status. File order decides, not the check kind.
const PRECEDENCE: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"},{"name":"dispatch_region","src":"Order","tgt":"Region"}],"equations":[{"lhs":["invoice_region"],"rhs":["ghost"],"rationale":"undeclared element"},{"lhs":["invoice_region"],"rhs":["dispatch_region"],"rationale":"bad status","status":"bogus"}]},"instance":{"rows":{},"tables":{}}}"#;

/// Both paths are declared by this document but span different objects: the
/// pushout can never repair that, so the equation slot raises it.
const ILL_TYPED_LOCAL: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Ledger","Region"],"arrows":[{"name":"ledger","src":"Order","tgt":"Ledger"},{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[{"lhs":["ledger"],"rhs":["invoice_region"],"rationale":"mismatch"}]},"instance":{"rows":{},"tables":{}}}"#;

/// A seam equation whose arrow a sibling declares, followed by an equation
/// with an invalid status: the seam resolves over the pushout, so the
/// status error is the run's first error.
const SEAM_STATUS: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"}],"equations":[{"lhs":["invoice_region"],"rhs":["dispatch_region"],"rationale":"seam"},{"lhs":["invoice_region"],"rhs":["invoice_region"],"rationale":"bad status","status":"bogus"}]},"instance":{"rows":{},"tables":{}}}"#;

/// The sibling that declares `dispatch_region` — the seam claim's other
/// half, structure only.
const SEAM_SIBLING: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"dispatch_region","src":"Order","tgt":"Region"}],"equations":[]},"instance":{"rows":{},"tables":{}}}"#;

/// Both equations resolve; the second carries the invalid status, so the
/// deferral must not swallow a document-level error.
const STATUS_SECOND: &str = r#"{"format_version":"1.4","schema":{"objects":["Order","Region"],"arrows":[{"name":"invoice_region","src":"Order","tgt":"Region"},{"name":"dispatch_region","src":"Order","tgt":"Region"}],"equations":[{"lhs":["invoice_region"],"rhs":["dispatch_region"],"rationale":"first"},{"lhs":["invoice_region"],"rhs":["dispatch_region"],"rationale":"second","status":"bogus"}]},"instance":{"rows":{},"tables":{}}}"#;

/// Parse one fixture as a named document.
fn document(name: &str, text: &str) -> Document {
    parse_document(name, text).expect("the document parses at document level")
}

/// The labeled failure of gluing two fixture documents.
fn glue_failure(first: (&str, &str), second: (&str, &str)) -> CompositionError {
    let a = document(first.0, first.1);
    let b = document(second.0, second.1);
    glue(&[a, b]).expect_err("the overlap is refused")
}

/// Two clean documents that disagree on a shared object's row set are a
/// labeled conflict naming both documents and the differing row.
#[test]
fn differing_row_sets_are_a_row_conflict() {
    let error = glue_failure(("a.ct.json", PARTIAL_A), ("b.ct.json", PARTIAL_B));
    assert_eq!(error.code(), "row_conflict", "stable conflict code");
    assert_eq!(
        error.documents(),
        vec!["a.ct.json", "b.ct.json"],
        "both declaring documents named"
    );
    assert_eq!(error.node(), Some("Order"), "the shared object is named");
    assert!(
        error.message().contains("differing rows: [o2]"),
        "the differing row names are named: {}",
        error.message()
    );
}

/// A `null` cell is a value claim: against a declared target row it is a
/// conflict, never a silent fill.
#[test]
fn a_dangling_cell_against_a_value_is_a_row_conflict() {
    let error = glue_failure(("nul.ct.json", NULL_CELL), ("val.ct.json", VALUE_CELL));
    assert_eq!(error.code(), "row_conflict", "stable conflict code");
    assert_eq!(
        error.documents(),
        vec!["nul.ct.json", "val.ct.json"],
        "both declaring documents named"
    );
    assert!(
        error.message().contains("<dangling>") && error.message().contains("north"),
        "both claims are spelled: {}",
        error.message()
    );
}

/// `null` against `null` agrees: the merged cell stays dangling, exactly as
/// each input declares it.
#[test]
fn agreeing_dangling_cells_glue() {
    let a = document("a.ct.json", NULL_CELL);
    let b = document("b.ct.json", NULL_CELL);
    let glued = glue(&[a, b]).expect("two identical dangling claims agree");
    let typed = TypedInstance::build(&glued.schema, &glued.instance);
    let findings = checks::check(&glued.schema, &typed);
    assert_eq!(
        findings.len(),
        1,
        "the declared dangling cell: {findings:?}"
    );
    assert!(
        matches!(findings[0].kind, FindingKind::Dangling(_)),
        "the merged cell is dangling, not fabricated away: {:?}",
        findings[0].kind
    );
}

/// A document that declares no rows for an object carries no row claim, so
/// it never conflicts with a sibling's row set.
#[test]
fn a_document_without_rows_claims_no_row_set() {
    let a = document("a.ct.json", PARTIAL_A);
    let b = document("seam.ct.json", ROW_LESS);
    let glued = glue(&[a, b]).expect("the schema-only document claims no rows");
    assert_eq!(
        glued
            .instance
            .rows
            .get(&Ident("Order".to_string()))
            .map(Vec::len),
        Some(2),
        "the declaring document's row set is the merged one"
    );
    assert!(!glued.empty_interface, "the shared object is an interface");
}

/// The empty-interface predicate compares namespaced identities, so row
/// names reused across objects (or a row name reused as an arrow name) are
/// not an interface.
#[test]
fn the_empty_interface_is_namespaced() {
    let a = document("a.ct.json", SAME_ROW_A);
    let b = document("b.ct.json", SAME_ROW_B);
    let glued = glue(&[a, b]).expect("a disjoint union still glues");
    assert!(
        glued.empty_interface,
        "a shared bare row name is not a shared identity"
    );
    let a = document("a.ct.json", CROSS_KIND_A);
    let b = document("b.ct.json", CROSS_KIND_B);
    let glued = glue(&[a, b]).expect("a disjoint union still glues");
    assert!(
        glued.empty_interface,
        "an arrow name and a row name are different namespaces"
    );
    let identical = document("a.ct.json", SAME_ROW_A);
    let glued = glue(&[identical.clone(), identical]).expect("a repeated input glues");
    assert!(
        !glued.empty_interface,
        "a shared object, arrow, and (object, row) pair is an interface"
    );
}

/// The same bytes produce the same first error code: the deferred path
/// resolves over the pushout with the parser's own check and order.
#[test]
fn deferred_resolution_keeps_parse_precedence() {
    let parse_error = parse(PRECEDENCE).expect_err("the parser rejects equation 0");
    let ParseError::Semantic(semantic) = parse_error else {
        panic!("the first error is a semantic code");
    };
    let document = document("p.ct.json", PRECEDENCE);
    let error =
        glue(&[document.clone(), document]).expect_err("the joint run rejects the same equation");
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

/// A status error is a document-level failure, not a name resolution: the
/// equation slot raises it exactly as the parser does.
#[test]
fn a_status_error_is_raised_in_the_equation_slot() {
    let parse_error = parse(STATUS_SECOND).expect_err("the parser rejects it");
    let ParseError::Semantic(semantic) = parse_error else {
        panic!("the first error is a semantic code");
    };
    let error =
        parse_document("s.ct.json", STATUS_SECOND).expect_err("the deferred parse raises it too");
    assert_eq!(
        semantic.code.as_str(),
        "invalid_status",
        "the parser's code"
    );
    assert_eq!(
        error,
        ParseError::Semantic(semantic),
        "the deferred parse reports the parser's error verbatim"
    );
}

/// A merged row id that no document claims is never produced: glue's row
/// space is the agreed set.
#[test]
fn the_merged_row_space_is_the_agreed_set() {
    let a = document("a.ct.json", PARTIAL_A);
    let b = document("b.ct.json", PARTIAL_A);
    let glued = glue(&[a, b]).expect("identical row sets agree");
    assert_eq!(
        glued
            .instance
            .rows
            .get(&Ident("Order".to_string()))
            .map(Vec::len),
        Some(2),
        "the merged row space is the agreed set"
    );
    assert_eq!(
        glued
            .instance
            .values
            .get(&Ident("ledger_region".to_string())),
        Some(&vec![Some(RowId(0)), Some(RowId(0))]),
        "every merged cell is a claim one input carries (no fabricated dangling)"
    );
}

/// The pushout satisfies the deferred path, so the later equation's invalid
/// status is the run's first error — resolution never swallows it.
#[test]
fn a_status_error_behind_a_resolved_seam_is_still_raised() {
    let seam = document("seam.ct.json", SEAM_STATUS);
    let sibling = document("sibling.ct.json", SEAM_SIBLING);
    let error = glue(&[seam, sibling]).expect_err("the later invalid status is still an error");
    assert_eq!(error.code(), "invalid_status", "the parser's code");
    assert_eq!(
        error.documents(),
        vec!["seam.ct.json"],
        "the declaring document is attributed"
    );
    assert!(
        error.message().contains("`bogus`"),
        "the offending status is named: {}",
        error.message()
    );
}

/// A local endpoint mismatch is not deferrable: a commonly declared arrow
/// keeps its endpoints, so a union cannot repair it and the equation slot
/// reports the parser's error verbatim.
#[test]
fn a_local_endpoint_mismatch_is_raised_in_the_equation_slot() {
    let parse_error = parse(ILL_TYPED_LOCAL).expect_err("the parser rejects the mismatch");
    let ParseError::Semantic(semantic) = parse_error else {
        panic!("the first error is a semantic code");
    };
    assert_eq!(semantic.code.as_str(), "ill_typed_equation", "the code");
    let error =
        parse_document("t.ct.json", ILL_TYPED_LOCAL).expect_err("the deferred parse raises it too");
    assert_eq!(
        error,
        ParseError::Semantic(semantic),
        "the deferred parse reports the parser's error verbatim"
    );
}
