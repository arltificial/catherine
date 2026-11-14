# Brief — catherine-r5k.1: Abstract Factory gap kata

Beads ticket: `catherine-r5k.1` (P3, lane:design). Parent epic: `catherine-r5k`.

## Why

Delegation rationale: gap katas are independent research slices (one pattern each, disjoint
ledger/fixture directories), dispatched in parallel. The lead audits every verdict before any
doc claim.

## Current state

Disposition: **disposed** — construction only; no invariant on a finite instance. Nothing here is validated; the row is a gap statement.

## Desired outcome

Either prove a law or close the gap honestly:

1. Name the law BEFORE writing fixtures. Candidate to test (unvalidated): product-family totality: every declared product kind is realized by a concrete factory -- a declared-cell totality check, not a functor law.
2. Follow the P6 ledger protocol: pre-registered hypothesis, seeded-fault matrix, committed
   conventional baseline arm, near-floor negative control, verdict
   (survived-scoped | killed | inconclusive) recorded with `ledger-record` in
   `ledger/katas/abstract-factory/` — never hand-written entries, never an edited sealed entry.
3. If no law can be named, close as **still gap**: record the candidate law, the missing
   surface predicate, and why it is missing, in the ticket close reason and the ledger side of
   the record. A named-missing-law close is a valid null result, not a failure.

Fixtures live under `fixtures/katas/abstract-factory/`; the baseline script under `prototype/`
(write it, run it, commit it) so the conventional arm is executable evidence.

## Out of scope

- Do not edit existing `ledger/katas/negative-control-bundle/` or `ledger/katas/weak-lens-set/`
  entries, the 4 killed rows (Prototype, Bridge, Command, Observer), or
  `docs/ref/pattern-kata-dispositions.md` (the lead updates the row after audit).
- No detection-rate claims, no inventing a law after seeing results, no prover scope.

## Exact files

- `ledger/katas/abstract-factory/` (new entries)
- `fixtures/katas/abstract-factory/` (fixture + README)
- `prototype/abstract_factory_baseline.py`

## Red → green

1. RED: baseline script asserts the conventional arm's detections on the seeded faults; the
   ledger entry's `expected` block is pre-registered before the kernel run.
2. GREEN: kernel run matches the pre-registered expectation; verdict recorded.

## Completion criteria (runnable; exit 0 = done)

```bash
cargo run -q -p catherine-ledger --bin ledger-check ledger/katas
python3 prototype/abstract_factory_baseline.py --assert
```

## Spawn model

Agent: `task` (harness default model). Context ceiling: ≤20% of the model window.

## Follow-up threshold

If the candidate law needs a kernel capability (a new predicate or surface) rather than a
fixture, stop and report the capability gap; do not build the capability in this ticket.
