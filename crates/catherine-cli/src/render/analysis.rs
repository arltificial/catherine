//! Purpose: the analysis payloads (P4) — view structs and renderers for
//! the search, distinguish, and query outcomes.
//!
//! Responsibilities: map the analysis outcome types into the genesis
//! envelope's `data` (the analysis crate is kernel-pure — genesis-free,
//! serde-free — so the CLI owns every envelope); render the human lines
//! for each outcome.
//!
//! Rationale: one envelope shape — the genesis `Check`/`List` kinds over
//! the same envelope, no second shape; the search outcome string names
//! its epistemic state and is never collapsed into "clean".

use super::render_one;
use super::CLI_VERSION;
use catherine_analysis::bounds::{SearchBounds, SearchScope};
use catherine_analysis::distinguish::{DistinguishOutcome, DistinguishReport, Observation};
use catherine_analysis::query::NodeRef;
use catherine_analysis::search::{SearchOutcome, SearchReport};
use catherine_core::findings::Finding;
use catherine_core::model::{Instance, RowId, Schema};
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;
use std::collections::BTreeMap;

/// The searched bounds as the payload spells them (the bound restated).
#[derive(Serialize)]
pub struct BoundView {
    pub max_rows: BTreeMap<String, usize>,
    pub max_candidates: usize,
}

impl BoundView {
    pub fn of(bound: &SearchBounds) -> Self {
        Self {
            max_rows: bound
                .max_rows
                .iter()
                .map(|(object, &count)| (object.0.clone(), count))
                .collect(),
            max_candidates: bound.max_candidates,
        }
    }
}

/// The exact scope a non-answer explored (the machine-visible half of an
/// UNKNOWN payload: `outcome: "unknown"` + this scope object).
#[derive(Serialize)]
pub struct ScopeView {
    pub rows: BTreeMap<String, usize>,
    pub candidates_explored: usize,
}

impl ScopeView {
    pub fn of(scope: &SearchScope) -> Self {
        Self {
            rows: scope
                .rows
                .iter()
                .map(|(object, &count)| (object.0.clone(), count))
                .collect(),
            candidates_explored: scope.candidates_explored,
        }
    }
}

/// A witness instance under the envelope: the kernel `Instance`'s rows
/// and arrow tables, row ids as bare numbers, dangling as `null`.
#[derive(Serialize)]
pub struct InstanceView {
    pub rows: BTreeMap<String, Vec<usize>>,
    pub values: BTreeMap<String, Vec<Option<usize>>>,
}

impl InstanceView {
    pub fn of(instance: &Instance) -> Self {
        Self {
            rows: instance
                .rows
                .iter()
                .map(|(object, rows)| {
                    (
                        object.0.clone(),
                        rows.iter().map(|row| row.0).collect::<Vec<_>>(),
                    )
                })
                .collect(),
            values: instance
                .values
                .iter()
                .map(|(name, cells)| {
                    (
                        name.0.clone(),
                        cells
                            .iter()
                            .map(|cell| cell.map(|row| row.0))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect(),
        }
    }
}

/// The search payload: the outcome string is the epistemic state —
/// `"witness"`, `"contradiction"` (with the finding under `findings`),
/// or `"unknown"` (with the scope object). A completed bounded run is a
/// successful tool execution; `"unknown"` names what the bounds did not
/// establish and is never collapsed into `"clean"`.
#[derive(Serialize)]
pub struct SearchData<'a> {
    pub outcome: &'a str,
    pub bound: BoundView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<ScopeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<InstanceView>,
    pub findings: &'a [Finding],
}

/// The one search-envelope path.
pub fn search_envelope(report: &SearchReport, warnings: Vec<Warning>) -> Envelope<SearchData<'_>> {
    let (outcome, scope, witness, findings) = match &report.outcome {
        SearchOutcome::Witness { instance } => {
            ("witness", None, Some(InstanceView::of(instance)), &[][..])
        }
        SearchOutcome::Contradiction { finding } => {
            ("contradiction", None, None, std::slice::from_ref(finding))
        }
        SearchOutcome::Unknown { scope } => ("unknown", Some(ScopeView::of(scope)), None, &[][..]),
    };
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        SearchData {
            outcome,
            bound: BoundView::of(&report.bound),
            scope,
            witness,
            findings,
        },
        warnings,
        Vec::new(),
    )
}

/// The search human lines: reuse the §5 finding renderer for the
/// contradiction; one honest line for the other outcomes (never a
/// naked "no violation found").
pub fn render_search(report: &SearchReport, schema: &Schema) -> Vec<String> {
    match &report.outcome {
        SearchOutcome::Witness { .. } => vec!["search: witness within bounds".to_string()],
        SearchOutcome::Contradiction { finding } => render_one(finding, schema),
        SearchOutcome::Unknown { scope } => vec![format!(
            "search: unknown — explored {} candidate(s) within bounds",
            scope.candidates_explored
        )],
    }
}

