//! Purpose: the PREVENTION-C variant-coverage ratchet (beads
//! `catherine-3zs`) — every variant of the contract enums is constructed
//! by at least one executed fixture run, so a variant with zero fixture
//! coverage (the u3k class: `ConstraintStatus::Proposed` shipped
//! unchecked) cannot recur. The variant inventories are pinned by
//! exhaustive `match` name functions with NO wildcard arms: a new
//! variant fails compilation here until this ratchet accounts for it.
//!
//! Rationale: coverage is measured over the REAL executed paths — the
//! `catherine` binary driven exactly as the other suites drive it over
//! the fixture corpus — not over bare enum lists. Two variants are
//! covered by direct construction, by adjudication, with the constraint
//! cited inline: `DomainQuestion::UncoveredInputs` is P8-reserved with
//! no producer anywhere in v1 (hard constraint, see
//! `catherine-analysis/src/parallel.rs` module docs), and
//! `EpistemicLabel::Unknown` is render-layer-reserved with no producer
//! op (see `catherine-core/src/findings.rs` — "no producer exists in
//! this crate"). Direct construction keeps the ratchet honest about the
//! variant existing without pretending a producer exists.

use catherine_core::findings::{DomainQuestion, EpistemicLabel, FindingKind};
use catherine_core::migrate::MigrationFindingKind;
use catherine_core::model::ConstraintStatus;
use serde_json::Value;
use std::collections::BTreeSet;
use std::process::Command;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/");
/// The committed migration fixtures (shared with the kernel suites) —
/// both `MigrationFindingKind` variants fire across these two maps.
const MIGRATION_FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/migration/");
const MIGRATION_COVERAGE_RUNS: [&str; 2] = ["drop_object.ct.json", "bogart_rehearsal.ct.json"];
const ALL_KINDS: [&str; 5] = [
    "violation",
    "dangling",
    "redundant",
    "contradiction",
    "underconstraint",
];
const ALL_MIGRATION_KINDS: [&str; 2] = ["lost_row", "lost_cell"];
const ALL_LABELS: [&str; 3] = ["executed", "hypothesized", "unknown"];
const ALL_QUESTIONS: [&str; 2] = ["parallel_paths", "uncovered_inputs"];
const ALL_OUTCOMES: [&str; 3] = ["witness", "contradiction", "unknown"];
const ALL_STATUSES: [&str; 2] = ["proposed", "accepted"];

/// Exhaustive variant-to-spelling maps: the compile-time ratchet (a new
/// enum variant breaks these matches until the inventories grow).
fn label_name(label: &EpistemicLabel) -> &'static str {
    match label {
        EpistemicLabel::Executed => "executed",
        EpistemicLabel::Hypothesized => "hypothesized",
        EpistemicLabel::Unknown => "unknown",
    }
}

fn kind_name(kind: &FindingKind) -> &'static str {
    match kind {
        FindingKind::Violation(_) => "violation",
        FindingKind::Dangling(_) => "dangling",
        FindingKind::Redundant(_) => "redundant",
        FindingKind::Contradiction(_) => "contradiction",
        FindingKind::Underconstraint(_) => "underconstraint",
    }
}

/// The migration taxonomy's compile-time ratchet (task 7's surface):
/// a new `MigrationFindingKind` variant breaks this match until the
/// inventory grows.
fn migration_kind_name(kind: &MigrationFindingKind) -> &'static str {
    match kind {
        MigrationFindingKind::LostRow { .. } => "lost_row",
        MigrationFindingKind::LostCell { .. } => "lost_cell",
    }
}

fn question_name(question: &DomainQuestion) -> &'static str {
    match question {
        DomainQuestion::ParallelPaths { .. } => "parallel_paths",
        DomainQuestion::UncoveredInputs { .. } => "uncovered_inputs",
    }
}

fn status_name(status: &ConstraintStatus) -> &'static str {
    match status {
        ConstraintStatus::Proposed => "proposed",
        ConstraintStatus::Accepted => "accepted",
    }
}

/// Spawn the `catherine` binary; stdout only (findings envelopes live
/// there; error envelopes on stderr are failures the other suites pin).
fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
        .expect("spawn catherine binary");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// One fixture path under the corpus dir.
