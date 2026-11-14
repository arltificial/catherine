//! Purpose: the pattern-lens seam (openspec change `add-pattern-lenses`,
//! beads `catherine-mx1.2`) — a static registry of design-pattern
//! analyses, each carrying law-scope metadata and running on the existing
//! analysis paths, so a later ticket can ship one lens per GoF row without
//! touching this module's contract.
//!
//! Responsibilities: the fixed types ([`LawKind`], [`PatternMeta`],
//! [`LensInput`], [`LensOutcome`], [`PatternLens`]), the explicit
//! [`registry`] literal, exact-id [`find`], fail-closed [`run`], and the
//! two helpers every lens shares ([`bind_domain`] for the rule-table
//! lenses' `--values` spelling, [`describe_rule_scope`] for a bounded
//! non-answer). Findings are the existing P2 taxonomy plus the migration
//! engine's own lost-data findings; no lens emits a new kind, and `run` has
//! no error arm for a built model.
//!
//! Rationale: the registry is a compile-time literal, ordered by id and
//! empty of production lenses at the foundation — the orchestrator adds one
//! line per landed lens (`catherine-mx1.3` … `.14`), so a missing lens is
//! the labeled [`LensError::UnknownPattern`] path, never a silently smaller
//! list. The replay harness therefore composes a test-local slice
//! (`tests/pattern_reference.rs`) with `registry()`; the seam stays empty
//! in production while the harness's reference lens proves the contract.
//! A lens binds its declared domain with
//! `RuleDomain::from_presentation` and reuses `rule_lens`; unevaluable
//! input is [`LensOutcome::Unknown`] naming the missing surface, never an
//! empty success. A completed outcome carries two typed channels
//! ([`LensFindings`]): the P2 taxonomy's checks and the migration engine's
//! `lost_row`/`lost_cell` findings, which are
//! `catherine_core::migrate::MigrationFinding`, not `Finding`. The Δ lenses
//! therefore report the migrate surface's own findings instead of a new
//! taxonomy kind, and the replay gate compares each row's recorded
//! multiset across both channels.

use crate::bounds::ScopeCaps;
use catherine_core::findings::Finding;
use catherine_core::migrate::MigrationFinding;
use catherine_core::model::{ColumnType, Ident, Literal, Model, RulePresentation};
use catherine_core::rule_lens::{RuleDomain, RuleScope};
use std::collections::BTreeMap;
use std::fmt;

/// The coarse invariant family a lens checks. One value per lens even where
/// the disposition row names two faces; the metadata's `scope` keeps the
/// row's full wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LawKind {
    /// Monoid/nil identity and identity-preservation equations.
    Identity,
    /// Declared-cell and link totality.
    Totality,
    /// Variant/region-domain coverage.
    Coverage,
    /// Equations that constrain sequence.
    Order,
    /// The save/restore pair's lost data and invertibility.
    Delta,
}

impl LawKind {
    /// The registry's spelling for `--list` and the JSON list payload.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Totality => "totality",
            Self::Coverage => "coverage",
            Self::Order => "order",
            Self::Delta => "delta",
        }
    }
}

/// Documentation-grade lens metadata: the id is the registry key (exact,
/// kebab-case), `scope` is the honesty boundary the kata verdict recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternMeta {
    /// Stable kebab-case id, the bead/ticket name.
    pub id: &'static str,
    /// GoF pattern name.
    pub name: &'static str,
    /// Universal abstraction, disposition-table wording.
    pub abstraction: &'static str,
    /// The registry's coarse law filter.
    pub law: LawKind,
    /// The disposition row's law scope, verbatim.
    pub scope: &'static str,
}

/// Everything a lens reads: the built model and the CLI's `--values`
/// spellings (`column=v1,v2,...`), unparsed because the lens alone knows
/// which surface it needs.
pub struct LensInput<'a> {
    /// The built model the lens runs over.
    pub model: &'a Model,
    /// The CLI's `--values` spellings, verbatim.
    pub values: &'a [String],
}

