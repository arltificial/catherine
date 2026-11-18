# Design: add-lean-kernel-model

Grounded in the current implementation (2026-11-16 read):
`crates/catherine-core/src/{model/mod.rs,instance.rs,checks.rs,findings.rs}`
and `crates/catherine-analysis/src/{search.rs,bounds.rs}`,
`crates/catherine-core/src/migrate.rs`. Line counts: kernel surfaces are
~2.6k lines — small enough to hand-mirror; the parity oracle replaces an
extraction harness (E8's parity-gate pattern, adapted to a proof twin).

## Decision 1: Lean 4, std only, no Mathlib

The kernel's data is finite and first-order: `BTreeMap<Ident, Vec<RowId>>`
row spaces, `Vec<Option<RowId>>` partial morphism tables, `Path(Vec<Ident>)`
composition with `None` short-circuit (`instance.rs` `compose`). All of this
mirrors to Lean `List`/`Nat`/`Option` with no dependent-types heroics, so
Mathlib's dependency weight buys nothing and the Lake build stays light and
reproducible. If a later phase needs Mathlib (e.g. category-level statements
about migration functors), that is a separate decision with its own cost
accounting.

## Decision 2: mirror-first, parity-gated

The Lean modules mirror Rust shapes 1:1 (same names, same ordering
contracts, same three landing places in `TypedInstance::build`: morphism
value, dangling record, typing violation). No transpilation, no IDL: the
`lean-parity` oracle is the correspondence gate (named apart from the
existing wasm/ledger parity surfaces). The Lean-side input interface is the
kernel's own trusted projection: `Catherine.Mirror` projects a
`model.ct.json` document into `{fixture, schema, instance: {rows, values}}`,
where `instance.rows` maps each object to its dense row count (its row ids
are in sorted row-name order) and `instance.values` maps each arrow to its
dense `Option RowId` cell list (equations/keys/coproducts canonically
sorted; each cell resolved target-first, then into the first declared object
whose row-name map contains it). `--emit-mirror` writes exactly that JSON
(read in Lean via core `Lean.Json`, std-only, no parser task) and the Lean
twin consumes it, so `parse` stays wholly Rust-side: malformed-input
rejection, spans, and error recovery never enter the twin (model-document
spec). `fixtures/parity/*/expected.json` is not the mirror: it is the
`--check-build` comparison input and the presentation-route outcome. The
oracle runs over the **check-applicable fixture subset** (katas, parity,
probes check inputs), enumerated dynamically — `rule_lens`/`schema_diff`/
`migration` fixtures feed other subcommands, not the check oracle.

## Decision 3: theorem scope — the walk's honesty, stated semantically

- **In, phase 1 (formalization)**: schema, typed instance + build reports,
  `compose`, satisfaction, the `check` walk (all five finding families,
  including the `violated`-cell accounting that suppresses duplicate
  dangling reports — `checks.rs` `dangling_findings` — and the pullback
  square's commutes/covers/unique verdicts with their dangling-leg and
  dangling-cospan undecidability rules — `checks/pullbacks.rs`), the
  taxonomy's total order, the accepted/proposed label rule.
- **In, phase 2 (proofs)**: soundness for all emitted kinds (witness
  extraction from the finding is total and faithful); equation completeness
  (semantic violation on applicable row ⇒ reported); dangling discipline
  (the four "never" clauses of `checks.rs`'s contract, as lemmas).
- **Modeled, completeness deferred**: keys, coproducts, and pullbacks are
  formalized and their findings are covered by soundness, but their
  completeness theorems wait for a follow-up (the pair/collision semantics
  are the most proof-heavy and the least defect-prone in practice).
- **Out (follow-up change)**: bounded search (`FOUND`/`EXHAUSTED`/`UNKNOWN`
  honesty — `search.rs` contract), Δ migration (`no_invented_rows`,
  `lost_data_complete`, round-trip counterexample soundness — `migrate.rs`),
  rule lens. The prior recommendation (kimi `1a1e453e`) scoped these as
  week 3 "extend or stop"; this change makes the *stop* branch explicit and
  decidable at the gate rather than assumed.

## Decision 4: divergence protocol

A parity mismatch is never merged away. It becomes exactly one of:
1. a **Lean-model defect** (the twin deviates from the documented contract:
   the specs under `openspec/specs/` are the referee), or
2. a **Rust kernel defect** (the twin is right, the kernel is wrong) —
   filed as its own beads ticket, fixed in Rust, regression fixture added.

The `lean-parity` report names the fixture and the attributed side with
the beads fields, machine-readably, so CI can fail with an actionable line
and the oracle's green always means "same semantics", never "same bugs".

## Decision 5: the stop gate is part of the deliverable

The fixture corpus + property tests already pin these contracts
operationally. The proof phase earns its keep only if it surfaces something
the corpus does not (an unintended theorem hole, a contract that can't be
stated, an actual kernel defect). At the gate, the ledger records one of:
extend (follow-up change for search/migration), hold (model maintained at
phase-2 scope), or retire (Lean tree archived, lessons kept) — as a ledger
entry under a new terminal `decision` kind on the `ledger/` entry schema
(`catherine-ledger` validates and seals it). None of the three is a
failure; an unrecorded drift into half-maintained third-language code is.

## Risks

| Risk | Mitigation |
|------|-----------|
| Lean tree drifts as the Rust kernel evolves | Parity oracle in CI over `fixtures/`; divergence protocol makes drift loud |
| Proof effort balloons (coproduct pair semantics) | Deferred per Decision 3; stop gate bounds total spend |
| Two sources of truth for semantics | Lean cites spec sections; specs remain normative (Lean is evidence, not contract) |
