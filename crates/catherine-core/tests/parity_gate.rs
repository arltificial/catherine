//! Purpose: the named parity gate (openspec change
//! `add-executed-checks`, task 9; E8) — the vendored `fixtures/parity/`
//! set, mirrored from specodelic's corpus discipline
//! (`tests/acset_parity.rs` + `tests/fixtures/{small,zero_file,
//! single_intent,dangling,typing_allowed,typing_violations}`), asserts
//! that the ported instance tables and checks produce specodelic's acset
//! behavior on every shared case: typing allowed/violated, dangling,
//! clean, and the degenerate empty boundaries.
//!
//! Responsibilities: for each fixture directory, load `model.ct.json`,
//! route it (parse for documents P0 accepts; the lenient presentation
//! loader for the typing-violation document whose wrong-object value P0's
//! parser gates structurally), build the typed instance, and compare
//! build reports, morphism tables, and check outcomes against the
//! vendored expected set — structurally, by kind and witness, never
//! snapshots. The expected sets ARE the parity contract.
//!
//! Rationale: specodelic's acset has no path equations, so the gate
//! covers the shared acset behavior — instance tables, typing, dangling —
//! and not equation semantics (the E10 new code carries its own property
//! tests). Donor behavior ground truth: specodelic's
//! `tests/snapshots/{dangling,typing_allowed,typing_violations}.txt`
//! oracles and its green `acset_parity` suite on the mirrored corpora.

mod common;

use common::load_presentation;

use catherine_core::checks::{check, CheckOutcome};
use catherine_core::instance::{TypedInstance, TypingViolation};
use catherine_core::model::{Ident, RowId};
use catherine_core::parse::parse;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PARITY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/parity");

#[test]
fn parity_fixtures_match_the_vendored_expected_sets() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(PARITY_DIR)
        .expect("fixtures/parity exists")
        .map(|e| e.expect("fixture dir entry").path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    assert!(
        !dirs.is_empty(),
        "the vendored parity set must not be empty"
    );
    for dir in &dirs {
        assert_fixture(dir);
    }
}

fn assert_fixture(dir: &Path) {
    let name = dir
        .file_name()
        .expect("fixture dir has a name")
        .to_string_lossy()
        .to_string();
    let model_text =
        std::fs::read_to_string(dir.join("model.ct.json")).expect("model.ct.json exists");
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("expected.json")).expect("expected.json exists"),
    )
    .expect("expected.json is valid JSON");

    let (schema, instance) = match expected["route"].as_str() {
        Some("parse") => {
            let model =
                parse(&model_text).unwrap_or_else(|e| panic!("[{name}] fixture must parse: {e:?}"));
            (model.schema, model.instance)
        }
        Some("presentation") => {
            assert!(
                parse(&model_text).is_err(),
                "[{name}] P0's parser must gate this wrong-object value structurally"
            );
            load_presentation(&model_text).unwrap_or_else(|e| panic!("[{name}] loader failed: {e}"))
        }
        other => panic!("[{name}] unknown route {other:?}"),
    };

    let built = TypedInstance::build(&schema, &instance);

    let expected_typing: Vec<TypingViolation> = expected["typing_violations"]
        .as_array()
        .unwrap_or_else(|| panic!("[{name}] missing typing_violations"))
        .iter()
        .map(|v| TypingViolation {
            arrow: Ident(v["arrow"].as_str().expect("arrow").to_string()),
            src: RowId(v["src"].as_u64().expect("src") as usize),
            value: RowId(v["value"].as_u64().expect("value") as usize),
        })
        .collect();
    assert_eq!(
        built.typing_violations(),
        expected_typing.as_slice(),
        "[{name}] typing violations"
    );

    let expected_dangling: Vec<catherine_core::instance::DanglingCell> = expected["dangling"]
        .as_array()
        .unwrap_or_else(|| panic!("[{name}] missing dangling"))
        .iter()
        .map(|v| catherine_core::instance::DanglingCell {
            arrow: Ident(v["arrow"].as_str().expect("arrow").to_string()),
            src: RowId(v["src"].as_u64().expect("src") as usize),
        })
        .collect();
    assert_eq!(
        built.dangling(),
        expected_dangling.as_slice(),
        "[{name}] dangling cells"
    );

    let expected_tables: BTreeMap<Ident, Vec<Option<RowId>>> = expected["tables"]
        .as_object()
        .unwrap_or_else(|| panic!("[{name}] missing tables"))
        .iter()
        .map(|(arrow, cells)| {
            (
                Ident(arrow.clone()),
                cells
                    .as_array()
                    .expect("cells array")
                    .iter()
                    .map(|c| match c {
                        Value::Null => None,
                        v => Some(RowId(v.as_u64().expect("row id") as usize)),
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(built.tables(), &expected_tables, "[{name}] morphism tables");

    let expected_outcomes: Vec<CheckOutcome> = expected["outcomes"]
        .as_array()
        .unwrap_or_else(|| panic!("[{name}] missing outcomes"))
        .iter()
        .map(|o| match o["kind"].as_str() {
            Some("dangling") => CheckOutcome::Dangling {
                arrow: Ident(o["arrow"].as_str().expect("arrow").to_string()),
                src: RowId(o["src"].as_u64().expect("src") as usize),
            },
            Some("violation") => CheckOutcome::Violation {
                equation: o["equation"].as_u64().expect("equation") as usize,
                row: RowId(o["row"].as_u64().expect("row") as usize),
                lhs: RowId(o["lhs"].as_u64().expect("lhs") as usize),
                rhs: RowId(o["rhs"].as_u64().expect("rhs") as usize),
            },
            Some("hypothesis") => CheckOutcome::Hypothesis {
                equation: o["equation"].as_u64().expect("equation") as usize,
                row: RowId(o["row"].as_u64().expect("row") as usize),
                lhs: RowId(o["lhs"].as_u64().expect("lhs") as usize),
                rhs: RowId(o["rhs"].as_u64().expect("rhs") as usize),
            },
            other => panic!("[{name}] unknown outcome kind {other:?}"),
        })
        .collect();
    let outcomes = check(&schema, &built);
    assert_eq!(outcomes, expected_outcomes, "[{name}] check outcomes");
}
