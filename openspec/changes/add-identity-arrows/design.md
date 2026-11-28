# Design: identity arrows

Docs-only change; implementation lands in a later phase. One feature
cycle (RED → GREEN) plus one refactor cycle. Additive format marker
(`KNOWN_MINOR` 6 → 7); documents without identity usage stay
hash-neutral, so no sealed artifact moves. Grounding: the carried
decision of bead `catherine-1bs` — *empty path vs declared id arrow* —
resolved from the code recon toward the empty path.

## Decision 1 — the identity form is the empty path, not a declared id arrow

An equation side `[]` is the identity on the equation's domain object.
Rationale, from the touchpoints:

- **Parse/check**: one branch in `check_path`
  (`crates/catherine-core/src/parse/equations.rs`) replaces the
  `equation path must be non-empty` syntax error; `check_same_endpoints`
  needs the empty side's endpoints *derived* (Decision 3) instead of
  `path_endpoints` indexing `path.0[0]`/`last` (which panics on empty).
  That is the entire parse surface.
- **Engine**: `engine_compose`
  (`crates/catherine-engine/src/engine/compose.rs`) already returns
  `Some(start)` for a zero-length path — the compose walk is
  identity-neutral today; only a test pins it.
- **Canonical**: `equation_value`
  (`crates/catherine-core/src/canonical/value.rs`) canonicalizes paths
  as JSON arrays — an empty path is already `[]`. Nothing to invent.
- **Analysis**: `parallel.rs` `related()` compares `Path` values
  directly — identity paths compare for free.

**Rejected alternative — a declared id arrow** (`id_Spec` as a
`RawArrow`): strictly more surface at every layer. The `RawArrow`
schema (`crates/catherine-core/src/model/schema.rs`), the arrow table
in `check_path`, canonical arrow values, and — decisively —
`engine_compose` reads `typed.tables[step]`, so an id arrow needs a
special case or a synthesized identity table threaded through the
bounded engine representation. It buys nothing the empty path does not:
both state `realize∘abstract = id_Spec` in one declaration, and the
declared form additionally forks "arrows with data" from "arrows
without", a distinction every consumer (diff, render, schema_map, web
types) would then carry. Rejected also because it inverts the epistemic
direction: the user would declare a *thing* (an arrow) where the
meaning is the *absence* of a path.

## Decision 2 — both sides empty is ill-typed, never defaulted

An equation whose lhs and rhs are both empty is rejected at parse with
`ErrorCode::IllTypedEquation` at the equation's node ("both sides are
the identity path — the equation has no object; state the identity on
one side"). Rationale: such an equation has no domain object to
evaluate over, so any semantics would be invented. The hard anti-goal
is one-directional and stays one-directional: **identity is explicit,
never inferred**. Treating an ambiguous or unresolvable path as
identity would default "could not evaluate" to "vacuously true" — a
dangling non-empty path remains not-evaluable-through-dangling (existing
kernel-checks behavior, unchanged by this design).

## Decision 3 — the domain comes from the non-empty side; one typing rule

`check_same_endpoints` restructures from "compute both endpoint pairs,
compare" to "compute each side's `Option<(src, tgt)>` (empty side →
`None`), then":

1. both `Some` and equal → ok (existing rule, unchanged);
2. exactly one `Some` → its pair is the equation's endpoints, and the
   empty side is the identity on that source object;
3. both `None` → the Decision-2 rejection.

`compile_equations` (`crates/catherine-engine/src/engine/compile.rs`)
mirrors rule 2: today `domain = lhs.get(0)…` compiles an empty lhs to
`domain: None`, which the walk **silently skips** — after this change
the domain is the first non-empty side's lead-arrow source, and rule 3
never reaches compile. This closes a silent-skip hole, not just a
syntax gap: the current `None`-domain path is exactly the "declared but
never executed" failure the kernel-checks spec exists to forbid.

## Decision 4 — the identity canonical form is `[]`, pinned before drift

Defined here so hash fixtures can pin it before any format movement:

- **Canonical bytes**: an equation side is a JSON array of arrow-name
  strings; the identity side is the empty array `[]`
  (`equation_value` → `path(&[])`), keyed as usual (`"lhs"`/`"rhs"`).
  No new field, no new section — there is nothing to omit, so
  **hash-neutrality is automatic** for documents without identity
  usage, pinned by the established pattern
  (`crates/catherine-core/tests/coproducts.rs::absent_coproducts_are_hash_neutral`,
  and `current_major_parses_a_1_5_document` for the known-minor cap):
  every pre-existing golden fixture, including 1.0 documents,
  canonicalizes byte-identically.
- **Findings JSON**: `EquationWitness`
  (`crates/catherine-cli/src/render/envelope.rs`) carries composed row
  ids. An empty path composes to its start row, so the identity side's
  witness value is the row id the path started at — the envelope shape
  is unchanged; no new finding kind, no new witness variant.
- **Human renderings**: one spelling rule, defined once and reused —
  an empty path spells as `id_<object>` where `<object>` is derivable
  from the counterpart side of the same constraint (its source object;
  both sides non-empty can never reach a renderer with an empty side,
  and both-empty never parses). Applied by `diff.rs` `spelled()`
  (e.g. `[realize,abstract] = []` → `[realize,abstract] = id_Spec`),
  the `path_string` call sites in `render.rs` violation lines, and the
  `rules.rs` human line. A bare `path_string(&Path([]))` (no
  counterpart) spells `id` — unreachable in rendered output, kept
  total for tests.

## Decision 5 — the version bump is a marker, not a gate

`KNOWN_MINOR` bumps 6 → 7 (`1.7`) because the repo convention is to
raise it when new additive semantics land (the version-gate hint text
and `negative_matrix` rows state the cap). But **no per-feature version
floor** is added: the parse gate remains `minor <= KNOWN_MINOR`, and an
older-minor document may use identity paths. Rationale: this is the
existing precedent — the coproduct fixture
`gap_suite/coproduct_payments.ct.json` declares `1.3` and parses, and
the `current_major_parses_a_1_5_document` test pins semantics-
not-gated-per-minor. Consequently the identity fixtures
(`identity_involution.ct.json`, `1.3`) flip to executed **without
editing their `format_version`**, and "format-1.0 documents parse
byte-identically" holds: the version bump is the only rejection
boundary this change adds. The `model-document` versioning
requirement's MODIFIED delta ("1.7 adds identity paths") is authored at
apply per the shared-requirement rule (coproduct design Decision 6).

## Decision 6 — evidence bar: the re-model must reproduce, not just parse

The vendored `notes/identity-arrows-evidence/adjunction.ct.json`
currently states its triangle laws through four endomorphism stand-ins
(`spec_round`, `impl_round`, `impl_t_round`, `test_round`). The
Verification arc re-models it: endomorphisms deleted, triangle laws
stated as `[realize, abstract] = []`-style identity equations — and the
re-model must reproduce the **same 4 drift violations** (drift case
`mod_ddl_doctor`) under `catherine check`. If identity semantics
changed the drift verdict, the change is wrong regardless of the unit
tests. The underconstraint meter runs over the same vendored corpus
docs against the committed baseline envelope
`notes/identity-arrows-evidence/underconstraint.json`: the 1611
round-trip-family `parallel_paths` questions collapse to O(1)
declared-law checks.
