//! Purpose: the merge surface over the kernel's checked pushout
//! (ticket `catherine-kw9.8`) — merging two models along their shared
//! sub-schema and reporting identity conflicts as UNKNOWN-labeled
//! merge cells with witnesses instead of aborting.
//!
//! Responsibilities: [`run`] loads two model documents and glues them
//! through the kernel pushout ([`super::load_glued`]'s own path). A clean
//! or partial merge (one side lacks the shared entity) returns the merged
//! [`Glued`] model with no cells. A shared-entity identity conflict (the
//! kernel's `schema_conflict`: the same arrow name declared with
//! incompatible endpoints/types on both sides) is downgraded from a hard
//! error to one merge cell labeled [`EpistemicLabel::Unknown`] whose
//! [`MergeWitness`] names both conflicting sides.
//!
//! Anti-goal: never silently place an unplaceable cell — an identity
//! conflict yields `glued: None` and the UNKNOWN witness IS the feature.
//!
//! Rationale: the CLI never re-implements the merge (design Decision 1);
//! this module only re-classifies the pushout's one hard failure mode so
//! callers can present conflict witnesses without losing the whole run.

use super::{load_document, glue_error, Glued};
use catherine_core::composition::glue;
use catherine_core::findings::EpistemicLabel;
use crate::render::LabeledError;

/// One merged model along with any identity-conflict cells the merge
/// could not resolve.
#[derive(Debug)]
pub struct MergeOutcome {
    /// The merged model when the pushout completed (clean or partial).
    /// `None` exactly when an identity conflict left the shared
    /// sub-schema unmergeable.
    pub glued: Option<Glued>,
    /// One UNKNOWN-labeled cell per unresolved identity conflict. Empty
    /// for every clean or partial merge — never a spurious label.
    pub cells: Vec<MergeCell>,
}

/// A merge cell the pushout could not place, labeled UNKNOWN, carrying
/// the witness that names both conflicting sides.
#[derive(Debug)]
pub struct MergeCell {
    /// Always [`EpistemicLabel::Unknown`] today: the pushout could not
    /// decide the shared identity, so nothing is silently placed.
    pub label: EpistemicLabel,
    /// Both conflicting sides of the shared identity.
    pub witness: MergeWitness,
}

/// The two sides of an identity conflict: the shared name and each
/// document's own declaration of it.
#[derive(Debug)]
pub struct MergeWitness {
    /// The shared identity that collided (the arrow or object name).
    pub entity: String,
    /// The left (first-input) document's declaration.
    pub left: String,
    /// The right (second-input) document's declaration.
    pub right: String,
}

/// Merge two model documents along their shared sub-schema. Clean and
/// partial merges return the merged model with zero cells; a shared
/// identity conflict returns the UNKNOWN cell with both sides named.
///
/// No existing verb reaches this path (`check`'s [`super::load_glued`]
/// hard-errors on the same conflict), so a CLI verb is required. The
/// exact `main.rs` wiring line (orchestrator applies):
/// `cli::Commands::Merge { left, right } => composition::merge::run(&left, &right),`
pub fn run(left_path: &str, right_path: &str) -> Result<MergeOutcome, LabeledError> {
    let left = load_document(left_path)?;
    let right = load_document(right_path)?;
    match glue(&[left, right]) {
        Ok(glued) => Ok(MergeOutcome {
            glued: Some(glued),
            cells: Vec::new(),
        }),
        Err(error) if error.code() == "schema_conflict" => {
            Ok(MergeOutcome { glued: None, cells: vec![conflict_cell(&error)] })
        }
        Err(error) => Err(glue_error(error)),
    }
}

/// The UNKNOWN merge cell for one kernel `schema_conflict`: the witness
/// names the shared identity and each side's own declaration (document
/// plus endpoints, split out of the kernel's stable two-sided message).
fn conflict_cell(error: &catherine_core::composition::CompositionError) -> MergeCell {
    let documents = error.documents();
    let message = error.message();
    let endpoints = message
        .split_once("endpoints ")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once(" and "));
    let (left, right) = match (documents.first(), documents.get(1), endpoints) {
        (Some(left_doc), Some(right_doc), Some((left_endpoints, right_endpoints))) => (
            format!("{left_doc}: {left_endpoints}"),
            format!("{right_doc}: {right_endpoints}"),
        ),
        _ => (message.to_string(), message.to_string()),
    };
    MergeCell {
        label: EpistemicLabel::Unknown,
        witness: MergeWitness {
            entity: error.node().unwrap_or(message).to_string(),
            left,
            right,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::model::Ident;

    fn fixture(name: &str) -> String {
        format!("{}/tests/fixtures/merge/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn conflicting_shared_arrow_yields_unknown_cell_naming_both_sides() {
        let outcome = run(&fixture("conflict_a.ct.json"), &fixture("conflict_b.ct.json"))
            .expect("a conflict is a reported outcome, not an error");
        assert!(outcome.glued.is_none(), "a conflicted pushout places no model");
        assert_eq!(outcome.cells.len(), 1);
        let cell = &outcome.cells[0];
        assert_eq!(cell.label, EpistemicLabel::Unknown);
        assert_eq!(cell.witness.entity, "f");
        assert!(
            cell.witness.left.contains("conflict_a.ct.json")
                && cell.witness.left.contains("Alpha -> Beta"),
            "left side names its document and endpoints: {}",
            cell.witness.left
        );
        assert!(
            cell.witness.right.contains("conflict_b.ct.json")
                && cell.witness.right.contains("Beta -> Alpha"),
            "right side names its document and endpoints: {}",
            cell.witness.right
        );
    }

    #[test]
    fn clean_merge_has_no_unknown_cells() {
        let outcome = run(&fixture("clean_a.ct.json"), &fixture("clean_b.ct.json"))
            .expect("a clean merge succeeds");
        assert!(outcome.cells.is_empty(), "clean merges carry no UNKNOWN label");
        let glued = outcome.glued.expect("clean merge returns the model");
        assert!(glued.schema.arrows.contains_key(&Ident("f".to_string())));
    }

    #[test]
    fn partial_shared_entity_proceeds_along_available_side() {
        let outcome = run(&fixture("partial_a.ct.json"), &fixture("partial_b.ct.json"))
            .expect("partiality along the intersection is allowed");
        assert!(outcome.cells.is_empty());
        let glued = outcome.glued.expect("partial merge returns the model");
        assert!(glued.schema.arrows.contains_key(&Ident("f".to_string())));
        assert!(glued.schema.arrows.contains_key(&Ident("g".to_string())));
    }
}
