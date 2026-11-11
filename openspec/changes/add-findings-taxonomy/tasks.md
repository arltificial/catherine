# Tasks: add-findings-taxonomy

One red→green→refactor cycle (beads `catherine-1x7`).

## RED — taxonomy before producers

- [ ] 1. Golden fixtures per finding kind (violation, dangling asserted
      from real kernel runs; redundant/contradiction/underconstraint
      fixture shapes pinned for P4 producers)
- [ ] 2. Fixture: proposed-equation failure emits `hypothesized` label

## GREEN

- [ ] 3. `Finding` enum with witness payloads; invalid states
      unrepresentable (no variant without its witness)
- [ ] 4. Epistemic label field; kernel can only construct
      executed/hypothesized/unknown
- [ ] 5. Reshape P1 check outcomes into findings; update P1 fixtures
- [ ] 6. Deterministic ordering (kind, then witness ids) + serde JSON
      encoding; stability property test
- [ ] 7. Underconstraint finding shape: tagged domain-question witness
      (`ParallelPaths{lhs, rhs}` | `UncoveredInputs{class, examples}` —
      the latter reserved for P8's rule-lens gap) + the domain question
      text — emitted only by analysis (P4), type defined here

## REFACTOR

- [ ] 8. Dedupe witness construction between checks and findings modules

## Validation

- [ ] 9. `just ci` green; `openspec validate add-findings-taxonomy
      --strict`; `bd close catherine-1x7` + fresh export
