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
arrows, equations, keys, and rows. Two documents meeting on the same
declaration are two *claims*, and both are checked rather than unioned.
Overlap rules:

- an arrow declared in both inputs must have identical endpoints;
  otherwise a labeled `schema_conflict` error naming both documents and
  the node;
- the documents that declare rows for one object must declare the same
  row set; otherwise `row_conflict` naming the object, both sets, and
  the differing row names. A document that declares no rows for the
  object (the seam-claim shape, structure without an instance) carries
  no row claim. Row sets are a claim because the alternative — unioning
  partial instances — fabricates `dangling` findings no input entails:
  a row declared by one document but not by the arrow's other declaring
  document would appear as a null cell that neither document declared;
- a row id present in both inputs must have identical values in every
  commonly-declared arrow; otherwise `row_conflict` naming the row. The
  parser's `null` is a *dangling claim*, so `null` against `null` agrees
  (the merged cell stays dangling, exactly as both inputs declare it)
  and `null` against a value is a conflict, never a silent fill;
- equations and keys are never merged-away: both sides are enforced on
  the merged instance (a repeated identical declaration enforces
  twice, exactly as repeated declarations within one document already
  do — presentation repetition, enforced multiset).

A glue whose intersection is empty (disjoint union) proceeds but is
never silent: it emits a labeled warning (`empty_composition_interface`)
naming the inputs, on the envelope under `--json` and on stderr in human
mode. The interface is namespaced — objects, arrows, and `(object, row)`
pairs are three namespaces, so a row name reused on different objects or
an arrow name reused as a row name is not a shared identity; a flat name
set would suppress the warning on exactly the typo-shaped unions it
exists for. Rejected: union-with-rename (the model has no rename
machinery and agent authors share names deliberately — the name IS the
interface); last-writer-wins (the corpus's own "no silent anything"
principle); hard-failing on empty intersection (a warning preserves the
envelope's warnings channel for legitimate — if pointless — disjoint
unions); unioning partial instances (fabricated findings).

## Decision 3 — seam equations live in a document; one resolver over the pushout

A cross-document equation is an ordinary `equations` entry in one input
whose paths reference names declared in another input. Arrow declaredness
is only decidable over the union, so the equation slot runs the parser's
own `check_equations` against the document's own arrows first and carries
the one failure a union may satisfy — a path element naming no arrow of
this document — to the post-glue stage. Every other failure
(`ill_typed_equation` over local arrows, `invalid_ident`, `invalid_status`,
syntax) is raised in the slot, exactly where `parse` raises it: endpoint
typing is not deferrable, because a commonly declared arrow keeps its
endpoints, so a local mismatch can never be repaired by a union. A path
that resolves over the pushout can still be ill-typed there, and that
failure is reported at glue with the parser's code and position.
Properties, pinned here:

- **One resolver, two arrow sets.** `check_equations` is the only
  name-resolution code path: `parse` calls it with the document's own
  arrows, and `glue` calls it with the pushout's arrows for a deferred
  document. The CLI deliberately keeps the singleton on `parse` (byte
  identity is a hard requirement), so there are two *entry* paths but one
  *resolution*: a document whose equations resolve alone is validated in
  parse order and reports parse's first error at either arity, and a
  deferred document is resolved by that same function, per equation in
  file order (paths, endpoint typing, status), over the merged arrows.
  A deferred document has no standalone parse to be parity with: its
  unresolved-path failure surfaces at the glue stage, after every input's
  document-level validation.
- **Spans survive deferral.** The pending equations keep the document's
  span scan, so a glue-time `unknown_arrow` reproduces the parse-time
  code, offending-node position, and message. The RED suite asserts the
  single-document error envelope is byte-identical to the current
  binary's, and a precedence test asserts the same bytes produce the same
  first error whether one path or two are passed.
- **The claim is document content, not standalone evidence.** The
  equation lives in an input document, so it is versioned, reviewed, and
  enforced by the joint run with source attribution; there is no CLI flag
  and no third file of unhashed evidence. A document is
  standalone-parseable — and therefore model-hashable and
  schema-diffable — only when every name it references is declared in
  it: a claim naming a sibling's arrows is rejected by `parse` with
  `unknown_arrow`, and only the self-contained form (the committed
  gap-suite seam-claim document, which declares all six arrows) carries
  an ordinary sealed hash and diff surface. Making a genuinely
  cross-document claim standalone-diffable would need a
  composition-aware `diff` and is out of scope (a superseding change).

Rejected: a CLI `--claim` flag or a third "claims" file (unhashed
evidence about a claim — the shape the keys change already rejected for
`rules --project`); validating equations only at glue time (it would let
a later document phase mask a parser error the same bytes report alone).

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
