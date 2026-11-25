//! Purpose: the named parity gate (openspec change
//! `add-executed-checks`, task 9; E8) — the vendored `fixtures/parity/`
//! set, which now holds the specodelic corpus mirrors
//! (`tests/acset_parity.rs` + `tests/fixtures/{small,zero_file,
//! single_intent,dangling,typing_allowed,typing_violations}`) plus one
//! catherine-authored `pullback_square` entry for the lean-kernel oracle.
//! It asserts that the ported instance tables and checks produce
//! specodelic's acset behavior on every shared case — typing
//! allowed/violated, dangling, clean, and the degenerate empty
//! boundaries — and the expected pullback witness on the added entry.
//!
//! Responsibilities: for each fixture directory, load `model.ct.json`,
//! route it (parse for documents P0 accepts; the lenient presentation
//! loader for the typing-violation document whose wrong-object value P0's
//! parser gates structurally), build the typed instance, and compare
//! build reports, morphism tables, and check findings against the
//! vendored expected set — structurally, by kind, witness, and label,
//! never snapshots. The expected sets ARE the parity contract; their
//! outcome lists record P1-era facts, re-sorted under the findings
//! taxonomy's total order (the P2 ordering contract) before comparing.
//!
//! Rationale: specodelic's acset has no path equations, so the gate
//! covers the shared acset behavior — instance tables, typing, dangling —
//! and not equation semantics (the E10 new code carries its own property
//! tests). Donor behavior ground truth: specodelic's
//! `tests/snapshots/{dangling,typing_allowed,typing_violations}.txt`
//! oracles and its green `acset_parity` suite on the mirrored corpora;
//! the `pullback_square` entry is catherine-authored (it has no
//! specodelic counterpart) and is sealed by the lean-kernel oracle
//! instead.

mod common;

use common::load_presentation;

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, PullbackWitness,
    ViolationWitness,
};
use catherine_core::instance::{DanglingCell, TypedInstance, TypingViolation};
use catherine_core::model::{Ident, Instance, RowId, Schema};
use catherine_core::parse::parse;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use common::Insist;

const PARITY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/parity");

#[test]
fn parity_fixtures_match_the_vendored_expected_sets() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(PARITY_DIR)
        .insist("fixtures/parity exists")
        .map(|e| e.insist("fixture dir entry").path())
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

/// The fixture directory's display name for assertion messages.
fn fixture_name(dir: &Path) -> String {
    dir.file_name()
        .insist("fixture dir has a name")
        .to_string_lossy()
        .to_string()
}

/// A fixture's display name with its vendored expected set, shared by
/// the load and assertion helpers so each stays within the two-parameter
/// test limit.
struct Fixture<'a> {
    name: &'a str,
    expected: &'a Value,
}

impl Fixture<'_> {
    /// Load the fixture: the model document routed through parse or the
    /// lenient presentation loader as `expected["route"]` demands.
    fn load(&self, model_text: &str) -> (Schema, Instance) {
        let name = self.name;
        let expected = self.expected;
        match expected["route"].as_str() {
            Some("parse") => {
                let model = parse(model_text)
                    .unwrap_or_else(|e| panic!("[{name}] fixture must parse: {e:?}"));
                (model.schema, model.instance)
            }
            Some("presentation") => {
                assert!(
                    parse(model_text).is_err(),
                    "[{name}] P0's parser must gate this wrong-object value structurally"
                );
                load_presentation(model_text)
                    .unwrap_or_else(|e| panic!("[{name}] loader failed: {e}"))
            }
            other => panic!("[{name}] unknown route {other:?}"),
        }
    }
}

fn assert_fixture(dir: &Path) {
    let name = fixture_name(dir);
    let model_text =
        std::fs::read_to_string(dir.join("model.ct.json")).insist("model.ct.json exists");
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("expected.json")).insist("expected.json exists"),
    )
    .insist("expected.json is valid JSON");

    let fixture = Fixture {
        name: &name,
        expected: &expected,
    };
    let (schema, instance) = fixture.load(&model_text);
    let built = TypedInstance::build(&schema, &instance);
    fixture.assert_typing(&built);
    fixture.assert_dangling(&built);
    fixture.assert_tables(&built);
    fixture.assert_outcomes((&schema, &built));
}

/// The build's typing violations match the vendored expected set.
impl Fixture<'_> {
    fn assert_typing(&self, built: &TypedInstance) {
        let name = self.name;
        let expected = self.expected;
        let expected_typing: Vec<TypingViolation> = expected["typing_violations"]
            .as_array()
            .unwrap_or_else(|| panic!("[{name}] missing typing_violations"))
            .iter()
            .map(typed_violation)
            .collect();
        assert_eq!(
            built.typing_violations(),
            expected_typing.as_slice(),
            "[{name}] typing violations"
        );
    }
}

/// One vendored typing violation.
fn typed_violation(v: &Value) -> TypingViolation {
    TypingViolation {
        arrow: Ident(v["arrow"].as_str().insist("arrow").to_string()),
        src: RowId(v["src"].as_u64().insist("src") as usize),
        value: RowId(v["value"].as_u64().insist("value") as usize),
    }
}

