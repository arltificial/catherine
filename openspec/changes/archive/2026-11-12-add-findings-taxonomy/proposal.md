# Change: Add the findings taxonomy — typed enum with witnesses and epistemic labels
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Catherine's output contract with agents and humans is *finding-typed*
diagnostics, not booleans: each finding names its kind, carries the
concrete witness that proves it, and states how it was established
(epistemic label). P1 produces raw check outcomes; without a stable
taxonomy the CLI (P3) has nothing honest to print and the analysis layer
(P4) has nothing to emit into. Beads ticket: `catherine-1x7` (P2; blocked
by `catherine-0gk`, blocks `catherine-86i`).

## What Changes

- Public `Finding` enum for v1 — violation, dangling, redundant,
  contradiction, underconstraint — each variant carrying its witness
  payload (equation + row + path values; arrow + source row; derivable
  path + reducing path; scope; domain-question witness as a tagged union:
  `ParallelPaths{lhs, rhs}` or `UncoveredInputs{class, examples}` — the
  latter reserved for the P8 rule-lens gap mapping)
- Epistemic label on every finding: `executed` (kernel-produced),
  `hypothesized` (proposed-equation failures, underconstraint), `unknown`
  (scope-bounded non-answers) — PROVED/ILLUSTRATED labels reserved for
  later layers, never fabricated by the kernel
- Stable machine-readable finding encoding (JSON, serde) — the shape
  `catherine findings` (P3) and the wasm boundary (P5) print verbatim
- Golden fixtures per finding type, asserted structurally (witness
  fields), never byte-compared
- Findings are ordered deterministically (by kind, then witness ids) so
  repeated runs are diff-stable
- Openspec capability `findings`

## Impact

- Affected specs: `findings` (new capability; depends on `kernel-checks`)
- Affected code: `crates/catherine-core` (new `findings` module; check
  outcomes re-shaped into findings)
- Beads: implements `catherine-1x7`; unblocks `catherine-86i` (P3 CLI)
- Deliberately out of scope: computing redundant/contradiction/
  underconstraint (P4), human/JSON rendering (P3)
