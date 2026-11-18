//! Purpose: order-stability properties for executed pullback checks
//! (openspec change `add-pullback-checks`, task 5.2) — input presentation
//! order must never reach canonical bytes or findings.
//!
//! Responsibilities: parse→build→check two presentations of the same
//! square document — row-name/table-key order permuted, and the pullback
//! declaration list permuted — and assert identical canonical bytes and
//! identical finding vectors.
//!
//! Rationale: the parser normalizes rows to dense sorted ids and stores
//! declarations in canonical `(apex, legs, cospan, status)` order, so
//! presentation perms are invisible; findings are sorted by the
//! taxonomy's total order. These are properties, not snapshots, and the
//! golden fixture suite remains the byte oracle for output shape.

use catherine_core::canonical::canonical_bytes;
use catherine_core::checks::check;
use catherine_core::findings::Finding;
use catherine_core::instance::TypedInstance;
use catherine_core::model::Model;
use catherine_core::parse::parse;

/// A 1.6 square over `P --p1--> A --f--> C` and `Q --q1--> A --f--> C`
/// (the same cospan); declaration list, rows, and tables are spliced in.
const SQUARE_DOC: &str = r#"{
  "format_version": "1.6", "schema": {
    "objects": ["P", "Q", "A", "B", "C"],
    "arrows": [
      {"name": "p1", "src": "P", "tgt": "A"}, {"name": "p2", "src": "P", "tgt": "B"}, {"name": "q1", "src": "Q", "tgt": "A"},
      {"name": "q2", "src": "Q", "tgt": "B"}, {"name": "f", "src": "A", "tgt": "C"}, {"name": "g", "src": "B", "tgt": "C"}],
    "equations": [], "pullbacks": __PULLBACKS__},
  "instance": {"rows": __ROWS__, "tables": __TABLES__}}"#;

/// Parse → build → check the square document with the spliced sections.
fn square(pullbacks: &str, rows: &str, tables: &str) -> (Model, Vec<Finding>) {
    let text = SQUARE_DOC
        .replace("__PULLBACKS__", pullbacks)
        .replace("__ROWS__", rows)
        .replace("__TABLES__", tables);
    let model = parse(&text).expect("the square document parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    let findings = check(&model.schema, &built);
    (model, findings)
}

/// Row-name and table-key order is presentation: dense ids follow sorted
/// names, so canonical bytes and the single commutes finding are identical.
#[test]
fn pullback_verdicts_are_row_order_stable() {
    let p = r#"[{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"], "status": "accepted"}]"#;
    let rows_a = r#"{"P": ["r1"], "A": ["a1"], "B": ["b1"], "C": ["c2", "c1"]}"#;
    let rows_b = r#"{"P": ["r1"], "A": ["a1"], "B": ["b1"], "C": ["c1", "c2"]}"#;
    let tables_a =
        r#"{"g": {"b1": "c2"}, "p2": {"r1": "b1"}, "f": {"a1": "c1"}, "p1": {"r1": "a1"}}"#;
    let tables_b =
        r#"{"p1": {"r1": "a1"}, "p2": {"r1": "b1"}, "f": {"a1": "c1"}, "g": {"b1": "c2"}}"#;
    let (a, findings_a) = square(p, rows_a, tables_a);
    let (b, findings_b) = square(p, rows_b, tables_b);
    assert_eq!(
        canonical_bytes(&a),
        canonical_bytes(&b),
        "row/table order is presentation"
    );
    assert_eq!(findings_a, findings_b, "identical findings");
    assert_eq!(
        findings_a.len(),
        1,
        "the square fails commutes exactly once"
    );
}

/// Declaration-list order is presentation too: the parser's canonical
/// sort yields one schema, bytes, and findings (one per declaration).
#[test]
fn pullback_findings_are_declaration_order_stable() {
    let square_p =
        r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"], "status": "accepted"}"#;
    let square_q =
        r#"{"apex": "Q", "legs": ["q1", "q2"], "cospan": ["f", "g"], "status": "accepted"}"#;
    let rows = r#"{"P": ["s1"], "Q": ["t1"], "A": ["a1"], "B": ["b1"], "C": ["c1", "c2"]}"#;
    let tables = r#"{"p1": {"s1": "a1"}, "p2": {"s1": "b1"}, "q1": {"t1": "a1"}, "q2": {"t1": "b1"}, "f": {"a1": "c1"}, "g": {"b1": "c2"}}"#;
    let (a, findings_a) = square(&format!("[{square_p}, {square_q}]"), rows, tables);
    let (b, findings_b) = square(&format!("[{square_q}, {square_p}]"), rows, tables);
    assert_eq!(
        a.schema, b.schema,
        "parser stores pullbacks in canonical sort order"
    );
    assert_eq!(
        canonical_bytes(&a),
        canonical_bytes(&b),
        "declaration order is presentation"
    );
    assert_eq!(findings_a, findings_b, "identical findings");
    assert_eq!(findings_a.len(), 2, "one commutes failure per declaration");
}
