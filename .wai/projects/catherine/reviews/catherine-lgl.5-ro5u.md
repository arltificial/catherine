# ro5u review — catherine-lgl.5 (Memento save/restore round-trip)

Lead-side review. Deliverables: `fixtures/katas/memento-roundtrip/`
(identity control + dropped-field, lossy-optional, version-skew),
`prototype/memento_roundtrip_baseline.py`, four entries in
`ledger/katas/memento-roundtrip/`, and
`crates/catherine-ledger/tests/memento_roundtrip_kata.rs`.

Orchestrator verification: baseline `--assert` exit 0; `ledger-check` ok
4 entries; kata test green (6 tests); entries checked (control killed;
positives 2/2/4 unique with executed arms); `cargo fmt --all --check` clean.

Five passes:

1. Correctness: the loss witnesses are the Δ engine's (named cells; the
   version-skew case also carries the round-trip non-invertibility proof
   naming `SnapshotPage` row 0 and the Document row it cannot return
   from). The identity control reports `lost: []` and no proof — the
   negative control self-negates. Kill-condition check: the conventional
   sampled save/restore diff catches only sampled fields and is blind to
   the unsampled losses, so the P11-baseline-armed comparison is
   `survived`, scoped.
2. Contract drift: kata-shaped entries, no catalog pin, validator
   untouched; canonical fixture hashes; the control's mistyped-hash
   incident was caught by the pin before any record survived, and the
   re-recorded entry is the one committed.
3. Tests: six frozen tests pin hashes and entry invariants; both arms
   replayable.
4. Negative matrix: control is a full-fidelity identity map; it yields no
   findings. The baseline's sampling (which fields it diffs) is
   pre-registered in faults.json, so the unique band is the sampling gap
   plus the round-trip proof — not a surprise.
5. Regression risk: fixtures/tests/ledger only; no kernel delta.

Findings + dispositions:

- [NOTE, RECORDED] The unique counts include migration lost-cell
   enumeration, which is capability-surface coverage rather than a
   detection-rate comparison; the README says so.
- [NOTE, ACCEPTED] The round-trip proof is the kata's strongest evidence:
   a named unrecoverable element, not a generic "non-invertible" verdict.

max_critical = 0. Verdict: PASS.