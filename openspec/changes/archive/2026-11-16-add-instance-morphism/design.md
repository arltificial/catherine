# Design: instance morphism check

One feature cycle (RED → GREEN) plus one refactor cycle. Additive
format minor; absent `instance_map` is hash-neutral. Grounding: the G2
analysis scoped this as "verify a given mapping — finite and decidable.
Do not build transformation search or synthesis; that's the zoo."

## Decision 1 — the map lives in the source document, hashed

`instance_map` is an additive schema-adjacent section of the document
that owns the source instance. This makes the mapping claim sealed
model evidence: hashed, diffable (dropping it is `cosmetic` at the
schema level — it constrains no rows — but recorded), and ledger-able
via `model_hashes`. A third mapping file or CLI flag was rejected: it
is exactly the "claim lives outside the document hash" shape the keys
change rejected for `rules --project`.

## Decision 2 — totality at parse, target resolution at construction

The map covers every row of every object (per-object maps union to the
whole-schema map; an uncovered row is `incomplete_instance_map` at that
node). Rationale: naturality is a per-arrow property over the whole
map; a partial map has no decidable reading without an implicit
identity default, and an implicit default is the silent-anything
failure mode (the corpus's own negative-input matrix exists because of
that class). Target resolution happens once at operation construction:
a cell naming a missing target row is a labeled `unknown_map_target`
error naming object, row, and arrow — before any findings fire. These
are malformed claims, not instance defects; dangling-as-value stays
within a single document's arrows.

## Decision 3 — commutation breaks are `violation`/`executed`

For each arrow `a: X → Y` and each source row `x`: compare
`map(a(x))` against `a(map(x))`. A mismatch fires one
`violation`/`executed` with witness `{arrow, source_row, lhs, rhs}`
(same shape discipline as equation violations; the equation encoding
stays byte-identical). Rationale: the coproduct declaration pattern —
the map asserts coherence, a break falsifies the assertion on a
concrete row, so it is a violation with a witness, not a
hypothesis-labeled question. No new finding variants; the finding
taxonomy stays closed (the R1 rule-lens discipline).

## Decision 4 — same-schema only; no general functor machinery

The operation requires identical schemas (validated by the existing
`distinguish` schema check, reusing its `schema_mismatch` error).
Same-object-name maps only. Rejected: mappings between different
schemas (that is Δ-migration's job, already shipped), natural
transformations between *schema functors* (no schema-functor layer
exists), and any synthesis (Decision: given map only). The Σ-special-
case reuse goes the other direction: `naturality` may later serve
migrate as a library call, not vice versa.

## Decision 5 — the naturality claim is probe-scoped

`ledger/probes/naturality/`: baseline arm is `distinguish`, committed
and executed, with the gap-suite finding recorded ("fires by luck of
row alignment; answers the wrong question"); floor control is a sync
defect `distinguish` does catch (the probe must show zero unique
detections there). Unique-detection claims above that floor are
arithmetic against the committed baseline, per the domain-probes spec.

## Decision 6 — one pending additive minor owns the versioning delta

This change's spec delta carries only ADDED requirements: the versioning
requirement's contract scenarios do not change with an additive minor —
only the illustrative `currently N` list does. The MODIFIED versioning
delta is owned by exactly one pending change at a time — the next
additive-minor change to archive (today `add-declarative-keys`, pending
with format 1.3); when this change applies it takes the next free
minor, bumps `KNOWN_MINOR` mechanically, and adds the MODIFIED
delta at apply time only if no other pending change owns it. Two
simultaneous pending changes modifying the same requirement fork the
overlay (the ah check overlay-conflict this decision avoids).
