# Tasks: add-bogart-pilot

One red→green→refactor cycle (beads `catherine-hc6`).

## RED

- [x] 1. Fixture: seeded bogart-like corpus with a drifted anchor →
      violation + dangling findings fire with witness rows

## GREEN

- [x] 2. Model 1: FTS-anchor equation + totality (corpus-storage
      invariant)
- [x] 3. Model 2: identity-domain disjointness table (spec-only if P8
      not landed)
- [x] 4. Model 3: capture-selector relation (overlap/gap)
- [ ] 5. Model 4: raw→serving Δ migration + lost-data report (lands
      after P7; rehearsed in P7 task 8)
- [ ] 6. Model 5: alerts tiered rules + priority policy (lands after
      P8; reported under mapped P2 taxonomy kinds)
- [x] 7. Bogart CI wiring: pinned-rev catherine checkout + cargo build
      of `catherine-cli` (cached target), `catherine check` step,
      class exit codes, docs in bogart repo
- [x] 8. Run on the live corpus; triage every finding: real vs noise +
      cause

## REFACTOR + report

- [x] 9. Pilot report: findings fired, real/noise accounting, defect
      links, ledger entry
- [x] 10. `openspec validate add-bogart-pilot --strict`; `bd close
      catherine-hc6` + fresh export
