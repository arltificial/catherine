# Change: Add the bogart pilot — first real-domain dogfood in a live CI
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Katas are the lab; bogart is the first evidence **outside** it. Bogart
(`at/bogart`, Clojure) carries prose invariants in its openspec docs that
map directly onto kernel findings (DESIGN §5 pilot table: 5 mappings).
Checking real `.ct.json` models of a live repo in its CI demonstrates the
whole stack — kernel, findings, CLI, migration — on a domain nobody
modeled to flatter the tool, and demonstrates language-neutrality: a
Clojure repo's invariants, checked by a Rust kernel over JSON. Gated on
P3 (CLI) + P4 (findings), deliberately NOT on P6 — real-domain signal is
worth collecting in parallel with kata validation. Beads ticket:
`catherine-hc6` (P9; blocked by `catherine-86i` + `catherine-1ay`).

## What Changes

- Five pilot models as `.ct.json` in bogart's tree (per the DESIGN §5
  table):
  1. FTS-anchor equation (`block → document → edition = block → edition`
     + totality of `block → document`) → violation + dangling on drifted
     denormalized anchors
  2. Identity-domain disjointness (aviso vs section-acquisition id
     formats) as a rule-lens-style table → overlap/shadow on identity
     domains (P8 preview; spec-only if P8 not yet landed)
  3. Capture selector relation (`capture_ambiguous` /
     `capture_unavailable`) → overlap (multiple matches, no tie-break) +
     gap (no match)
  4. Raw working corpus vs derived serving projection → schema map + Δ
     migration with lost-data counterexamples (P7 shape; rehearsed in
     P7 task 8)
  5. Alerts tiered rules (FTS layer always runs; structured on
     identity-covered docs; `layer` records which fired) → tiered rule
     table + priority policy + declared coverage
- Bogart CI step: `catherine check` per model, exit codes per finding
  class (P3 contract), non-zero failing the build on violation/dangling
  classes
- **Phased landing**: the gate is P3+P4 (deliberately not P6), and each
  model lands only when its capability exists — models 1–3 at the
  P3+P4 gate; model 4 after P7 (`add-delta-migration`); model 5 after
  P8 (`add-rule-lens`); model 2 stays spec-only until P8, as annotated
  above. A phased pilot, not a hidden dependency.
- **Findings vocabulary**: pilot findings are reported under the P2
  taxonomy names — rule-lens overlap/gap results land as their mapped
  kinds (violation / underconstraint), per `add-rule-lens` design.md
  Decision 1.
- **Binary distribution**: bogart CI checks out catherine at a pinned
  rev and builds `catherine-cli` (cargo, cached target dir) — no
  crates.io publication for a private-repo tool.
- **Pilot report**: which findings fired on the live corpus, which were
  real defects vs noise (with the noise accounted honestly — the
  record-discipline requirement applies to us on our own tool)
- Findings confirmed as real defects flow back as bogart fixes; findings
  that are noise are recorded with cause (model imprecision, finding
  granularity) — never silently suppressed
- Openspec capability `bogart-pilot`

## Impact

- Affected specs: `bogart-pilot` (new capability; consumes `cli`,
  `analysis`, `findings`; optionally `migration`, `rule-lens`)
- Affected code: `.ct.json` models + CI wiring live in **bogart's repo**;
  catherine gains no new kernel code except defects found (beads, not
  scope growth)
- Beads: implements `catherine-hc6`; the terminal phase — after this,
  catherine is either earning its keep in a live repo or the ledger says
  so
