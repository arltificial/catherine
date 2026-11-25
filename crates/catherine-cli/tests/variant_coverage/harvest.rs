//! The variant-coverage harvest machinery: every contract inventory is
//! harvested from REAL executed paths — the `catherine` binary driven
//! exactly as the other suites drive it over the fixture corpus — plus
//! the two producer-less variants constructed directly by adjudication
//! (constraint citations in the parent module docs). Split out of
//! `variant_coverage.rs` to keep both files inside the file-size gate.

use super::{spelling, ContractVariant};
use catherine_core::findings::{DomainQuestion, EpistemicLabel, FindingKind};
use catherine_core::model::ConstraintStatus;
use serde_json::Value;
use std::collections::BTreeSet;
use std::process::Command;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/");
/// The committed migration fixtures (shared with the kernel suites) —
/// plus the gap-suite dropped-equation map, so every
/// `MigrationFindingKind` variant fires across these maps.
const MIGRATION_FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
const MIGRATION_COVERAGE_RUNS: [(&str, &str); 3] = [
    ("fixtures/migration/", "drop_object.ct.json"),
    ("fixtures/migration/", "bogart_rehearsal.ct.json"),
    (
        "crates/catherine-cli/tests/fixtures/gap_suite/",
        "dropped_equation_migration.ct.json",
    ),
];

/// The kind/label spellings harvested from executed runs.
#[derive(Default)]
pub(super) struct Coverage {
    pub(super) kinds: BTreeSet<String>,
    pub(super) labels: BTreeSet<String>,
}

/// One `search` invocation: the document, explicit per-object row
/// bounds, and the candidate budget.
struct SearchRun<'a> {
    document: &'a str,
    bounds: &'a [(&'a str, usize)],
    candidates: usize,
}

/// The six inventories, harvested: the sets the parent module's ratchet
/// pins against the registered variant spellings.
pub(super) struct Harvested {
    pub(super) kinds: BTreeSet<String>,
    pub(super) migration_kinds: BTreeSet<String>,
    pub(super) labels: BTreeSet<String>,
    pub(super) questions: BTreeSet<String>,
    pub(super) outcomes: BTreeSet<String>,
    pub(super) statuses: BTreeSet<String>,
}

/// The full harvest: corpus producers, the granularity producer, the
/// search outcomes, the migration runs, the parser-constructed
/// statuses, and the two direct constructions. The emptiness guard
/// lives here so a silent corpus change cannot make the ratchet vacuous.
pub(super) fn harvested_inventories() -> Harvested {
    let mut coverage = Coverage::default();
    let underconstraint = harvest_corpus_producers(&mut coverage);
    let outcomes = harvest_search(&mut coverage);
    harvest_granularity(&mut coverage.kinds);

    let mut questions: BTreeSet<String> = BTreeSet::new();
    harvest_questions(&underconstraint, &mut questions);
    let (uncovered, unknown_label) = direct_constructions();
    questions.insert(spelling(ContractVariant::Question(&uncovered)).to_string());
    coverage
        .labels
        .insert(spelling(ContractVariant::Label(&unknown_label)).to_string());

    let mut migration_kinds: BTreeSet<String> = BTreeSet::new();
    harvest_migration(&mut migration_kinds);

    Harvested {
        kinds: coverage.kinds,
        migration_kinds,
        labels: coverage.labels,
        questions,
        outcomes,
        statuses: statuses_both(),
    }
}

/// Spawn the `catherine` binary; stdout only (findings envelopes live
/// there; error envelopes on stderr are failures the other suites pin).
fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("spawn catherine binary: {error}"));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// One fixture path under the corpus dir.
fn fixture(name: &str) -> String {
    format!("{FIXTURES}{name}")
}

/// The corpus's `.ct.json` fixture paths, sorted for determinism.
fn fixture_files() -> Vec<String> {
    let mut files = Vec::new();
    let entries =
        std::fs::read_dir(FIXTURES).unwrap_or_else(|error| panic!("fixture dir readable: {error}"));
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
                    .unwrap_or_else(|error| {
                        panic!("a corpus finding deserializes into the contract Finding: {error}")
                    })
            })
            .collect(),
    )
}

