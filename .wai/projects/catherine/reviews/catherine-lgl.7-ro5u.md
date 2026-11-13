# ro5u review — catherine-lgl.7 (CoR + Strategy as rule lens)

Lead-side review. Deliverables: `fixtures/katas/cor-strategy-rules/`
(clean 7-handler chain table + 7 above-floor faults + two-handler
control + 3 control faults), `prototype/cor_strategy_rules_baseline.py`,
two entries in `ledger/katas/cor-strategy-rules/`, and
`crates/catherine-ledger/tests/cor_strategy_rules_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0 (10 faults, 3
above-floor unique); `ledger-check` ok 2 entries; kata test green;
entries checked (control killed; positive survived with 3 unique and an
executed arm whose artifact carries the recomputed script hash).

Five passes:

1. Correctness: the lens findings are the CLI's verbatim output (gap,
   shadow, derivable, overlap classes). Unique = 3 (unsampled fallback
   gap, dead duplicate handler, derivable branch); the other four
   above-floor faults are parity, caught by the scripted sample. The
   strategy-selection mismatch produces zero lens findings and is recorded
   as a surface boundary in `expressibility_gap` — no fabricated
   detection.
2. Contract drift: kata entries, no catalog pin, validator untouched;
   12 fixture hashes pinned; P8 scope statement (authored rule tables are
   commodity; this measures reverse-engineering transfer) is in the
   README, so no detection-rate claim.
3. Tests: frozen hash pins + entry invariants; the baseline re-executes
   the real binary arm and derives unique arithmetically.
4. Negative matrix: the two-handler floor control self-negates; its three
   injected faults are caught by the exhaustive four-input scripted test.
5. Regression risk: fixtures/tests/ledger only.

Findings + dispositions:

- [NOTE, RECORDED] The one pre-existing pretender violations list
   included repo files outside this slice; the slice's own files are
   clean. The orchestrator runs the repo-wide gate before the final
   commit.
- [NOTE, ACCEPTED] P8's survived-scoped precedent is honored: the claim is
  scoped to the reverse-engineering transfer, not lens uniqueness on
  authored tables.

max_critical = 0. Verdict: PASS (scoped).