# Tasks: add-delta-migration

One red→green→refactor cycle (beads `catherine-alr`).

## RED

- [x] 1. Fixture (kata-10 pattern): map dropping an object → one
      lost-data finding per row, never clean
- [x] 2. Fixture: field-narrowing map → lost-field findings
- [x] 3. Fixture: non-invertible round-trip → proof with offending
      element

## GREEN

- [x] 4. Schema maps in format (additive `format_version` minor bump +
      JSON Schema update); totality validated (partial map → labeled
      error naming the unmapped element)
- [x] 5. Pullback reindex implementation; no-invention property test
- [x] 6. Lost-data enumeration + round-trip analysis; findings labeled
      `executed`
- [x] 7. CLI subcommand (`catherine migrate`) with envelope output
      — shared path (`crates/catherine-cli/src/migrate.rs` +
      `render/migrate.rs`); envelope carries `data.rows`/`data.lost`
      (kernel encoding verbatim)/`data.row_map`; lost data exits 8
      (exit table's additive class, `LOST_DATA_EXIT_CODE`), lossless
      exits 0, absent map is the labeled `no_schema_map` error;
      `tests/cli_migrate.rs` (7 tests) + 5 negative-matrix rows +
      ratchet extended to `MigrationFindingKind`
- [x] 8. Bogart-shaped rehearsal fixture: raw corpus → serving projection
      map (dry run for P9)
      — `fixtures/migration/bogart_rehearsal.ct.json` (RawCapture +
      prev_page/captured_page dropped): executed run emits exactly the
      10 documented findings (4 lost rows, 6 lost cells), exit 8;
      expectations recorded in `fixtures/migration/README.md`

## REFACTOR

- [x] 9. Share schema-map validation with parser; dedupe finding
      construction — verified, no change: `parse/schema_map.rs` is the
      only totality validator (every consumer parses through it; the
      engine consumes the already-validated map), and the migration
      taxonomy shares `EpistemicLabel` with the check taxonomy but no
      kind/witness shape (`MigrationFinding::lost_row`/`lost_cell` are
      its only constructors) — a unified wrapper would be abstraction
      without a duplicate to remove

## Validation

- [x] 10. `just ci` green; `openspec validate add-delta-migration
      --strict`; `bd close catherine-alr` + fresh export
