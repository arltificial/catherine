# Tasks: add-delta-migration

One red→green→refactor cycle (beads `catherine-alr`).

## RED

- [ ] 1. Fixture (kata-10 pattern): map dropping an object → one
      lost-data finding per row, never clean
- [ ] 2. Fixture: field-narrowing map → lost-field findings
- [ ] 3. Fixture: non-invertible round-trip → proof with offending
      element

## GREEN

- [ ] 4. Schema maps in format (additive `format_version` minor bump +
      JSON Schema update); totality validated (partial map → labeled
      error naming the unmapped element)
- [ ] 5. Pullback reindex implementation; no-invention property test
- [ ] 6. Lost-data enumeration + round-trip analysis; findings labeled
      `executed`
- [ ] 7. CLI subcommand (`catherine migrate`) with envelope output
- [ ] 8. Bogart-shaped rehearsal fixture: raw corpus → serving projection
      map (dry run for P9)

## REFACTOR

- [ ] 9. Share schema-map validation with parser; dedupe finding
      construction

## Validation

- [ ] 10. `just ci` green; `openspec validate add-delta-migration
      --strict`; `bd close catherine-alr` + fresh export
