# Change: Identity arrows — the empty path as the explicit identity form

> Beads: `catherine-1bs`
>
> Grounding: the vendored evidence `notes/identity-arrows-evidence/`
> (charly/eco-views spec ⇄ implementation ⇄ tests adjunction): the
> kernel cannot state `realize∘abstract = id_Spec`, so users declare
> throwaway endomorphisms (`spec_round`, `impl_round`, `impl_t_round`,
> `test_round`) as identity stand-ins — and the underconstraint lens
> then asks **1611** round-trip-family questions over one corpus that
> with identities collapses to the handful of declared laws the model
> actually means.

## Why

An equation path must be a non-empty arrow list: the parser rejects an
empty side with `error[syntax]: equation path must be non-empty`
(`crates/catherine-core/src/parse/equations.rs`), so the most basic law
of an adjunction — the triangle `abstract∘realize = id_Spec` — cannot
be stated. Every declared constraint is only as strong as the
equations it can name, and identity is the equation every round-trip,
involution, and memento claim bottoms out in. Modelers work around the
gap by declaring endomorphism arrows that mean nothing but "the
identity lives here", which pollutes schemas, multiplies
underconstraint questions superlinearly (1611 over the vendored
corpus), and still does not say what is meant.

The gap is already pre-registered: the expressibility gap suite carries
the `identity-empty-path-equations` row (fixtures
`identity_involution.ct.json`, `identity_memento_roundtrip.ct.json`)
observing today's exit-1 syntax rejection as a `Limitation`.

## What Changes

- **The empty path is the identity.** An equation side MAY be `[]`,
  the identity path on the equation's domain object. A one-sided-empty
  equation (`["realize","abstract"] = []`) takes its domain object from
  the non-empty side's source; the equation then executes like any
  other — an empty path composes to the row it starts at, so the law
  holds exactly when the non-empty path returns every row to itself.
- **Both sides empty is ill-typed, never defaulted.** An equation with
  `[]` on both sides has no object and is rejected with a labeled
  `IllTypedEquation` error at the equation's node. Identity is
  **explicit** — an empty path or nothing; the system never *infers*
  identity for an unresolvable or ambiguous path.
- **Domain derivation, not silent skips.** The compile step derives an
  equation's domain from the first non-empty side (today an empty lhs
  compiles with `domain: None`, which silently skips the check); a
  both-empty equation never reaches compile because parse rejects it.
- **Canonical and rendering are pinned in the same change.** The
  canonical bytes of an identity path are `[]`; human renderings spell
  an identity side as `id_<object>` where the object is derivable from
  the counterpart side; the findings JSON envelope needs no new shape —
  an empty path's composed value is the start row's id.
- **Format**: `KNOWN_MINOR` bumps 6 → 7 (`1.7`) as the additive marker.
  The bump is the **only** rejection boundary: no per-feature version
  floor is added, and a document that uses no identity path keeps its
  exact canonical bytes and hash — including every pre-existing 1.0
  golden fixture.

### Scope boundary — explicitly out

- **Declared identity arrows are out.** A `{"name": "id_Spec", "src":
  "Spec", "tgt": "Spec"}` declaration with synthesized identity
  semantics touches the `RawArrow` schema, the arrow table, canonical
  arrow values, and `engine_compose`'s typed-table lookup (an id arrow
  has no data table — it needs a special case or a synthesized identity
  table). The empty path achieves the same laws with none of that
  surface; a declared form can be layered later if a use case demands
  it.
- **Identity is never inferred.** An unresolvable (dangling)
  path stays not-evaluable-through-dangling; an ambiguous equation
  (both sides empty) is rejected. No check path ever *defaults* to
  identity — that would turn "could not evaluate" into "vacuously
  true", which is unsound.
- **No changes to the `search`, `distinguish`, or `agree` surfaces**,
  no new finding kinds, and no changes to the parallel-paths lens
  beyond what falls out of users no longer declaring endomorphism
  stand-ins.

## Capabilities

### Modified Capabilities
- `kernel-checks`: the path-equation requirement is amended — an
  equation side may be the explicit identity path (`[]`), with domain
  derived from the non-empty side; both-sides-empty is ill-typed.

### New Capabilities

## Impact

- Affected specs: `kernel-checks` delta only (this directory). The
  `model-document` versioning requirement is shared — its MODIFIED
  delta ("1.7 adds identity paths") is authored at apply per the
  shared-requirement rule (the coproduct change's design Decision 6
  convention).
- Affected code (implementation, per tasks.md):
  `crates/catherine-core/src/parse/equations.rs` (accept one-sided
  empty, reject both-empty), `crates/catherine-core/src/canonical`
  (pin `[]`), `crates/catherine-engine/src/engine/compile.rs` (domain
  derivation) and `families.rs`/`compose.rs` (already identity-neutral,
  test-pinned), `crates/catherine-core/src/parse/schema.rs`
  (`KNOWN_MINOR`), `crates/catherine-cli/src/render` (`id_<object>`
  spelling in `diff.rs` `spelled()`, `text.rs` `path_string` callers,
  `rules.rs` human lines).
- Evidence: the gap-suite identity rows flip to `Cleared`; the vendored
  `notes/identity-arrows-evidence/adjunction.ct.json` re-models without
  its four endomorphism stand-ins, stating triangle laws as
  `[realize, abstract] = []` while reproducing the same 4 drift
  violations (case `mod_ddl_doctor`); the vendored corpus
  underconstraint envelope collapses from 1611 round-trip-family
  questions to O(1) declared-law checks.
- Beads: feature ticket `catherine-1bs`. Out of scope: the
  catherine-ugc corpus re-model (separate ticket).