fn fixture(name: &str) -> String {
    format!("{FIXTURES}{name}")
}

/// The corpus's `.ct.json` fixture paths, sorted for determinism.
fn fixture_files() -> Vec<String> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(FIXTURES).expect("fixture dir readable");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    files.sort();
    files
}

/// The envelope's `data.findings`, each deserialized into the contract
/// `Finding` type — a corpus finding that does not deserialize is
/// contract drift and fails loudly (that is the ratchet, not a
/// convenience).
fn findings_of(output: &str) -> Option<Vec<catherine_core::findings::Finding>> {
    let value = serde_json::from_str::<Value>(output).ok()?;
    let raw = value["data"]["findings"].as_array()?;
    Some(
        raw.iter()
            .map(|finding| {
                serde_json::from_value::<catherine_core::findings::Finding>(finding.clone())
                    .expect("a corpus finding deserializes into the contract Finding")
            })
            .collect(),
    )
}

/// Harvest kind/label spellings from one envelope's `data.findings`
/// (non-envelope output — parse errors, empty runs — harvests nothing).
fn harvest(output: &str, kinds: &mut BTreeSet<String>, labels: &mut BTreeSet<String>) {
    let Some(findings) = findings_of(output) else {
        return;
    };
    for finding in findings {
        kinds.insert(kind_name(&finding.kind).to_string());
        labels.insert(label_name(&finding.label).to_string());
    }
}

/// Harvest the domain-question spellings from underconstraint findings
/// in one envelope.
fn harvest_questions(output: &str, questions: &mut BTreeSet<String>) {
    let Some(findings) = findings_of(output) else {
        return;
    };
    for finding in findings {
        if let FindingKind::Underconstraint(witness) = finding.kind {
            questions.insert(question_name(&witness.case).to_string());
        }
    }
}

/// The `data.outcome` spelling of one search envelope (empty string when
/// the envelope is not a findings envelope).
fn outcome_of(output: &str) -> String {
    serde_json::from_str::<Value>(output)
        .ok()
        .and_then(|value| value["data"]["outcome"].as_str().map(String::from))
        .unwrap_or_default()
}

/// The `migrate` envelope's `data.lost` over the committed migration
/// fixtures, each finding deserialized into the contract
/// `MigrationFinding` — a lost finding that does not deserialize is
/// contract drift and fails loudly (the same ratchet rule as `Finding`).
fn harvest_migration(kinds: &mut BTreeSet<String>) {
    for name in MIGRATION_COVERAGE_RUNS {
        let output = run(&["migrate", &format!("{MIGRATION_FIXTURES}{name}"), "--json"]);
        let value = serde_json::from_str::<Value>(&output).expect("migrate envelope parses");
        let lost = value["data"]["lost"].as_array().expect("data.lost array");
        assert!(
            !lost.is_empty(),
            "`migrate {name}` produced no lost findings — coverage would be vacuous"
        );
        for finding in lost {
            let finding: catherine_core::migrate::MigrationFinding =
                serde_json::from_value(finding.clone())
                    .expect("a lost finding deserializes into the contract MigrationFinding");
            kinds.insert(migration_kind_name(&finding.kind).to_string());
        }
    }
}

/// One `search` invocation with explicit per-object row bounds.
fn search(document: &str, bounds: &[(&str, usize)], candidates: usize) -> String {
    let mut args: Vec<String> = vec!["search".into(), document.into(), "--json".into()];
    for (object, rows) in bounds {
        args.push("--rows".into());
        args.push(format!("{object}={rows}"));
    }
    args.push("--max-candidates".into());
    args.push(candidates.to_string());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&refs)
}

/// The kernel walk over every corpus fixture plus the two analysis
/// producers — the kinds/labels the executed corpus constructs.
fn harvest_corpus_producers(kinds: &mut BTreeSet<String>, labels: &mut BTreeSet<String>) -> String {
    for file in fixture_files() {
        harvest(&run(&["check", &file, "--json"]), kinds, labels);
    }
    let underconstraint = run(&[
        "underconstraint",
        &fixture("analysis_underconstraint.ct.json"),
        "--json",
    ]);
    harvest(&underconstraint, kinds, labels);
    harvest(
        &run(&[
            "redundant",
            &fixture("analysis_redundant.ct.json"),
            "--json",
        ]),
        kinds,
        labels,
    );
    underconstraint
}

