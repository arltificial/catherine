# ro5u review — catherine-lgl.1 (Composite + Null Object as monoid)

Lead-side review. Deliverables: fixtures under
`fixtures/katas/composite-null-object/` (clean model, three above-floor
faults, near-floor control), `prototype/composite_null_object_baseline.py`,
four recorded entries in `ledger/katas/composite-null-object/`, and
`crates/catherine-ledger/tests/composite_null_object_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
4 entries, no rollup; kata test green; entry shapes checked (control
killed/unique 0; positives survived with executed arms, unique 1/1/7).

Five passes:

1. Correctness: the control self-negates (both injected check faults are
   caught by the sampled conventional traversal AND by the kernel). The
   positives record one dangling witness, one equation-violation witness,
   and seven lost cells on the flatten map. The README states honestly
   that monoid associativity is unexpressible on the kernel surface; the
   closest executable laws (`parent∘top = top`, `top∘top = top`) are
   recorded, with the root-exception totality gap named. No fabricated
   associativity claim.
2. Contract drift: kata-shaped entries in their own directory, no new
   entry kind, no catalog pin (id is not a registered probe), validator
   untouched; `model_hashes` are canonical fixture hashes.
3. Tests: fixtures' model hashes pinned; ledger invariants asserted;
   both arms are replayable (script `--assert`, cargo test).
4. Negative matrix: near-floor control is a one-level composite where
   the conventional sample covers every node and the nil row — it
   self-negates. The migrate unique set is recorded explicitly as surface
   coverage (the baseline never executes a schema map), not a
   detection-rate win, matching E4.
5. Regression risk: new fixtures/tests/ledger only; no kernel change.

Findings + dispositions:

- [NOTE, ACCEPTED] Associativity — the pattern's headline law — is not
   executable with the current kernel; the kata records that gap and uses
   the identity/totality laws instead. This scopes ploeh's equivalence
   honestly (the check inherits what the surface can express).
- [NOTE, RECORDED] The seven lost-cell unique detections are
   capability-surface evidence, not a detection-rate comparison; the
   entry text says so.

max_critical = 0. Verdict: PASS (scoped: identity/totality laws only).