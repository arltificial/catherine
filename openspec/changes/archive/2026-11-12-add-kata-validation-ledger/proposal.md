# Change: Add the kata validation ledger — the project stop condition
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Every claim in the proposal is falsifiable, and this change is where it
gets falsified or confirmed. The ct-catlab series validated categorical
modeling's boundary on 10 katas (2026-11-02); catherine must reproduce
that boundary *with its own tool, without Julia* — or stop. The negative
control runs first: if catherine cannot say "modeling buys nothing here"
on FizzBuzz, it is a ceremonial math generator, not a workbench. Beads
ticket: `catherine-9vv` (P6; blocked by `catherine-9b5`; gates
`catherine-alr` and `catherine-nq5`).

## What Changes

- Port the three validated katas as browser fixtures: **Bowling**
  (temporal semantics — expected partial: history-dependence is a
  validated CT limit), **Gilded Rose** (state-shape invariants),
  **Vending Machine** (strongest validated structural result, 6/6)
- **Negative control first**: FizzBuzz fixture must produce the verdict
  "no categorical advantage" — recorded before any kata that could
  succeed (no goalpost moves)
- Validation ledger: entries with verdict vocabulary `killed | survived |
  inconclusive | invalid-run`, content-addressed via canonical-JSON +
  sha256 (P0 hash port), ct-catlab record discipline applied to the tool
  itself
- Success criteria (per kata): validated findings reproduced as executed,
  witness-backed checks, finding-typed, expressible as path equations +
  totality; the **expressibility gap vs full ACSet invariants is recorded
  in the ledger, not hidden** (DESIGN §4.1 known limitation)
- **Stop condition**: a `killed` verdict stops the project — write the
  negative result, keep the ledger; P7/P8 do not start
- Openspec capability `kata-validation`

## Impact

- Affected specs: `kata-validation` (new capability; consumes `wasm-ui`,
  `analysis`, `findings`)
- Affected code: fixtures under `fixtures/katas/`; ledger under
  `ledger/`; no kernel changes expected (kernel defects found by katas
  are bug fixes, tracked as beads, not scope growth)
- Beads: implements `catherine-9vv`; gates `catherine-alr` (P7) and
  `catherine-nq5` (P8)
- Source material: `ak/matadero/series/ct-catlab` (validated expectations,
  on disk)
