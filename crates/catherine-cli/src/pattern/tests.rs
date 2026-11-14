//! Purpose: unit tests for the `pattern` runner's pure seams (beads
//! `catherine-mx1.2`) — the lost-data exit decision, the two-channel
//! payload shape, the human renderers, the registry list head, and the
//! labeled usage errors.
//!
//! Responsibilities: exercise [`super`] directly; no process spawning, no
//! model loading.
//!
//! Rationale: kept in a submodule file so `pattern.rs` stays under the
//! file-size gate; the tests and their assertions are unchanged.

use super::*;
use catherine_core::findings::EpistemicLabel;
use catherine_core::model::{Ident, RowId};

fn lost_cell(arrow: &str, row: usize) -> MigrationFinding {
    MigrationFinding {
        label: EpistemicLabel::Executed,
        kind: MigrationFindingKind::LostCell {
            arrow: Ident(arrow.to_string()),
            row: RowId(row),
        },
    }
}

#[test]
fn any_migration_finding_is_the_lost_data_exit() {
    assert_eq!(outcome_exit(&LensFindings::default()), 0);
    let with_loss = LensFindings {
        checks: Vec::new(),
        migrations: vec![lost_cell("parent", 2)],
    };
    assert_eq!(outcome_exit(&with_loss), LOST_DATA_EXIT_CODE);
    assert_eq!(LOST_DATA_EXIT_CODE, 8);
}

#[test]
fn the_migration_channel_keeps_the_migrate_serde_shape() {
    let value = serde_json::to_value(lost_cell("parent", 2)).expect("serializes");
    assert_eq!(value["label"], "executed");
    assert_eq!(value["kind"], "lost_cell");
    assert_eq!(value["witness"]["arrow"], "parent");
    assert_eq!(value["witness"]["row"], 2);
}

#[test]
fn the_payload_keeps_checks_and_migrations_apart() {
    let data = PatternData {
        outcome: "analyzed",
        scope: None,
        findings: &[],
        migrations: &[lost_cell("parent", 2)],
    };
    let value = serde_json::to_value(&data).expect("serializes");
    assert!(value["findings"].as_array().expect("findings").is_empty());
    assert_eq!(value["migrations"][0]["label"], "executed");
    assert_eq!(value["migrations"][0]["kind"], "lost_cell");
    assert_eq!(value["migrations"][0]["witness"]["row"], 2);
}

#[test]
fn human_migration_lines_name_the_witness() {
    let lines = render_migrations(&[lost_cell("parent", 2)], &Schema::default());
    assert_eq!(lines, ["lost-cell: parent at 2"]);
}

#[test]
fn the_list_names_the_registry_shape() {
    let lines = list_lines(&[]);
    assert_eq!(lines, ["pattern: 0 registered lens(es)"]);
}

#[test]
fn an_unknown_pattern_id_is_a_bad_argument() {
    let error = unknown_pattern(&LensError::UnknownPattern {
        id: "visitorr".to_string(),
        known: vec!["visitor"],
    });
    assert_eq!(error.code, "bad_argument");
    assert!(error.message.contains("visitorr"));
    assert!(error.message.contains("visitor"));
    assert!(!error.hint.is_empty());
}

#[test]
fn only_the_values_spelling_is_checked_here() {
    assert!(check_values_syntax(&["kind=lit,neg".to_string()]).is_ok());
    assert!(check_values_syntax(&["kind".to_string()]).is_err());
    assert!(check_values_syntax(&["=lit".to_string()]).is_err());
    assert!(check_values_syntax(&["kind=".to_string()]).is_err());
}
