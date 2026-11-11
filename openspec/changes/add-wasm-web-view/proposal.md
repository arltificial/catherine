# Change: Add the wasm boundary and the read-mostly web view
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Catherine must be browser-runnable (PROPOSAL §1: "openable in a browser
tab") — the human half of the same artifact the agents drive via CLI.
CatColab proves the Rust→WASM path (E1); catherine occupies the layer
below it: the embeddable kernel. The UI is a thin static viewer over the
wasm exports — never the source of truth (E5). Beads ticket:
`catherine-9b5` (P5; blocked by `catherine-1ay`; blocks `catherine-9vv` —
see design.md for the sequencing consequence).

## What Changes

- New crate `crates/catherine-wasm`: wasm-bindgen bindings over core +
  analysis — JSON in/out only, no logic of its own; row-count cap
  enforced at the boundary with a typed error (DoS floor for a browser
  tab)
- TS type definitions generated from the `model-document` JSON Schema
  artifact (single source; no hand-written drift)
- `web/` static site (plain TS + HTML + SVG, no framework, no Leptos/Yew,
  no server): load/save `.ct.json` files, SVG schema-graph rendering,
  findings panel grouped by kind with witnesses, JSON textarea as the
  only editing surface in v1
- Findings panel renders kind, epistemic label, and witness — the same
  taxonomy the CLI prints, so browser and terminal agree
- Everything after wasm/TS boundary stays out of the kernel: `catherine-*`
  kernel crates remain wasm-optional and genesis-free
- Openspec capability `wasm-ui`

## Impact

- Affected specs: `wasm-ui` (new capability; consumes `findings`,
  `model-document` via the wasm boundary)
- Affected code: new `crates/catherine-wasm` + `web/`; kernel crates
  compiled to `wasm32` target in CI (compile-only gate)
- Beads: implements `catherine-9b5`; unblocks `catherine-9vv` (P6 kata
  validation — the stop condition)
- Deliberate v1 limits: no editing UI beyond the textarea, no server, no
  persistence beyond downloaded files
