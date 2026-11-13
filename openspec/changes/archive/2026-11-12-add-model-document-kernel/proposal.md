# Change: Add the model document kernel — `.ct.json` format, parser, canonical hashing
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Catherine has converged design docs (README, PROPOSAL, THEORY, DESIGN; gates
green) but no code and no executable contracts. Everything downstream
(executed checks, findings, CLI, wasm) consumes one artifact: the **model
document** (`.ct.json`: schema + instance). This change establishes that
format and its parser as the kernel skeleton, starting from specodelic's
`acset-core` executable contracts — **adapt, don't author** (DESIGN §3 reuse
map; E8 parity-gated extraction). Beads ticket: `catherine-bpz` (P0, the
only unblocked ticket; blocks `catherine-0gk`).

## What Changes

- New crate `crates/catherine-core` (Rust workspace root): schema
  presentation types (objects, arrows, path equations with rationale +
  status), instance presentation types (rows, per-arrow value tables with
  `null` = dangling reference)
- Parser for `.ct.json` v1: malformed presentations rejected with
  **line-accurate errors** (golden fixtures: duplicate idents, unknown
  arrows in equations, ill-typed equation endpoints — path src/tgt must
  match; equation `status ∈ {proposed, accepted}`)
- Canonical serialization: objects/arrows/equations enumerated in canonical
  sorted order so serialization is byte-stable regardless of input order
  (adapted from acset-core `canonical_order` + `build_deterministic`)
- Canonical-JSON + sha256 model hashing, ported from wild `formats.rs`
  (byte-for-byte normative) — model identity for later content-addressed
  ledger use (P6)
- Round-trip property test: `parse(serialize(m)) == m` for arbitrary models
  (proptest, not insta-snapshots)
- Published JSON Schema artifact for `.ct.json` v1 (editor/agent
  validation); `format_version` policy from day one: additive = minor,
  breaking = new extension, never in-place mutation of v1 (wild
  accretion-laws discipline)
- Openspec capability `model-document`: contract adapted from specodelic
  `acset-core` (drop corpus-specific invariants: fixed 5-object schema,
  typing-table drift, adapter parity — those are specodelic-domain)

## Impact

- Affected specs: `model-document` (new capability — first deployed spec
  in this repo)
- Affected code: new workspace + `crates/catherine-core` only; no CLI, no
  wasm, no UI
- Beads: implements `catherine-bpz`; unblocks `catherine-0gk` (P1 checks)
- Donors: specodelic `acset/instance.rs` typing conventions (Apache-2.0,
  NOTICE preserved); wild `formats.rs` canonicalizer (Apache-2.0)
- Deliberately out of scope: executed checks (P1), findings taxonomy
  (P2), analysis of any kind (P4)
