# Tasks: add-check-cli

One red→green→refactor cycle (beads `catherine-86i`).

## RED

- [ ] 1. Acceptance fixture: violating model → exact envelope JSON shape
      + violation exit code (structural assertion, not byte-compare)
- [ ] 2. Fixture: malformed model → labeled error envelope, error exit
      code, line/column present

## GREEN

- [ ] 3. `crates/catherine-cli` + clap wiring; genesis-vibes stable
      modules (envelope, guide) — kernel untouched
- [ ] 4. `catherine check` human + `--json` renderers over `findings`
- [ ] 5. `catherine findings` view
- [ ] 6. Exit-code table per finding class; document in `--help` + README
- [ ] 7. End-to-end walkthrough scripted (DESIGN §5 example) as an
      integration test

## REFACTOR

- [ ] 8. Extract render/format module; dedupe envelope construction

## Validation

- [ ] 9. `just ci` green; envelope validated against genesis-vibes
      expectations; `openspec validate add-check-cli --strict`; `bd close
      catherine-86i` + fresh export
