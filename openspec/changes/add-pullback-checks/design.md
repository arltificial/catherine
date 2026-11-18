# Design: pullback checks

One feature cycle (RED → GREEN) plus one refactor cycle. Additive
format minor 1.6; absent `pullbacks` is hash-neutral. Grounding: beads
`catherine-4vs` (demand-side gate from aleandro-2bc) and the aleandro
relation inventory's pin — "resolution is a *relation object*, not an
equation". The rule-of-5 pass over the capability design produced the
three-check formulation (Decision 2) and the decidability split
(Decision 4); the ticket's GREEN acceptance criteria name the same
three checks, so proposal and ticket gate identically.

## Decision 1 — the declaration grammar is `keys`/`coproducts` again

`{"apex": P, "legs": [p1, p2], "cospan": [f, g], "status": ...}` —
apex + named arrows + status, the same one-grammar principle as keys
(object + arrows + status) and coproducts. The apex is redundant with
`src(p1) = src(p2)` but named anyway: the declaration reads as the
relation object aleandro wants, and the redundancy is validated, not
tolerated (`invalid_pullback` when the legs do not source at it).
Canonical order `(apex, legs, cospan, status)`; duplicate declarations
kept as presentation; section omitted from canonical bytes when empty.
Rejected: a `cones: [{kind: pullback, ...}]` namespace — a `kind` enum
with one variant is the speculative machinery the coproduct design's
Decision 4 rejected, and it would resurface a field name whose current
rejection (`declared-cones-ignored`) is load-bearing documentation.

## Decision 2 — the universal property is three checks, not one

In Set, `P` is the pullback of `f` along `g` iff `P` is in bijection
with the fiber `{(a,b) | f(a) = g(b)}` via `(p1, p2)`. Over a finite
instance that bijection decouples into three decidably separate
verdicts:

1. **commutes** — `f∘p1 = g∘p2` on every apex row (soundness; a row
   outside the fiber). Witness: square, row, both composed values.
2. **covers** — every fiber pair is projected by some apex row (no
   missing pairs). Witness: square, the uncovered pair.
3. **unique** — no two apex rows project to the same pair (the legs
   are jointly injective; no duplicated pairs). Witness: square, both
   rows.

Rationale: commutativity alone cannot distinguish a pullback from any
commutative square, and the aleandro experiment needs the
*enumeration* claim — a missed co-reference is exactly a coverage
failure. Rejected: commutativity-only GREEN (the ticket's original
wording) — it would verify a weaker property than the declaration
names, repeating the inert-prose mistake the corpus finding warned
against.

## Decision 3 — violation semantics reuse the status rule, not new epistemics

Accepted pullbacks fire `violation`/`executed`; proposed pullbacks
fire `hypothesized` — the equation/key/coproduct status rule verbatim
(status defaults to accepted when absent).
(Coproduct.status precedent). The witness is a new **`PullbackWitness`**
(square identity, offending row or pair) added to the fixed per-kind
witness union in the findings spec — the `CoproductWitness` precedent:
the finding *kind* set is unchanged, the violation witness union
grows. "No new semantics" in the ticket means no new exit codes
(0 clean / 3 violated stand) and no new epistemic labels.

## Decision 4 — decidability follows the coproduct undecidability rule

- An apex row whose `p1` or `p2` cell dangles: commutativity is not
  decidable for that row. The dangling findings carry it (existing
  kinds, never double-reported); the row is reported as undefined, in
  the naturality module's undefined-on-both-sides spirit — never a
  violation, never a silent clean.
- A fiber pair is decidable for coverage only when both its cospan
  cells resolve: pairs touching a dangling `f(a)` or `g(b)` cell are
  excluded from the decidable set (the fiber itself is a function of
  `f`,`g` alone — the legs never enter coverage).
- A zero-row apex over an empty fiber is *evaluated-clean*: the check
  ran, found the bijection trivially satisfied, and silence means
  evaluated (the `vacuous-clean-instances` discipline).

## Decision 5 — smallest slice; migration parity is a task, not a spec delta

The coproduct change shipped no migration spec delta and carried
declarations through schema maps as an implementation task; this
change does the same — pullbacks validate through schema maps with
the same passes as coproducts, and a map dropping an apex or cospan
arrow is loud the way dropped equations are loud. Any gap discovered
beyond that goes to the findings ledger per the ticket's rule
("expressibility gaps … not new machinery by default"). Sequencing
with `add-lean-kernel-model`: pullback checks are out of the twin's
weeks-1–2 scope; when the twin formalizes kernel semantics, the
pullback fixtures join the parity-oracle corpus — a coordination note
there, not a blocker here.

## Decision 6 — the three checks compute cheaply over finite instances

Commutes walks apex rows (path composition exists). Unique buckets
apex rows by `(p1-cell, p2-cell)` — BTreeMap, one pass. Covers groups
`A`-rows by `f`-value and `B`-rows by `g`-value per `C`-row and walks
the fiber (O(|A| + |B| + |fiber|)); uncovered pairs are enumerated in
canonical row order so findings are countable and diff-stable. Every
enumeration walks `BTreeMap` order per the migration module's
determinism rule.

Degenerate declarations are checked, not special-cased: a cospan where
`f = g` yields the diagonal fiber and all three verdicts remain
well-defined over it; no rejection and no code path treats it
differently. Kernel-checks vocabulary follows the deployed spec's
"not-evaluable-through-dangling" for verdicts a dangling cell withholds
(the naturality module's `undefined` outcome is the same shape).