/// An observation as the payload spells it.
#[derive(Serialize)]
pub struct ObservationView {
    pub object: String,
    pub row: usize,
    pub path: Vec<String>,
}

impl ObservationView {
    pub fn of(observation: &Observation) -> Self {
        Self {
            object: observation.object.0.clone(),
            row: observation.row.0,
            path: observation.path.0.iter().map(|a| a.0.clone()).collect(),
        }
    }
}

/// The differing observation and both values.
#[derive(Serialize)]
pub struct DifferenceView {
    pub observation: ObservationView,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

/// The distinguish payload: `"distinguishable"` names the differing
/// observation and both values; `"equivalent-in-scope"` echoes the
/// observation set it holds over — no schema-equivalence claim exists.
#[derive(Serialize)]
pub struct DistinguishData {
    pub outcome: &'static str,
    pub observations: Vec<ObservationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difference: Option<DifferenceView>,
}

/// The one distinguish-envelope path.
pub fn distinguish_envelope(
    report: &DistinguishReport,
    warnings: Vec<Warning>,
) -> Envelope<DistinguishData> {
    let (outcome, difference) = match &report.outcome {
        DistinguishOutcome::Distinguishable {
            observation,
            left,
            right,
        } => (
            "distinguishable",
            Some(DifferenceView {
                observation: ObservationView::of(observation),
                left: left.map(|row| row.0),
                right: right.map(|row| row.0),
            }),
        ),
        DistinguishOutcome::EquivalentInScope => ("equivalent-in-scope", None),
    };
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        DistinguishData {
            outcome,
            observations: report
                .observations
                .iter()
                .map(ObservationView::of)
                .collect(),
            difference,
        },
        warnings,
        Vec::new(),
    )
}

/// The distinguish human lines: the differing observation with both
/// values (`Object#n`, or `unresolved`), or the in-scope equivalence.
pub fn render_distinguish(report: &DistinguishReport, schema: &Schema) -> Vec<String> {
    match &report.outcome {
        DistinguishOutcome::Distinguishable {
            observation,
            left,
            right,
        } => {
            let codomain = schema
                .arrows
                .get(observation.path.0.last().unwrap_or(&observation.object))
                .map(|arrow| arrow.tgt.0.as_str());
            vec![
                format!(
                    "distinguishable: {} @ {}#{}",
                    super::path_string(&observation.path),
                    observation.object.0,
                    observation.row.0
                ),
                format!(
                    "  left={} right={}",
                    value_at(codomain, *left),
                    value_at(codomain, *right)
                ),
            ]
        }
        DistinguishOutcome::EquivalentInScope => vec![format!(
            "equivalent-in-scope: {} observation(s) agree",
            report.observations.len()
        )],
    }
}

/// A composed value as the transcript spells it: `Object#n` when the
/// codomain object is known, `unresolved` when the cell holds no row.
fn value_at(object: Option<&str>, value: Option<RowId>) -> String {
    match (object, value) {
        (Some(name), Some(row)) => format!("{name}#{}", row.0),
        (_, Some(row)) => row.0.to_string(),
        _ => "unresolved".to_string(),
    }
}

/// A query node as the payload spells it.
#[derive(Serialize)]
pub struct NodeView {
    pub object: String,
    pub row: usize,
}

/// The query payload: the closure's nodes, sorted (the ported
/// traversal's `BTreeSet` discipline).
#[derive(Serialize)]
pub struct QueryData {
    pub direction: String,
    pub morphisms: Vec<String>,
    pub nodes: Vec<NodeView>,
}

/// The one query-envelope path (the genesis `List` kind: the payload is
/// a node list).
pub fn query_envelope(
    direction: &str,
    morphisms: &[String],
    nodes: &[NodeRef],
    warnings: Vec<Warning>,
) -> Envelope<QueryData> {
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::List,
        QueryData {
            direction: direction.to_string(),
            morphisms: morphisms.to_vec(),
            nodes: nodes
                .iter()
                .map(|node| NodeView {
                    object: node.object.0.clone(),
                    row: node.row.0,
                })
                .collect(),
        },
        warnings,
        Vec::new(),
    )
}

/// The query human lines: a head line and one node per line.
pub fn render_query(direction: &str, nodes: &[NodeRef]) -> Vec<String> {
    let mut lines = vec![format!(
        "query: {} node(s) in {} closure",
        nodes.len(),
        direction
    )];
    lines.extend(
        nodes
            .iter()
            .map(|node| format!("  {}#{}", node.object.0, node.row.0)),
    );
    lines
}
