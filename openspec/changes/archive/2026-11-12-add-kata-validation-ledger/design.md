# Design: add-kata-validation-ledger

## Decision 1 — the stop condition runs through the browser (ratified)

P6 is blocked by P5 (wasm + web UI), so the kill/survive signal arrives
only after the full browser chain exists. This is ratified *as designed*:
the acceptance artifacts are browser fixtures ("port 3 validated katas as
browser fixtures", DESIGN §5), and P5 is deliberately scoped thin (viewer
only) so it cannot delay P6 meaningfully. A CLI-only fast path to the
stop condition was considered and rejected: it would fork the acceptance
surface (ledger evidence in one medium, shipped product in another).
Consequence accepted: P5 latency gates the project's exit signal — which
is the point of a stop condition.

## Decision 2 — negative control first, pre-registered

FizzBuzz (kata-6: checks fired 3/3, earned nothing — the triviality
floor) is recorded *before* any potentially-positive kata. Order is part
of the spec, not advice: it prevents the goalpost move of "we tried the
hard ones first, the easy one would have worked". The ledger records the
run order.

## Decision 3 — verdict vocabulary is closed

`killed | survived | inconclusive | invalid-run` — exactly four. A run
with tooling defects is `invalid-run`, never silently re-run until a
better number appears (ct-catlab REPORT methodology §1: the corpus's own
failure mode was verdict/record contradictions). Inconclusive runs
gates re-runs with documented cause. `killed` at the project level ends
the project: P7/P8 are beads-gated on P6's `survived`.

## Decision 4 — the ledger is content-addressed

Each entry: canonical JSON (P0 canonicalizer) + sha256 id, referencing
the model-document hashes it validated and the kata fixture revisions.
Ledger entries are append-only; corrections are new entries superseding
old ones, never edits (wild accretion discipline).

## Decision 5 — expressibility gaps are ledger data

Where a validated kata invariant is not expressible as path equations +
totality (e.g. kata-8/9 injectivity-flavored constraints), the gap is
recorded as a ledger field per kata — the tool's known limit is part of
the result, not a footnote (DESIGN §4.1).
