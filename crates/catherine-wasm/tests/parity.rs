//! Purpose: the parity gate (openspec change `add-wasm-web-view`, task
//! 7) — the wasm boundary and the CLI's kernel path produce identical
//! finding encodings on the fixture corpus.
//!
//! Responsibilities: for every fixture document (the parity corpus in
//! `fixtures/parity/` and the CLI's own fixtures), drive the kernel path
//! exactly as `catherine-cli` does (parse → build → `checks::check`) and
//! the boundary's plain `check` function, then assert the findings JSON
//! encodings are identical — no reshaping at the boundary. Parse-error
//! fixtures assert the boundary's typed `Parse` error carries the
//! kernel's own line/column.
//!
//! Rationale: parity is structural (one execution path, two front
//! doors), but the test pins it anyway: the boundary must never grow a
//! second serialization. Encodings, never snapshots — comparison is
//! between the two live paths, not against stored bytes. The tests call
//! the plain Rust boundary functions (`boundary::check`), never the
//! `#[wasm_bindgen]` wrappers — those are extern stubs that panic on
//! non-wasm targets; the plain functions are the audited surface.

use catherine_core::checks;
use catherine_core::error::ParseError;
use catherine_core::instance::TypedInstance;
use catherine_core::parse;
use catherine_wasm::boundary;
use std::fs;
use std::path::{Path, PathBuf};

/// The workspace root (the corpus lives outside this crate's dir; tests
/// run with the crate dir as cwd — the repo convention anchors at
/// `CARGO_MANIFEST_DIR` and walks up).
const WORKSPACE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Every fixture document the parity gate covers.
fn fixture_documents() -> Vec<PathBuf> {
    let mut docs = Vec::new();
    let parity = Path::new(WORKSPACE_ROOT).join("fixtures/parity");
    for dir in fs::read_dir(&parity).expect("parity corpus present") {
        let path = dir.expect("corpus entry").path().join("model.ct.json");
        docs.push(path);
    }
    let cli = fs::read_dir(Path::new(WORKSPACE_ROOT).join("crates/catherine-cli/tests/fixtures"))
        .expect("cli fixtures present");
    for file in cli {
        docs.push(file.expect("fixture entry").path());
    }
    docs.sort();
    docs
}

/// The kernel path's findings encoding — what the CLI forwards in
/// `data.findings`, serialized from the kernel types directly.
fn kernel_findings(document: &str) -> Result<serde_json::Value, ParseError> {
    let model = parse::parse(document)?;
    let typed = TypedInstance::build(&model.schema, &model.instance);
    let findings = checks::check(&model.schema, &typed);
    serde_json::to_value(&findings).map_err(|_| unreachable!())
}

#[test]
fn wasm_and_kernel_paths_agree_on_every_fixture() {
    for path in fixture_documents() {
        let document = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("fixture {} readable: {error}", path.display()));
        match kernel_findings(&document) {
            Ok(expected) => {
                let report = boundary::check(&document).unwrap_or_else(|error| {
                    panic!("boundary check failed on {}: {error:?}", path.display())
                });
                let value: serde_json::Value =
                    serde_json::from_str(&report).expect("boundary report is JSON");
                assert_eq!(
                    value["findings"],
                    expected,
                    "findings encoding diverges at {}",
                    path.display()
                );
            }
            Err(kernel_error) => {
                // Parse-error fixtures: the boundary must fail closed
                // with the kernel's own position — never findings. The
                // message text rides along; the position is the pinned
                // contract.
                let (line, column) = match &kernel_error {
                    ParseError::Syntax { line, column, .. } => (*line, *column),
                    ParseError::Semantic(e) => (e.line, e.column),
                };
                let error = boundary::check(&document)
                    .expect_err("parse-error fixture must not produce findings");
                let boundary::BoundaryError::Parse {
                    line: got_line,
                    column: got_column,
                    ..
                } = error
                else {
                    panic!("expected parse error at {}, got {error:?}", path.display())
                };
                assert_eq!(
                    (got_line, got_column),
                    (line, column),
                    "boundary parse position diverges at {}",
                    path.display()
                );
            }
        }
    }
}
