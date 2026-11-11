# Tasks: add-bounded-analysis

One red→green→refactor cycle (beads `catherine-1ay`).

## RED

- [x] 1. Fixture: over-constrained schema (mutually exclusive equations) →
      contradiction within bounds
- [x] 2. Fixture: budget smaller than search space → UNKNOWN with scope
- [x] 3. Fixture: parallel paths, no equation → underconstraint hypothesis
- [x] 4. Fixture: path derivable by declared equations → redundant with
      `by` witness
- [x] 5. Fixture: two instances differing on one observation →
      distinguishable + witness

## GREEN

- [x] 6. Bounded search engine (explicit bounds in, witness/UNKNOWN out;
      zero-size bounds → UNKNOWN with empty scope, never contradiction)
- [x] 7. Normal-form rewriting for path equivalence (confluence scoped to
      declared equations; documented limits)
- [x] 8. Parallel-path detection over the schema graph
- [x] 9. Port `query.rs` traversals behind the parity gate; property
      tests green
- [x] 10. `distinguish` over explicit observation sets
- [x] 11. CLI commands for each analysis op (envelope output, labels
      intact)

## REFACTOR

- [x] 12. Share bound/scope types across analysis ops; dedupe witness
      builders

## Validation

- [x] 13. `just ci` green; parity gate documented; `openspec validate
      add-bounded-analysis --strict`; `bd close catherine-1ay` + fresh
      export
