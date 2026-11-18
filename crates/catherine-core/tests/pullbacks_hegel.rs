//! Purpose: order-stability properties for executed pullback checks
//! (openspec change `add-pullback-checks`, task 5.2) — input presentation
//! order must never reach canonical bytes or findings.
//!
//! Responsibilities: draw a presentation permutation of the same square
//! document with hegel generators — row-name/table-key order permuted,
//! and the pullback declaration list permuted — then parse→build→check
//! it against the unpermuted presentation and assert identical canonical
//! bytes and identical finding vectors.
//!
//! Rationale: the parser normalizes rows to dense sorted ids and stores
//! declarations in canonical `(apex, legs, cospan, status)` order, so
//! presentation perms are invisible; findings are sorted by the
//! taxonomy's total order. Drawing the permutation exercises orders the
//! old fixed pair enumeration never saw, while the property itself is
//! unchanged: canonical bytes of two presentations of one instance (the
//! golden fixture suite remains the byte oracle for output shape). Every
//! permutation of a valid presentation is itself a valid document, so a
//! drawn case needs no `tc.assume`; degenerate draws (identity order,
//! single-entry lists) are vacuously stable rather than panic-prone.

use hegel::generators as gs;
use hegel::TestCase;

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

/// One object's presented row-name list, in the order the document lists
/// the object and names its rows.
type RowsPresentation = Vec<(String, Vec<String>)>;

/// One arrow's presented table, in the order the document lists the
/// arrow and its source-row entries.
type TablesPresentation = Vec<(String, Vec<(String, String)>)>;

/// Draw one presentation order of `items`: the same elements, permuted.
fn permuted<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    tc: &TestCase,
    items: Vec<T>,
) -> Vec<T> {
    tc.draw(gs::permutations(items))
}

/// Serialize one JSON name (object key, row key, or cell value).
fn name_json(name: &str) -> String {
    serde_json::to_string(name).expect("a name serializes as JSON")
}

/// Render the rows section in the drawn presentation order.
fn rows_json(rows: &RowsPresentation) -> String {
    let entries: Vec<String> = rows
        .iter()
        .map(|(object, names)| {
            let names = serde_json::to_string(names).expect("row names serialize as JSON");
            format!("{}: {}", name_json(object), names)
        })
        .collect();
    format!("{{{}}}", entries.join(", "))
}

/// Render the tables section in the drawn presentation order.
fn tables_json(tables: &TablesPresentation) -> String {
    let entries: Vec<String> = tables
        .iter()
        .map(|(arrow, cells)| {
            let cells: Vec<String> = cells
                .iter()
                .map(|(row, value)| format!("{}: {}", name_json(row), name_json(value)))
                .collect();
            format!("{}: {{{}}}", name_json(arrow), cells.join(", "))
        })
        .collect();
    format!("{{{}}}", entries.join(", "))
}

/// One pullback declaration, serialized with fixed key order.
fn declaration_json(apex: &str, legs: [&str; 2], cospan: [&str; 2]) -> String {
    serde_json::json!({
        "apex": apex,
        "legs": legs,
        "cospan": cospan,
        "status": "accepted",
    })
    .to_string()
}

/// Render the declaration list in the drawn presentation order.
fn pullbacks_json(declarations: &[String]) -> String {
    format!("[{}]", declarations.join(", "))
}