/// The build's dangling cells match the vendored expected set.
impl Fixture<'_> {
    fn assert_dangling(&self, built: &TypedInstance) {
        let name = self.name;
        let expected = self.expected;
        let expected_dangling: Vec<DanglingCell> = expected["dangling"]
            .as_array()
            .unwrap_or_else(|| panic!("[{name}] missing dangling"))
            .iter()
            .map(dangling_cell)
            .collect();
        assert_eq!(
            built.dangling(),
            expected_dangling.as_slice(),
            "[{name}] dangling cells"
        );
    }
}

/// One vendored dangling cell.
fn dangling_cell(v: &Value) -> DanglingCell {
    DanglingCell {
        arrow: Ident(v["arrow"].as_str().insist("arrow").to_string()),
        src: RowId(v["src"].as_u64().insist("src") as usize),
    }
}

/// The build's morphism tables match the vendored expected set.
impl Fixture<'_> {
    fn assert_tables(&self, built: &TypedInstance) {
        let name = self.name;
        let expected = self.expected;
        let expected_tables: BTreeMap<Ident, Vec<Option<RowId>>> = expected["tables"]
            .as_object()
            .unwrap_or_else(|| panic!("[{name}] missing tables"))
            .iter()
            .map(|(arrow, cells)| (Ident(arrow.clone()), expected_cells(cells)))
            .collect();
        assert_eq!(built.tables(), &expected_tables, "[{name}] morphism tables");
    }
}

/// One vendored arrow table: `null` cells stay dangling.
fn expected_cells(cells: &Value) -> Vec<Option<RowId>> {
    cells
        .as_array()
        .insist("cells array")
        .iter()
        .map(|c| match c {
            Value::Null => None,
            v => Some(RowId(v.as_u64().insist("row id") as usize)),
        })
        .collect()
}

/// The check findings match the vendored expected set. The vendored
/// expected files record the parity facts (and P1's then-current
/// dangling-first order); the findings taxonomy's total order is the P2
/// contract, so the expected set is sorted the same way `check` sorts
/// before comparing — same facts, P2 order.
impl Fixture<'_> {
    fn assert_outcomes(&self, built: (&Schema, &TypedInstance)) {
        let (schema, built) = built;
        let name = self.name;
        let expected = self.expected;
        let mut expected_findings: Vec<Finding> = expected["outcomes"]
            .as_array()
            .unwrap_or_else(|| panic!("[{name}] missing outcomes"))
            .iter()
            .map(|o| outcome_finding(name, o))
            .collect();
        expected_findings.sort();
        let findings = check(schema, built);
        assert_eq!(findings, expected_findings, "[{name}] check findings");
    }
}

/// One vendored outcome: a dangling finding, or a violation-shaped one
/// under the kind-named label.
fn outcome_finding(name: &str, o: &Value) -> Finding {
    if o["kind"].as_str() == Some("dangling") {
        dangling_outcome(o)
    } else {
        violation_outcome(name, o)
    }
}

/// A vendored violation/hypothesis outcome; anything else is a broken
/// vendored set.
fn violation_outcome(name: &str, o: &Value) -> Finding {
    if !matches!(o["kind"].as_str(), Some("violation") | Some("hypothesis")) {
        panic!("[{name}] unknown outcome kind {:?}", o["kind"].as_str());
    }
    Finding {
        // Kind names the shape; P1's `hypothesis` outcomes carry
        // the `hypothesized` label on the same violation shape.
        label: violation_label(o),
        kind: FindingKind::Violation(if o.get("pullback").is_some() {
            ViolationWitness::Pullback(pullback_witness(name, o))
        } else {
            ViolationWitness::Equation(EquationWitness {
                equation: o["equation"].as_u64().insist("equation") as usize,
                row: RowId(o["row"].as_u64().insist("row") as usize),
                lhs: RowId(o["lhs"].as_u64().insist("lhs") as usize),
                rhs: RowId(o["rhs"].as_u64().insist("rhs") as usize),
            })
        }),
    }
}

/// A vendored pullback witness: the verdict is named by its field set —
/// `pair` covers, `rows` unique, else commutes with `row`/`lhs`/`rhs`.
fn pullback_witness(name: &str, o: &Value) -> PullbackWitness {
    let id = |v: &Value| {
        RowId(
            v.as_u64()
                .unwrap_or_else(|| panic!("[{name}] pullback witness row id")) as usize,
        )
    };
    let pullback = o["pullback"].as_u64().insist("pullback") as usize;
    if let Some(pair) = o.get("pair") {
        PullbackWitness::Covers {
            pullback,
            pair: [id(&pair[0]), id(&pair[1])],
        }
    } else if let Some(rows) = o.get("rows") {
        PullbackWitness::Unique {
            pullback,
            rows: [id(&rows[0]), id(&rows[1])],
        }
    } else {
        PullbackWitness::Commutes {
            pullback,
            row: id(&o["row"]),
            lhs: id(&o["lhs"]),
            rhs: id(&o["rhs"]),
        }
    }
}

/// The outcome's label: P1's `hypothesis` kind carries `hypothesized`.
fn violation_label(o: &Value) -> EpistemicLabel {
    match o["kind"].as_str() {
        Some("hypothesis") => EpistemicLabel::Hypothesized,
        _ => EpistemicLabel::Executed,
    }
}

/// A vendored dangling outcome.
fn dangling_outcome(o: &Value) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Dangling(DanglingWitness {
            arrow: Ident(o["arrow"].as_str().insist("arrow").to_string()),
            src: RowId(o["src"].as_u64().insist("src") as usize),
        }),
    }
}
