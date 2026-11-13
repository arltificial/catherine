# ro5u review — catherine-lgl.3 (Visitor + Interpreter as sum type)

Lead-side review. Deliverables: `fixtures/katas/visitor-sum-types/` (clean
5-variant table + 5 above-floor faults + two-variant control + 3 control
faults), `prototype/visitor_sum_types_baseline.py`, two entries in
`ledger/katas/visitor-sum-types/`, and
`crates/catherine-ledger/tests/visitor_sum_types_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
2 entries; kata test green (4 tests); entries checked (control killed;
positive survived with 3 unique and an executed arm).

Five passes:

1. Correctness: findings are the lens's verbatim output — a missing
   visitor method (gap), a duplicate handler under DeclaredDisjoint
   (overlap), a broad rule shadowing a later handler, a derivable
   handler, and a new variant with no rule (gap). Unique = 3 (shadow,
   derivable, missing-variant coverage) because the sampled conventional
   dispatch test covers `num/add/neg` only; the two caught faults are
   parity. The derivable handler is behavior-preserving, so no
   conventional behavioral test can reach it — named as the structural
   fact.
2. Contract drift: kata entries, no catalog pin, validator untouched; 10
   fixture hashes pinned; honest boundary (authored tables are commodity,
   P8) stated.
3. Tests: frozen pins + ledger invariants; both arms replayable.
4. Negative matrix: the two-variant control self-negates; its three
   faults are caught by the exhaustive 4-input scripted test. A
   behavior-preserving derivable handler cannot be expressed at the floor
   (distinct actions required), recorded in README and entry
   discrepancies.
5. Regression risk: fixtures/tests/ledger only.

Findings + dispositions:

- [NOTE, RECORDED] The unique band is the sampling gap plus one structural
   fact; the report says so and claims no detection-rate win.
- [NOTE, ACCEPTED] New-variant coverage is caught as a gap, which is the
   practical payoff of the sum-type reading for languages without
   exhaustiveness checks.

max_critical = 0. Verdict: PASS.