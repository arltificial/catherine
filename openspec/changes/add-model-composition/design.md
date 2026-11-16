# Design: model composition

One feature cycle (RED → GREEN) plus one refactor cycle. No format
change — composition is an operation over already-parsed documents, so
every sealed hash is untouched. Grounding: the G1 gap analysis scoped
this as "executed checks at integration seams, findings not a
model-algebra platform, gated probe with a ledger entry, not a headline
feature".

## Decision 1 — variadic `check`, not a new subcommand

The gap fixture pinned the failed invocation `catherine check billing
shipping` (exit 2). Making `check` accept 2+ documents repairs exactly
that surface and inherits the exit-code contract, the envelope, and the
determinism guarantees for free. A separate `compose` subcommand was
rejected: it would fork the agent loop surface (two ways to run the
same kernel operation) for zero demonstrated gain — the same
minimal-implementation reasoning that rejected a new `probe` ledger
entry kind.

## Decision 2 — gluing is by name; conflicts are labeled errors

The pushout identifies shared structure by declared name: objects,
arrows, equations, keys, and rows. Overlap rules:

- an object or arrow declared in both inputs must have an identical
  declaration (arrow endpoints); otherwise a labeled
  `schema_conflict` error naming both documents and the node;
- a row id present in both inputs must have identical values in every
  commonly-declared arrow; otherwise `row_conflict` naming the row;
- equations and keys are never merged-away: both sides are enforced on
  the merged instance (a repeated identical declaration enforces
  twice, exactly as repeated declarations within one document already
  do — presentation repetition, enforced multiset).

A glue whose intersection is empty (disjoint union) proceeds but is
never silent: it emits a labeled envelope warning
(`empty_composition_interface`) naming the inputs — a typo'd interface
name with no seam claim yet must not produce a signal-free joint run.
Rejected: union-with-rename (the model has no rename machinery and
agent authors share names deliberately — the name IS the interface);
last-writer-wins (the corpus's own "no silent anything" principle);
hard-failing on empty intersection (a warning preserves the envelope's
warnings channel for legitimate — if pointless — disjoint unions).

## Decision 3 — seam equations live in a document, not on the CLI

A cross-document equation is an ordinary `equations` entry in one input
whose paths reference names declared in another input. Validation is
deferred to the post-glue stage: pre-glue, such an equation would fail
`unknown_arrow`, so document-level parse validates *syntax* only and
name resolution happens once over the pushout. Two properties make the
deferral safe, pinned here:

- **One resolution path.** Single-document runs resolve through the
  same post-glue stage over a trivially-single-document pushout — there
  is no parse-time-vs-glue-time fork to diverge.
- **Spans survive deferral.** Each unresolved path carries the
  line/column of its offending node from parse into resolution, so a
  glue-time `unknown_arrow` is byte-identical (code, pointer, message
  shape) to today's parse-time one; the RED suite asserts the
  single-document error envelope is byte-identical to the current
  binary's. This keeps the seam
claim hashed, sealed, and diffable (`dropping the seam equation is
`weakened` through the existing diff scan) — a CLI flag or third file
would be unhashed evidence about a claim, the exact shape the keys
change rejected for `rules --project`.

## Decision 4 — findings only; no merged-document emission

The G1 scope guardrail is enforced by construction: the operation
produces findings with a `source` attribution naming the declaring
document; there is no `--emit`, no serialized pushout. The
hand-merge (Recipe D) remains the only materialization path. Revisit
only when a fixture demands reproducible merged artifacts — and then
as a superseding change with its own probe.

## Decision 5 — the composition claim is probe-scoped, pre-registered

`ledger/probes/composition/` per the domain-probes spec: floor control
on a fault the single-document check already catches (the probe must
show zero unique detections there), and the baseline arm is the
committed, executed hand-merged document from the gap fixture. Honest
expectation, pinned: on seam-fault *detections* the hand-merged baseline
plausibly matches the joint run (the merge carries the same equations),
so the detection arithmetic is expected to land at parity — the value
claim under probe is the **elimination of the error-prone merge step**,
which is not a detection. Per the domain-probes spec's parity shape, the
entry records the detections, an empty unique set, and the workflow-
elimination claim as a *named gap* on the entry — never as a detection
claim. A `killed` verdict kills the seam-check claim, not the capability
(probe verdicts do not gate phases).
