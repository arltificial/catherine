# ro5u review — catherine-lgl.6 (Wrapper law preservation)

Lead-side review. Deliverables: `fixtures/katas/wrapper-laws/` (clean
8-object/15-arrow model with 7 laws, four above-floor faults, identity
wrapper control + three control faults), `prototype/wrapper_laws_baseline.py`,
two entries in `ledger/katas/wrapper-laws/`, and
`crates/catherine-ledger/tests/wrapper_laws_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
2 entries; kata test green (6 tests); entries checked (control killed;
positive survived with 5 unique and an executed arm).

Five passes:

1. Correctness: the kernel's findings are dangling adapter/decorator
   fields and two equation violations naming rows; the control model
   (identity wrapper) is silent and its three faults are caught by the
   exhaustive floor suite, so it self-negates. The positive's 5 uniques
   are the rows the 13-cell frozen sample does not check; the README
   records the density sensitivity explicitly (10 assertions → 6,
   13 → 5, complete 23 → 0). That is the honest framing: coverage of the
   sampled suite, not a defect class conventional testing cannot express.
2. Contract drift: kata entries, no catalog pin, validator untouched;
   9 fixture hashes pinned; the kata is marked HYPOTHESIZED with no
   series row (the ploeh §6 dictionary lists no wrapper candidate) — not
   presented as a series-validated equivalence.
3. Tests: six frozen tests (hash pins, clean-fixture silence, per-fault
   witness inventory, entry invariants); both arms replayable.
4. Negative matrix: the cross-object law `wrapper.call ≡ target.call` is
   only expressible as link-level field agreement; optionality, runtime
   dispatch and object identity are recorded gaps. No overclaim.
5. Regression risk: fixtures/tests/ledger only.

Findings + dispositions:

- [NOTE, RECORDED] Unique counts shrink to zero under a complete
   cross-product suite; the entry and README say so. This is the suite's
   clearest "coverage, not capability" case and is cited in lgl.10.
- [NOTE, ACCEPTED] An earlier nesting violation in the baseline script was
   refactored; the re-recorded uncommitted entries carry the new script
   hash and are the committed truth.

max_critical = 0. Verdict: PASS (scoped; HYPOTHESIZED, coverage-only win).