/// A completed lens run's findings, split by the kernel surface that
/// produced them: the P2 taxonomy's checks and the migration engine's
/// lost-data findings.
///
/// Rationale: the Δ lenses' recorded multisets (`lost_row`, `lost_cell`)
/// are [`MigrationFinding`], not [`Finding`] — the migrate surface's own
/// shape, not another taxonomy kind. Two typed lists keep each channel's
/// serde encoding the kernel's verbatim: the CLI prints `checks` under
/// `data.findings` and `migrations` under `data.migrations`, so no
/// consumer has to guess which taxonomy a flattened list carries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LensFindings {
    /// The P2 taxonomy's findings, from the shared kernel analyses.
    pub checks: Vec<Finding>,
    /// The migration engine's lost-data findings, verbatim.
    pub migrations: Vec<MigrationFinding>,
}

/// A lens's completed result: findings, or the exact scope it could not
/// decide. UNKNOWN is a completed bounded run, never "clean".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LensOutcome {
    /// The lens completed; findings are the existing taxonomy's checks and
    /// the migration engine's lost-data findings.
    Findings(LensFindings),
    /// The requested surface could not be evaluated; names the scope.
    Unknown {
        /// The exact surface or bound that was not decided.
        scope: String,
    },
}

/// One design-pattern analysis. Implementors live in `src/pattern/<id>.rs`
/// (or, for the seam's reference lens, in the test crate) and are reachable
/// only through the registry.
///
/// `Sync` is required because the registry literal is a `static`: lenses
/// are stateless values shared across threads, never per-run state.
pub trait PatternLens: Sync {
    /// This lens's registry metadata.
    fn meta(&self) -> &'static PatternMeta;
    /// Run over a built model under the shared caps.
    fn run(&self, input: &LensInput<'_>, caps: &ScopeCaps) -> LensOutcome;
}

/// The seam's fail-closed refusal: an id absent from the registry is a
/// labeled usage error, never an empty success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LensError {
    /// No lens owns `id`; `known` lists the registry's ids in order.
    UnknownPattern {
        /// The requested id.
        id: String,
        /// The registry's ids, for the remediation hint.
        known: Vec<&'static str>,
    },
}

impl LensError {
    /// The CLI's stable code for this refusal (usage class, exit 2).
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownPattern { .. } => "bad_argument",
        }
    }
}

impl fmt::Display for LensError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPattern { id, known } => {
                write!(formatter, "unknown pattern id `{id}`; known ids: ")?;
                if known.is_empty() {
                    write!(formatter, "(none registered)")
                } else {
                    write!(formatter, "{}", known.join(", "))
                }
            }
        }
    }
}

impl std::error::Error for LensError {}

/// The production registry: one line per landed lens, ordered by id. Empty
/// at the foundation — the orchestrator wires each lens's module here.
static LENSES: &[&'static dyn PatternLens] = &[];

/// The shipped lenses, in registry order (`--list` and the unknown-id
/// diagnostic are byte-stable over this literal).
pub fn registry() -> &'static [&'static dyn PatternLens] {
    LENSES
}

/// The registry's id list, in order — the unknown-id remediation.
pub fn known_ids() -> Vec<&'static str> {
    registry().iter().map(|lens| lens.meta().id).collect()
}

/// Exact, case-sensitive lookup; no aliases, no normalization.
pub fn find(id: &str) -> Option<&'static dyn PatternLens> {
    registry().iter().copied().find(|lens| lens.meta().id == id)
}

/// Run the lens named by `id`; an unregistered id is the labeled refusal.
pub fn run(id: &str, input: &LensInput<'_>, caps: &ScopeCaps) -> Result<LensOutcome, LensError> {
    match find(id) {
        Some(lens) => Ok(lens.run(input, caps)),
        None => Err(LensError::UnknownPattern {
            id: id.to_string(),
            known: known_ids(),
        }),
    }
}

