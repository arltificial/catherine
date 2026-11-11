# Design: add-model-document-kernel

## Decision 1 — adapt acset-core contracts, subset deliberately

specodelic's `acset-core` contract is the spec base, not a template to
imitate wholesale. Kept (adapted): morphisms typed with exactly one
src/tgt; `(source, name)` uniqueness; canonical sorted enumeration;
byte-stable builds; ids interned as dense indices with bidirectional maps;
partial morphism storage (`None` = unresolved, never dropped). Dropped as
specodelic-domain (catherine has no corpus or pre-existing graph builder):
`adapter_graph_equivalent`/`parity_with_existing` (no legacy walk to match
— catherine's parity gate arrives in P1 against specodelic's *instance*
behavior), `schema_matches_typing_table` (no format doc), refinement
predicates (no kind column).

## Decision 2 — equation `status` is format data; semantics deferred to P1

DESIGN §4.1 declares `ConstraintStatus: Proposed | Accepted` but never
defines execution semantics per status. P0 fixes only: the field exists,
is required per equation, defaults to `accepted` when absent, and is
validated to the two-value enum. The semantic split (accepted → violation
findings; proposed → hypothesis-labeled findings) is a P1 (`add-executed-checks`)
requirement, decided there explicitly rather than silently here.

## Decision 3 — line-accurate errors over a text format are a JSON obligation

`.ct.json` is JSON; "line-accurate" means the parser reports the line/column
of the offending JSON node (serde_json span tracking), not of a bespoke
text syntax. Golden fixtures pin: duplicate ident, unknown arrow in an
equation path, ill-typed equation (endpoint mismatch), unknown `status`,
`format_version` absent or > current major. Error shape: `{line, column,
code, message}` — code is a stable enum, message is human text.

## Decision 4 — canonical order is semantic, not cosmetic

`Instance`/`Schema` use `BTreeMap` keyed by `Ident`; serialization emits
sorted keys; row ids are dense per object. Consequence: model hash is
independent of key order in the input file — two byte-different files
expressing the same model hash identically. This is what makes the P6
ledger content-addressing sound.

## Decision 5 — proptest, never snapshot

Round-trip is a property (arbitrary model → serialize → parse → equal),
mirroring the DESIGN non-goal "proptest not insta-snapshots". Findings and
errors are asserted structurally in golden fixtures; bytes are never
compared for pass/fail except in the canonicalizer's own normative tests
(ported from wild, where byte-for-byte IS the contract).
