# ro5u review — catherine-lgl.4 (State pattern reverse modeling)

Lead-side review. Deliverables: `fixtures/katas/state-discovery/` (control
+ four above-floor cases), `prototype/state_discovery_baseline.py`, five
entries in `ledger/katas/state-discovery/`, and
`crates/catherine-ledger/tests/state_discovery_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
5 entries; kata test green; entries checked (control killed; three
survived with one unique each; the unreachable-state case killed/null).

Five passes:

1. Correctness: findings are the engine's — a dangling target, an
   unfilled declared cell, an equation violation on drift, and the honest
   silence on unreachable states (recorded as a null kill, not a
   fabricated finding). The two `parallel_paths` discovery questions are
   recorded verbatim and **never counted** as detections. Parse rejections
   for undeclared states are recorded as construction probes, excluded
   from unique — the right classification.
2. Contract drift: kata entries, no catalog pin, validator untouched;
   canonical model hashes; the modeling deviation (declared Cell object to
   make totality expressible) is documented in README and entry
   discrepancies.
3. Tests: fixture hashes pinned, ledger invariants asserted, both arms
   replayable.
4. Negative matrix: the near-floor control self-negates; the
   missing-exit expectation was interpreted through the executed dangling
   (not upgraded to an underconstraint finding) — recorded honestly.
   Unreachability is a recorded gap.
5. Regression risk: fixtures/tests/ledger only.

Findings + dispositions:

- [NOTE, ACCEPTED] The kata's headline product (discovery pressure) shows
   up as hypothesized questions, not executed findings; the entry schema
   separates them correctly.
- [NOTE, RECORDED] Unreachable-state detection is not expressible with the
   current surface; recorded as a null result.

max_critical = 0. Verdict: PASS (scoped: executable state/transition
laws; discovery questions recorded as hypotheses).