/// Bind a rule-table lens's declared finite domain from the CLI's raw
/// `--values` spellings. A malformed spelling, a column the presentation
/// does not declare, a missing column, or an ill-typed value is the exact
/// scope string a lens reports as [`LensOutcome::Unknown`] — the lens never
/// silently ignores the flag and never invents a value list.
pub fn bind_domain(
    presentation: &RulePresentation,
    values: &[String],
) -> Result<RuleDomain, String> {
    let mut supplied: BTreeMap<Ident, Vec<Literal>> = BTreeMap::new();
    for spec in values {
        let Some((name, list)) = spec.split_once('=') else {
            return Err(format!(
                "`--values {spec}` is not a column=v1,v2,... spelling; nothing evaluated"
            ));
        };
        let column = Ident(name.to_string());
        let Some(column_type) = presentation.columns.get(&column) else {
            return Err(format!(
                "`--values` names `{name}`, which the rule presentation does not declare; nothing evaluated"
            ));
        };
        let parsed = list
            .split(',')
            .map(|token| parse_literal(token, *column_type, name))
            .collect::<Result<Vec<_>, _>>()?;
        supplied.insert(column, parsed);
    }
    RuleDomain::from_presentation(presentation, supplied)
        .map_err(|error| format!("`--values` do not bind the declared domain: {error}"))
}

/// One typed value from a `--values` list; a string stays verbatim.
fn parse_literal(token: &str, column_type: ColumnType, column: &str) -> Result<Literal, String> {
    match column_type {
        ColumnType::Int => token
            .parse::<i64>()
            .map(Literal::Int)
            .map_err(|_| format!("`{token}` is not an int for column `{column}`")),
        ColumnType::Bool => match token {
            "true" => Ok(Literal::Bool(true)),
            "false" => Ok(Literal::Bool(false)),
            _ => Err(format!("`{token}` is not a bool for column `{column}`")),
        },
        ColumnType::String => Ok(Literal::String(token.to_string())),
    }
}

/// The exact scope a rule-domain enumeration stopped at, as the UI's
/// unknown payload spells it: declared column sizes and candidates explored.
pub fn describe_rule_scope(scope: &RuleScope) -> String {
    let columns = scope
        .columns
        .iter()
        .map(|(column, size)| format!("{}={size}", column.0))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "rule-domain enumeration stopped after {} candidate(s) over columns {{{columns}}}; nothing decided",
        scope.candidates_explored
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_is_ordered_and_unique() {
        let ids = known_ids();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids, sorted, "the registry literal is ordered by id");
        assert!(registry().iter().all(|lens| {
            let meta = lens.meta();
            !meta.id.is_empty()
                && !meta.name.is_empty()
                && !meta.abstraction.is_empty()
                && !meta.scope.is_empty()
        }));
    }

    #[test]
    fn an_unregistered_id_fails_closed() {
        let model = Model::empty();
        let input = LensInput {
            model: &model,
            values: &[],
        };
        let error = run("nope", &input, &ScopeCaps::default()).expect_err("not registered");
        assert_eq!(error.code(), "bad_argument");
        assert!(error.to_string().contains("nope"));
        assert!(find("nope").is_none());
    }

    #[test]
    fn a_domain_binds_from_the_cli_spelling() {
        use catherine_core::model::{Policy, Rule};
        let presentation = RulePresentation {
            domain: Ident("Input".to_string()),
            columns: [(Ident("kind".to_string()), ColumnType::String)]
                .into_iter()
                .collect(),
            rules: vec![Rule {
                name: Ident("all".to_string()),
                predicate: catherine_core::model::Predicate::And(Vec::new()),
                action: "all".to_string(),
            }],
            policy: Some(Policy::PriorityOrder),
        };
        let values = vec!["kind=lit,neg".to_string()];
        let domain = bind_domain(&presentation, &values).expect("binds");
        assert_eq!(
            domain.column_sizes().get(&Ident("kind".to_string())),
            Some(&2)
        );
        let error =
            bind_domain(&presentation, &["nope=1".to_string()]).expect_err("unknown column");
        assert!(error.contains("nope"));
    }
}
