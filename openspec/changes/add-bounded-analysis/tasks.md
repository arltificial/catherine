# Tasks: add-bounded-analysis

One red→green→refactor cycle (beads `catherine-1ay`).

## RED

- [ ] 1. Fixture: over-constrained schema (mutually exclusive equations) →
      contradiction within bounds
- [ ] 2. Fixture: budget smaller than search space → UNKNOWN with scope
- [ ] 3. Fixture: parallel paths, no equation → underconstraint hypothesis
- [ ] 4. Fixture: path derivable by declared equations → redundant with
      `by` witness
- [ ] 5. Fixture: two instances differing on one observation →
      distinguishable + witness

## GREEN

- [ ] 6. Bounded search engine (explicit bounds in, witness/UNKNOWN out;
      zero-size bounds → UNKNOWN with empty scope, never contradiction)
- [ ] 7. Normal-form rewriting for path equivalence (confluence scoped to
      declared equations; documented limits)
- [ ] 8. Parallel-path detection over the schema graph
- [ ] 9. Port `query.rs` traversals behind the parity gate; property
      tests green
- [ ] 10. `distinguish` over explicit observation sets
- [ ] 11. CLI commands for each analysis op (envelope output, labels
      intact)

## REFACTOR

- [ ] 12. Share bound/scope types across analysis ops; dedupe witness
      builders

## Validation

- [ ] 13. `just ci` green; parity gate documented; `openspec validate
      add-bounded-analysis --strict`; `bd close catherine-1ay` + fresh
      export
