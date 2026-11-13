# ro5u review — catherine-lgl.2 (Builder as monoid)

Lead-side review. Deliverables: `fixtures/katas/builder-monoid/` (base,
four above-floor faults, near-floor control + two floor faults),
`prototype/builder_monoid_baseline.py`, five entries in
`ledger/katas/builder-monoid/`, and
`crates/catherine-ledger/tests/builder_monoid_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
5 entries; kata test green; entries checked (control killed; positives
survived/killed as reported, executed arms).

Five passes:

1. Correctness: the probe's findings are the engine's (25 lost
   row/cell witnesses on the dropped-step migration, one violation on the
   out-of-order splice, one lost cell on the optional part). The
   duplicated-step entry is honestly **killed**: the check surface is
   silent on instance multiplicity (a recorded expressibility gap), and
   the conventional sampled test catches the row, so unique is zero. No
   goalpost move.
2. Contract drift: kata entries, no catalog pin, validator untouched;
   `model_hashes` canonical.
3. Tests: fixture hashes pinned; both arms replayable; the baseline
   asserts its frozen expectations.
4. Negative matrix: the floor control self-negates. The 25 migration
   uniques are recorded as surface-coverage evidence (the baseline never
   executes a schema map), not a detection-rate win; E4 respected.
5. Regression risk: fixtures/tests/ledger only.

Findings + dispositions:

- [NOTE, RECORDED] Instance multiplicity (the duplicated row itself) is
   not expressible as a path equation; the duplicate declaration is, and
   the conventional test catches both. Recorded as a gap in the entry.
- [NOTE, ACCEPTED] The migration unique count is large because the
   surface enumerates every lost element; the review keeps the framing
   "coverage", not "advantage".

max_critical = 0. Verdict: PASS.