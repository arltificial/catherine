# Tasks: add-wasm-web-view

One red→green→refactor cycle (beads `catherine-9b5`).

## RED

- [ ] 1. Boundary test: oversized document → typed cap error, kernel not
      invoked
- [ ] 2. Golden DOM test: findings panel renders kind + label + witness
      for each finding class fixture

## GREEN

- [ ] 3. `crates/catherine-wasm`: check + analysis exports, JSON in/out
- [ ] 4. `wasm32` compile gate in CI for kernel crates
- [ ] 5. TS types generated from `ct-schema-v1.json`
- [ ] 6. `web/`: file load/save, SVG schema graph, findings panel, JSON
      textarea
- [ ] 7. Parity test: CLI and wasm produce identical finding encodings on
      the fixture corpus

## REFACTOR

- [ ] 8. Dedupe TS rendering helpers; keep web/ framework-free

## Validation

- [ ] 9. `just ci` green (incl. wasm gate); headless-browser smoke of
      load→check→render; `openspec validate add-wasm-web-view --strict`;
      `bd close catherine-9b5` + fresh export
