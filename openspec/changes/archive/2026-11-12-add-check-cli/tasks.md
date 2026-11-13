# Tasks: add-check-cli

One red→green→refactor cycle (beads `catherine-86i`).

## RED

- [x] 1. Acceptance fixture: violating model → exact envelope JSON shape
      + violation exit code (structural assertion, not byte-compare)
- [x] 2. Fixture: malformed model → labeled error envelope, error exit
      code, line/column present

## GREEN

- [x] 3. `crates/catherine-cli` + clap wiring; genesis-vibes stable
      modules (envelope, guide) — kernel untouched
- [x] 4. `catherine check` human + `--json` renderers over `findings`
- [x] 5. `catherine findings` view
- [x] 6. Exit-code table per finding class; document in `--help` + README
- [x] 7. End-to-end walkthrough scripted (DESIGN §5 example) as an
      integration test

## REFACTOR

- [x] 8. Extract render/format module; dedupe envelope construction

## Validation

- [x] 9. `just ci` green; envelope validated against genesis-vibes
      expectations; `openspec validate add-check-cli --strict`; `bd close
      catherine-86i` + fresh export
