# Design: coproduct exhaustiveness

One feature cycle (RED → GREEN) plus one refactor cycle. Additive
format minor; absent `coproducts` is hash-neutral. Grounding: the G3
analysis scoped this as "the coproduct side [of the mono/coproduct
pair], the natural completion… produces genuine findings (an uncovered
case is a gap; an instance matching two variants is an overlap — both
existing finding kinds, no new epistemic machinery)."

## Decision 1 — the declaration grammar is `keys` again

`{"object": C, "arrows": [...], "status": ...}` — the exact `Key`
shape, new semantics: keys assert injectivity of the arrow-value tuple,
coproducts assert image-coverage and image-disjointness of the arrows'
targets. Canonical order `(object, arrows, status)`; duplicates kept as
presentation; section omitted from canonical bytes when empty. Rationale:
one declaration grammar means one parse discipline (`invalid_coproduct`
mirrors `invalid_key`), one canonicalization path, and a learning
surface of zero for existing users. Rejected: a `variants: [V1..Vn]`
naming scheme — variant objects are already rows/arrows; a second way to
name them would fork the model's identity rules.

## Decision 2 — coverage failures are `violation`/`executed`, not
`underconstraint`

The rule-lens mapping pinned `gap → underconstraint` *hypothesized*
"until totality is declared". Here totality IS declared — that is the
entire content of the coproduct section — so a falsified coverage claim
is a violation with a named witness row, and a falsified disjointness
claim is a violation naming the row and both injections. Rationale: the
finding must state that a *declared* constraint failed (executed), not
that an undeclared parallel path was noticed (hypothesized). The
taxonomy stays closed either way.

## Decision 3 — status participates like keys, diffs like keys

`accepted` enforces and joins the diff's enforced set; `proposed` is
preserved as a hypothesis-labeled question and stays out of the
enforced multiset. Adding an accepted coproduct is `accreted`; dropping
one is caught by the existing bounded weakening scan with no new scan
logic — the kernel satisfaction predicate already treats `executed`
violations as unclean, so the scan needs only the enlarged constraint
set. This is the keys change's integration shape, reused verbatim.

## Decision 4 — no limits machinery, no cones

Rejected: pullbacks (no fixture demands them), general limit cones
(the corpus finding that declared cones were inert prose is a warning
against declaring structure with no executed semantics — if cones ever
ship, they ship as a superseding change with their own probe), and
defaults-on-coverage-failure (a default row is synthesis — the tool
verifies claims, it does not repair them).

## Decision 5 — probe first, capability second

The G3 claim — "rogue variants are the natural data evolution and
today's tool is silent" — is pre-registered in
`ledger/probes/coproduct/` before the kernel check lands: fixtures from
the gap suite, floor control on a variant defect the *keys* check
already catches (zero unique detections there), and the baseline arm is
the committed hand-authored test suite that pins today's silent-
acceptance behavior (executed, hash-pinned). If the probe shows the
conventional test suite catches everything the coproduct check would,
the verdict is `killed` and the change is recorded as such before any
kernel code merges — the kata-6 discipline applied to a capability
change.

## Decision 6 — shared-requirement deltas are rebased, and one archive order holds

Two rules discovered in review, stated as conventions:

1. **One pending change owns a shared requirement's MODIFIED delta at a
   time; a sibling authors its delta at apply.** This change modifies the
   same two analysis requirements `add-declarative-keys` (pending)
   modifies — so its analysis delta is NOT in this proposal: it is
   authored at apply time, rebased onto keys' text **as archived by
   then**, extended with the coproduct enforced-set and the
   dropped-coproduct witness shape. The ah check overlay composes all
   pending changes, so a same-id scenario with a different body conflicts
   today even when the extension is intentional — the deferral is the
   only clean state. Review correction: an earlier draft carried the
   rebased delta in-proposal; the tooling rejected it.
2. **Archive order is fixed**: `add-declarative-keys` → this change. The
   versioning requirement is likewise owned by the next additive-minor
   change to archive (keys today; `add-instance-morphism` next); this
   change takes the next free minor, bumps `KNOWN_MINOR` mechanically,
   and adds the small MODIFIED versioning delta at apply time only if no
   other pending change owns it by then.