/// Harvest kind/label spellings from one envelope's `data.findings`
/// (non-envelope output — parse errors, empty runs — harvests nothing).
fn harvest(output: &str, coverage: &mut Coverage) {
    let Some(findings) = findings_of(output) else {
        return;
    };
    for finding in findings {
        coverage
            .kinds
            .insert(spelling(ContractVariant::Kind(&finding.kind)).to_string());
        coverage
            .labels
            .insert(spelling(ContractVariant::Label(&finding.label)).to_string());
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
            questions.insert(spelling(ContractVariant::Question(&witness.case)).to_string());
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
    for (prefix, name) in MIGRATION_COVERAGE_RUNS {
        let output = run(&[
            "migrate",
            &format!("{MIGRATION_FIXTURES}{prefix}{name}"),
            "--json",
        ]);
        let value = serde_json::from_str::<Value>(&output)
            .unwrap_or_else(|error| panic!("migrate envelope parses: {error}"));
        let lost = value["data"]["lost"]
            .as_array()
            .unwrap_or_else(|| panic!("data.lost array"));
        assert!(
            !lost.is_empty(),
            "`migrate {name}` produced no lost findings — coverage would be vacuous"
        );
        for finding in lost {
            let finding: catherine_core::migrate::MigrationFinding =
                serde_json::from_value(finding.clone()).unwrap_or_else(|error| {
                    panic!(
                        "a lost finding deserializes into the contract MigrationFinding: {error}"
                    )
                });
            kinds.insert(spelling(ContractVariant::MigrationKind(&finding.kind)).to_string());
        }
    }
}

/// One `search` invocation with explicit per-object row bounds.
fn search(config: &SearchRun) -> String {
    let mut args: Vec<String> = vec!["search".into(), config.document.into(), "--json".into()];
    for (object, rows) in config.bounds {
        args.push("--rows".into());
        args.push(format!("{object}={rows}"));
    }
    args.push("--max-candidates".into());
    args.push(config.candidates.to_string());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&refs)
}

/// The kernel walk over every corpus fixture plus the two analysis
/// producers — the kinds/labels the executed corpus constructs.
fn harvest_corpus_producers(coverage: &mut Coverage) -> String {
    for file in fixture_files() {
        harvest(&run(&["check", &file, "--json"]), coverage);
    }
    let underconstraint = run(&[
        "underconstraint",
        &fixture("analysis_underconstraint.ct.json"),
        "--json",
    ]);
    harvest(&underconstraint, coverage);
    harvest(
        &run(&[
            "redundant",
            &fixture("analysis_redundant.ct.json"),
            "--json",
        ]),
        coverage,
    );
    underconstraint
}

/// The three search bound-configurations: the outcomes set, and the
/// exhaustion run's contradiction finding harvested alongside.
fn harvest_search(coverage: &mut Coverage) -> BTreeSet<String> {
    let contradiction = search(&SearchRun {
        document: &fixture("analysis_contradiction.ct.json"),
        bounds: &[("a", 1), ("b", 0), ("c", 1)],
        candidates: 64,
    });
    let unknown = search(&SearchRun {
        document: &fixture("analysis_contradiction.ct.json"),
        bounds: &[("a", 1), ("b", 0), ("c", 1)],
        candidates: 2,
    });
    let witness = search(&SearchRun {
        document: &fixture("bowling-fixed.ct.json"),
        bounds: &[
            ("Assignment", 1),
            ("Equipment", 1),
            ("Staff", 1),
            ("Unit", 1),
        ],
        candidates: 64,
    });
    harvest(&contradiction, coverage);
    [
        outcome_of(&contradiction),
        outcome_of(&unknown),
        outcome_of(&witness),
    ]
    .into_iter()
    .collect()
}

/// The two producer-less variants, constructed directly with the
/// constraint citations living in the parent module docs.
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
    let bytes =
        std::fs::read_to_string(file).unwrap_or_else(|error| panic!("fixture readable: {error}"));
    let model = catherine_core::parse::parse(&bytes)
        .unwrap_or_else(|error| panic!("fixture parses: {error}"));
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
            statuses.insert(spelling(ContractVariant::Status(&status)).to_string());
        }
    }
    statuses
}

/// The granularity kind's producer run (ticket `catherine-kw9.3`): the
/// lint fires over the committed over-cap fixture, which lives in the
/// repo fixture corpus rather than `tests/fixtures/`, so it is driven
/// here the same way the migration runs drive their repo fixtures.
fn harvest_granularity(kinds: &mut BTreeSet<String>) {
    let output = run(&[
        "check",
        &format!("{MIGRATION_FIXTURES}fixtures/granularity/over_cap.ct.json"),
        "--json",
    ]);
    let harvested = findings_of(&output)
        .unwrap_or_else(|| panic!("the over-cap run yields a findings envelope"));
    assert!(
        !harvested.is_empty(),
        "the over-cap run produced no findings — granularity coverage would be vacuous"
    );
    for finding in harvested {
        kinds.insert(spelling(ContractVariant::Kind(&finding.kind)).to_string());
    }
}
