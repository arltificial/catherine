# Tasks: add-executed-checks

One red→green→refactor cycle (beads `catherine-0gk`).

## RED — checks fail before they exist

- [x] 1. Golden fixture: one violating row on an accepted equation →
      expected violation outcome with witness (equation, row, lhs, rhs)
- [x] 2. Golden fixture: dangling arrow value → expected dangling outcome
- [x] 3. Golden fixture: typing violation (arrow value of wrong object) →
      expected build-time typing violation
- [x] 4. Golden fixture: proposed equation failing → outcome labeled
      proposed/hypothesis, distinct from violation

## GREEN — the E3 bet executed

- [x] 5. Port instance tables from specodelic `instance.rs` (dense row
      ids, `Vec<Option<RowId>>` arrow tables) behind the parity gate
- [x] 6. Instance builder: total on parsed presentations; typing
      violations recorded, never silent drops
- [x] 7. Path composition + equality evaluation per row; dangling
      short-circuit semantics
- [x] 8. Status split: accepted → violation; proposed → hypothesis-labeled
- [x] 9. Parity gate run over vendored fixtures (`fixtures/parity/`,
      imported from specodelic's corpus): identical findings; record
      result in this change before any rewrite
- [x] 10. Property test: empty schema/instance → zero outcomes; self-loops
      legal and evaluated

## REFACTOR

- [x] 11. Extract shared path-composition helper; align error/outcome
      types with P0's error shape

## Validation

- [x] 12. `just ci` green; parity gate documented; `openspec validate
      add-executed-checks --strict`; `bd close catherine-0gk` + fresh
      export
