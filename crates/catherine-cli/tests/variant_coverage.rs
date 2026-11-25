//! Purpose: the PREVENTION-C variant-coverage ratchet (beads
//! `catherine-3zs`) — every variant of the contract enums is constructed
//! by at least one executed fixture run, so a variant with zero fixture
//! coverage (the u3k class: `ConstraintStatus::Proposed` shipped
//! unchecked) cannot recur. The variant inventories are pinned by an
//! exhaustive `match` spelling dispatch with NO wildcard arms: a new
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
//! variant existing without pretending a producer exists. The harvest
//! machinery (binary probes, deserialization guards) lives in the
//! `harvest` submodule.

#[path = "variant_coverage/harvest.rs"]
mod harvest;

use catherine_core::findings::{DomainQuestion, EpistemicLabel, FindingKind};
use catherine_core::migrate::MigrationFindingKind;
use catherine_core::model::ConstraintStatus;
use harvest::Harvested;
use std::collections::BTreeSet;

const ALL_KINDS: [&str; 6] = [
    "violation",
    "dangling",
    "redundant",
    "contradiction",
    "underconstraint",
    "granularity",
];
const ALL_MIGRATION_KINDS: [&str; 3] = ["lost_row", "lost_cell", "weakened"];
const ALL_LABELS: [&str; 3] = ["executed", "hypothesized", "unknown"];
const ALL_QUESTIONS: [&str; 2] = ["parallel_paths", "uncovered_inputs"];
const ALL_OUTCOMES: [&str; 3] = ["witness", "contradiction", "unknown"];
const ALL_STATUSES: [&str; 2] = ["proposed", "accepted"];

/// Any contract value the ratchet spells. Each inventory's match is
/// exhaustive with no wildcard arms — the compile-time ratchet: a new
/// enum variant fails this dispatch until its spelling table grows
/// (the migration taxonomy's table is task 7's surface).
enum ContractVariant<'a> {
    /// The check findings taxonomy (six spellings).
    Kind(&'a FindingKind),
    /// The migration taxonomy (three spellings).
    MigrationKind(&'a MigrationFindingKind),
    /// The finding labels (three spellings).
    Label(&'a EpistemicLabel),
    /// The domain questions (two spellings).
    Question(&'a DomainQuestion),
    /// The parser-constructed constraint statuses (two spellings).
    Status(&'a ConstraintStatus),
}

/// The inventories' variant-to-spelling dispatch.
fn spelling(variant: ContractVariant) -> &'static str {
    match variant {
        ContractVariant::Kind(kind) => match kind {
            FindingKind::Violation(_) => "violation",
            FindingKind::Dangling(_) => "dangling",
            FindingKind::Redundant(_) => "redundant",
            FindingKind::Contradiction(_) => "contradiction",
            FindingKind::Underconstraint(_) => "underconstraint",
            FindingKind::Granularity(_) => "granularity",
        },
        ContractVariant::MigrationKind(kind) => match kind {
            MigrationFindingKind::LostRow { .. } => "lost_row",
            MigrationFindingKind::LostCell { .. } => "lost_cell",
            MigrationFindingKind::Weakened { .. } => "weakened",
        },
        ContractVariant::Label(label) => match label {
            EpistemicLabel::Executed => "executed",
            EpistemicLabel::Hypothesized => "hypothesized",
            EpistemicLabel::Unknown => "unknown",
        },
        ContractVariant::Question(question) => match question {
            DomainQuestion::ParallelPaths { .. } => "parallel_paths",
            DomainQuestion::UncoveredInputs { .. } => "uncovered_inputs",
        },
        ContractVariant::Status(status) => match status {
            ConstraintStatus::Proposed => "proposed",
            ConstraintStatus::Accepted => "accepted",
        },
    }
}

/// One inventory pin: the enum's name, its variant spellings, and the
/// harvested set that must contain every one of them.
struct Ratchet<'a> {
    enum_name: &'a str,
    inventory: &'a [&'a str],
    got: &'a BTreeSet<String>,
}

/// Every inventory entry must appear in the harvested set — the failure
/// names the uncovered variant.
fn assert_all_covered(ratchet: &Ratchet) {
    for name in ratchet.inventory {
        assert!(
            ratchet.got.contains(*name),
            "{} variant `{}` has no executed fixture coverage",
            ratchet.enum_name,
            name
        );
    }
}

/// Pin all six inventories: each harvested set must contain every
/// registered variant spelling.
fn assert_inventories_covered(harvested: &Harvested) {
    let ratchets = [
        Ratchet {
            enum_name: "FindingKind",
            inventory: &ALL_KINDS,
            got: &harvested.kinds,
        },
        Ratchet {
            enum_name: "MigrationFindingKind",
            inventory: &ALL_MIGRATION_KINDS,
            got: &harvested.migration_kinds,
        },
        Ratchet {
            enum_name: "EpistemicLabel",
            inventory: &ALL_LABELS,
            got: &harvested.labels,
        },
        Ratchet {
            enum_name: "DomainQuestion",
            inventory: &ALL_QUESTIONS,
            got: &harvested.questions,
        },
        Ratchet {
            enum_name: "SearchOutcome",
            inventory: &ALL_OUTCOMES,
            got: &harvested.outcomes,
        },
        Ratchet {
            enum_name: "ConstraintStatus",
            inventory: &ALL_STATUSES,
            got: &harvested.statuses,
        },
    ];
    for ratchet in &ratchets {
        assert_all_covered(ratchet);
    }
}

#[test]
fn every_contract_enum_variant_is_constructed_by_an_executed_fixture() {
    let harvested = harvest::harvested_inventories();
    assert!(
        !harvested.kinds.is_empty(),
        "the corpus run produced no findings — coverage would be vacuous"
    );
    assert_inventories_covered(&harvested);
}
