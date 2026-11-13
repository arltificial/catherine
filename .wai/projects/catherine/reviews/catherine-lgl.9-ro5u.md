# ro5u review — catherine-lgl.9 (negative-control bundle)

Lead-side review. Deliverables: `fixtures/katas/negative-control-bundle/`
(control cache floor, Flyweight duplicate, procedural CRUD stale read,
Decorator lookalike with a legitimate difference; clean + fault pairs),
`prototype/negative_control_bundle_baseline.py`, four entries in
`ledger/katas/negative-control-bundle/`, and
`crates/catherine-ledger/tests/negative_control_bundle_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
4 entries; kata test green (4 tests); entries checked — control and all
three positives are `killed`, every unique set empty. Independently ran
`check` on all eight fixtures: clean fixtures 0 findings; control fault
exactly 1 violation; lookalike fault exactly 1 (the true `Product#1`,
no false `Product#0`); Flyweight duplicate and CRUD stale read 0.

Five passes:

1. Correctness: the bundle does what it claims — silence on the
   unexpressible lenses (Flyweight identity, procedural read-after-write),
   a true-only finding on the declared lookalike transformation, and a
   control whose single coherence equation is exactly what the
   conventional arm checks. No fabricated finding.
2. Contract drift: kata entries, no catalog pin, validator untouched;
   canonical fixtures hashes pinned; observed results appended after the
   pre-registration without editing it.
3. Tests: frozen pins and entry invariants; both arms replayable; the
   baseline asserts its own frozen matrix.
4. Negative matrix: this *is* the suite's negative matrix. The falsifiers
   are written down (control unique non-empty = invalid run; any finding
   outside the pre-registered set = fabricated = failure). All three
   positives correctly stay killed with recorded gaps.
5. Regression risk: fixtures/tests/ledger only.

Findings + dispositions:

- [RESULT, RECORDED] The honesty gate passes: no weak-lens disposition in
   lgl.8 is bought with false positives.
- [NOTE, RECORDED] The lookalike gap (no underconstraint detector on a
   wrongly omitted equation) is a real surface limitation, named rather
   than smoothed.

max_critical = 0. Verdict: PASS.