/// Row-name and table-key order is presentation: dense ids follow sorted
/// names, so canonical bytes and the single commutes finding are identical
/// for every drawn reordering.
#[hegel::test]
fn pullback_verdicts_are_row_order_stable(tc: TestCase) {
    let pullbacks = pullbacks_json(&[declaration_json("P", ["p1", "p2"], ["f", "g"])]);
    let base_rows: RowsPresentation = vec![
        ("P".to_owned(), vec!["r1".to_owned()]),
        ("A".to_owned(), vec!["a1".to_owned()]),
        ("B".to_owned(), vec!["b1".to_owned()]),
        ("C".to_owned(), vec!["c2".to_owned(), "c1".to_owned()]),
    ];
    let base_tables: TablesPresentation = vec![
        ("g".to_owned(), vec![("b1".to_owned(), "c2".to_owned())]),
        ("p2".to_owned(), vec![("r1".to_owned(), "b1".to_owned())]),
        ("f".to_owned(), vec![("a1".to_owned(), "c1".to_owned())]),
        ("p1".to_owned(), vec![("r1".to_owned(), "a1".to_owned())]),
    ];

    let (unpermuted, findings_unpermuted) = square(
        &pullbacks,
        &rows_json(&base_rows),
        &tables_json(&base_tables),
    );

    // Permute object key order, each object's row-name list, arrow key
    // order, and each arrow's source-row entries — all presentation.
    let rows: RowsPresentation = base_rows
        .into_iter()
        .map(|(object, names)| (object, permuted(&tc, names)))
        .collect();
    let rows = permuted(&tc, rows);
    let tables: TablesPresentation = base_tables
        .into_iter()
        .map(|(arrow, cells)| (arrow, permuted(&tc, cells)))
        .collect();
    let tables = permuted(&tc, tables);
    tc.note(&format!("rows = {rows:?}"));
    tc.note(&format!("tables = {tables:?}"));

    let (reordered, findings_reordered) =
        square(&pullbacks, &rows_json(&rows), &tables_json(&tables));
    assert_eq!(
        canonical_bytes(&unpermuted),
        canonical_bytes(&reordered),
        "row/table order is presentation"
    );
    assert_eq!(
        findings_unpermuted, findings_reordered,
        "identical findings"
    );
    assert_eq!(
        findings_unpermuted.len(),
        1,
        "the square fails commutes exactly once"
    );
}

/// Declaration-list order is presentation too: the parser's canonical
/// sort yields one schema, bytes, and findings (one per declaration).
#[hegel::test]
fn pullback_findings_are_declaration_order_stable(tc: TestCase) {
    let base_declarations = vec![
        declaration_json("P", ["p1", "p2"], ["f", "g"]),
        declaration_json("Q", ["q1", "q2"], ["f", "g"]),
    ];
    let rows = rows_json(&vec![
        ("P".to_owned(), vec!["s1".to_owned()]),
        ("Q".to_owned(), vec!["t1".to_owned()]),
        ("A".to_owned(), vec!["a1".to_owned()]),
        ("B".to_owned(), vec!["b1".to_owned()]),
        ("C".to_owned(), vec!["c1".to_owned(), "c2".to_owned()]),
    ]);
    let tables = tables_json(&vec![
        ("p1".to_owned(), vec![("s1".to_owned(), "a1".to_owned())]),
        ("p2".to_owned(), vec![("s1".to_owned(), "b1".to_owned())]),
        ("q1".to_owned(), vec![("t1".to_owned(), "a1".to_owned())]),
        ("q2".to_owned(), vec![("t1".to_owned(), "b1".to_owned())]),
        ("f".to_owned(), vec![("a1".to_owned(), "c1".to_owned())]),
        ("g".to_owned(), vec![("b1".to_owned(), "c2".to_owned())]),
    ]);

    let (unpermuted, findings_unpermuted) =
        square(&pullbacks_json(&base_declarations), &rows, &tables);

    let declarations = permuted(&tc, base_declarations);
    tc.note(&format!("declarations = {declarations:?}"));
    let (reordered, findings_reordered) = square(&pullbacks_json(&declarations), &rows, &tables);
    assert_eq!(
        unpermuted.schema, reordered.schema,
        "parser stores pullbacks in canonical sort order"
    );
    assert_eq!(
        canonical_bytes(&unpermuted),
        canonical_bytes(&reordered),
        "declaration order is presentation"
    );
    assert_eq!(
        findings_unpermuted, findings_reordered,
        "identical findings"
    );
    assert_eq!(
        findings_unpermuted.len(),
        2,
        "one commutes failure per declaration"
    );
}