/// The three search bound-configurations: the outcomes set, and the
/// exhaustion run's contradiction finding harvested alongside.
fn harvest_search(kinds: &mut BTreeSet<String>, labels: &mut BTreeSet<String>) -> BTreeSet<String> {
    let contradiction = search(
        &fixture("analysis_contradiction.ct.json"),
        &[("a", 1), ("b", 0), ("c", 1)],
        64,
    );
    let unknown = search(
        &fixture("analysis_contradiction.ct.json"),
        &[("a", 1), ("b", 0), ("c", 1)],
        2,
    );
    let witness = search(
        &fixture("bowling-fixed.ct.json"),
        &[
            ("Assignment", 1),
            ("Equipment", 1),
            ("Staff", 1),
            ("Unit", 1),
        ],
        64,
    );
    harvest(&contradiction, kinds, labels);
    [
        outcome_of(&contradiction),
        outcome_of(&unknown),
        outcome_of(&witness),
    ]
    .into_iter()
    .collect()
}

/// The two producer-less variants, constructed directly with the
/// constraint citations living in the module docs.
fn direct_constructions() -> (DomainQuestion, EpistemicLabel) {
    let uncovered = DomainQuestion::UncoveredInputs {
        class: "no-producer-in-v1".to_string(),
        examples: vec![],
    };
    (uncovered, EpistemicLabel::Unknown)
}

/// The parser-constructed statuses in a parsed fixture document — the
/// parser constructs the enum, exactly the u3k coverage that was
/// missing.
fn statuses_of(file: &str) -> Vec<ConstraintStatus> {
    let bytes = std::fs::read_to_string(file).expect("fixture readable");
    let model = catherine_core::parse::parse(&bytes).expect("fixture parses");
    model
        .schema
        .equations
        .iter()
        .map(|equation| equation.status)
        .collect()
}

/// The parser-constructed statuses over the two dedicated fixtures.
fn statuses_both() -> BTreeSet<String> {
    let mut statuses = BTreeSet::new();
    for file in ["proposed_equation.ct.json", "bowling-fixed.ct.json"] {
        for status in statuses_of(&fixture(file)) {
            statuses.insert(status_name(&status).to_string());
        }
    }
    statuses
}

/// Every inventory entry must appear in the harvested set — the failure
/// names the uncovered variant.
fn assert_all_covered(enum_name: &str, inventory: &[&str], got: &BTreeSet<String>) {
    for name in inventory {
        assert!(
            got.contains(*name),
            "{enum_name} variant `{name}` has no executed fixture coverage"
        );
    }
}

#[test]
fn every_contract_enum_variant_is_constructed_by_an_executed_fixture() {
    let mut kinds: BTreeSet<String> = BTreeSet::new();
    let mut labels: BTreeSet<String> = BTreeSet::new();
    let underconstraint = harvest_corpus_producers(&mut kinds, &mut labels);
    let outcomes = harvest_search(&mut kinds, &mut labels);

    let mut questions: BTreeSet<String> = BTreeSet::new();
    harvest_questions(&underconstraint, &mut questions);
    let (uncovered, unknown_label) = direct_constructions();
    questions.insert(question_name(&uncovered).to_string());
    labels.insert(label_name(&unknown_label).to_string());

    let mut migration_kinds: BTreeSet<String> = BTreeSet::new();
    harvest_migration(&mut migration_kinds);

    assert!(
        !kinds.is_empty(),
        "the corpus run produced no findings — coverage would be vacuous"
    );

    assert_all_covered("FindingKind", &ALL_KINDS, &kinds);
    assert_all_covered(
        "MigrationFindingKind",
        &ALL_MIGRATION_KINDS,
        &migration_kinds,
    );
    assert_all_covered("EpistemicLabel", &ALL_LABELS, &labels);
    assert_all_covered("DomainQuestion", &ALL_QUESTIONS, &questions);
    assert_all_covered("SearchOutcome", &ALL_OUTCOMES, &outcomes);
    assert_all_covered("ConstraintStatus", &ALL_STATUSES, &statuses_both());
}
