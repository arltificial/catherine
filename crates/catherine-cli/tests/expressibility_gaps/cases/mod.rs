//! Purpose: the executable per-row cases of the expressibility gap suite
//! (`openspec/changes/add-expressibility-gap-suite`, beads `catherine-j6g`) —
//! every known limitation from the corpus investigations becomes one
//! structural observation over committed fixture documents, so the disposition
//! table can be checked in both directions.
//!
//! Responsibilities: [`all`] registers one [`Case`] per row of
//! `docs/ref/expressibility-dispositions.md`; the observation functions live
//! in the thematic submodules [`schema`], [`migration`], [`analysis`] and
//! [`future`], on the shared CLI plumbing in [`probe`]. A case owns its
//! fixture names and an `observe` function returning [`Observation::Limitation`]
//! while the claimed limitation is observable and [`Observation::Cleared`]
//! once the behavior changes. Observations assert exit codes, finding
//! kinds/labels and labeled-error codes — never output snapshots. Each case
//! returns `Limitation` only for its exact claimed signature and `Cleared`
//! only for its exact fixed signature; every other outcome panics naming the
//! case, the exit code and stderr, so a fixture that fails for an unrelated
//! reason is never read as a flip.
//!
//! Rationale: expressibility is a surface property (openspec design Decision
//! 3), so every probe is a CLI invocation against a real document, not a
//! kernel unit test. The runner that reads the table and compares verdicts is
//! GREEN work; these cases are the RED assertions it dispatches over. The
//! split by theme keeps every file under the repository's 400-line gate while
//! leaving the registry as the one row list in Rust.

mod analysis;
mod future;
mod migration;
mod probe;
mod schema;

/// What a case observed at the current build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observation {
    /// The limitation the row claims is still observable at HEAD.
    Limitation,
    /// The limitation is gone: the case now shows the changed behavior.
    Cleared,
}

/// One disposition row's executable case.
pub struct Case {
    /// The kebab-case id shared with the disposition table.
    pub id: &'static str,
    /// Fixture documents under `tests/fixtures/gap_suite/` the case runs.
    pub fixtures: &'static [&'static str],
    /// The structural observation: `Limitation` while the row's claim holds.
    pub observe: fn() -> Observation,
}

/// The disposition table's expected registration count: twelve rows at
/// registration time (eleven `gap` and one `closed`). This is the one
/// place the number is written down; [`all`] cross-checks it against the
/// registry length so a registration change must update it deliberately.
pub const EXPECTED_ROWS: usize = 12;

/// Every registered case, in disposition-table order.
pub fn all() -> Vec<Case> {
    let cases = vec![
        Case {
            id: "identity-empty-path-equations",
            fixtures: &[
                "identity_involution.ct.json",
                "identity_memento_roundtrip.ct.json",
            ],
            observe: schema::identity_empty_path_equations,
        },
        Case {
            id: "optional-references",
            fixtures: &["optional_reference.ct.json"],
            observe: schema::optional_references,
        },
        Case {
            id: "unknown-top-level-fields",
            fixtures: &["unknown_top_level_fields.ct.json"],
            observe: schema::unknown_top_level_fields,
        },
        Case {
            id: "declared-cones-ignored",
            fixtures: &["declared_cone.ct.json"],
            observe: schema::declared_cones_ignored,
        },
        Case {
            id: "non-functorial-schema-map",
            fixtures: &["nonfunctorial_schema_map.ct.json"],
            observe: migration::non_functorial_schema_map,
        },
        Case {
            id: "dropped-equations-on-migrate",
            fixtures: &["dropped_equation_migration.ct.json"],
            observe: migration::dropped_equations_on_migrate,
        },
        Case {
            id: "vacuous-clean-instances",
            fixtures: &["wrong_shape_instance.ct.json", "zero_rows_instance.ct.json"],
            observe: migration::vacuous_clean_instances,
        },
        Case {
            id: "distinguish-observation-scope",
            fixtures: &["distinguish_left.ct.json", "distinguish_right.ct.json"],
            observe: analysis::distinguish_observation_scope,
        },
        Case {
            id: "associativity-scaffolding",
            fixtures: &[
                "associativity_scaffolded.ct.json",
                "associativity_omitted_triples.ct.json",
            ],
            observe: analysis::associativity_scaffolding,
        },
        Case {
            id: "composition-absence",
            fixtures: &[
                "composition_billing.ct.json",
                "composition_shipping.ct.json",
                "composition_seam_claim.ct.json",
            ],
            observe: future::composition_absence,
        },
        Case {
            id: "naturality-absence",
            fixtures: &[
                "naturality_crm.ct.json",
                "naturality_warehouse.ct.json",
                "naturality_warehouse_break.ct.json",
            ],
            observe: analysis::naturality_absence,
        },
        Case {
            id: "coproduct-absence",
            fixtures: &["coproduct_payments.ct.json"],
            observe: analysis::coproduct_absence,
        },
    ];
    assert_count(cases.len());
    cases
}

/// The registration count is deliberate: a registry change must update
/// [`EXPECTED_ROWS`] in the same edit.
fn assert_count(registered: usize) {
    assert_eq!(
        registered, EXPECTED_ROWS,
        "the disposition table registers twelve rows at registration time"
    );
}
