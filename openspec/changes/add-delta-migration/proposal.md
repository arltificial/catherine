# Change: Add the Δ migration axis — schema evolution with loud lost-data proofs
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Migration is the one capability with **no conventional counterpart**
(C3/E4, kata-10): conventional refactoring silently loses data; Δ
migration can *prove* what is lost, row by row, before it happens. This
is catherine's flagship scenario (bogart's raw-corpus → serving-projection
case is already identified, DESIGN §5). Gated on P6 `survived`. Beads
ticket: `catherine-alr` (P7; blocked by `catherine-9vv`; unblocks
`catherine-nq5`).

## What Changes

- Schema maps (object and arrow assignments between schemas) as format
  extension within `format_version` additive policy — **total maps**
  (every source object and arrow assigned; partial maps are labeled
  build errors)
- Δ reindexing: pullback along the schema map — reshape data, invent
  nothing (Spivak functorial migration; Σ/Π deliberately out of scope
  until Δ is validated)
- **Loud lost-data counterexamples** (kata-10 pattern) as fixtures: a
  migration that discards rows or fields exhibits each discarded element
  concretely — a migration report lists what is dropped, never silently
  drops
- **Unsupported-inverse proofs**: round-trip Δ∘Δ′ analysis that proves
  "this mapping cannot be inverted" with the offending element, rather
  than failing at runtime later
- Findings integrate with the taxonomy: lost data is a finding with
  witnesses, labeled `executed`
- Openspec capability `migration`

## Impact

- Affected specs: `migration` (new capability; consumes `model-document`,
  `findings`)
- Affected code: `crates/catherine-core` (schema maps, reindex) + CLI
  subcommand; format additive version bump
- Beads: implements `catherine-alr`; unblocks `catherine-nq5` (P8)
- Practical target: bogart raw working corpus vs derived serving
  projection (P7 fixture shape; P9 applies it live)
