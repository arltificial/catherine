# Change: Add bounded analysis — search, counterexamples, redundancy, underconstraint
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

The kernel executes declarations on a *given* instance; the analysis layer
answers the questions agents actually ask mid-conversation: *can* these
constraints be satisfied at all (contradiction)? is this declaration
derivable (redundancy)? which parallel paths were never reconciled
(underconstraint — the discovery mechanism, kata-8 pattern)? where do two
models differ (distinguish)? All deterministic, all bounded, all
scope-honest. Beads ticket: `catherine-1ay` (P4; blocked by `catherine-86i`;
unblocks `catherine-9b5` and co-unblocks `catherine-hc6` with the CLI).

## What Changes

- Bounded instance search: enumerate small finite instances within
  explicit, caller-supplied bounds; outcome is a witness instance or
  `UNKNOWN {scope}` — never a naked "no violation found"
  (contradiction finding: bounds exhausted, no satisfying instance;
  zero-size bounds yield UNKNOWN, never a contradiction claim)
- Counterexample minimality: smallest violating instance within bounds
- Redundancy: path-equivalence via normal-form rewriting under declared
  equations → `redundant` findings (path, derivable-by path); also flags
  declared equations derivable from others
- Underconstraint detection: parallel paths (same endpoints, no declared
  equation between them) → `hypothesized` findings raising the domain
  question
- `distinguish`: bounded behavioral distinguishability of two instances
  over an explicit observation set (schema equivalence is undecidable —
  out of scope by name); witness input where they differ
- Query traversals ported from specodelic `query.rs` (264 LOC) behind the
  parity gate + property tests (E8)
- Openspec capability `analysis`; emits into the P2 taxonomy (no new
  finding kinds)

## Impact

- Affected specs: `analysis` (new capability; consumes `findings`,
  `kernel-checks`)
- Affected code: new `crates/catherine-analysis` (dependency direction
  `analysis → core`); CLI gains analysis commands
- Beads: implements `catherine-1ay`; unblocks `catherine-9b5` (P5) and
  co-unblocks `catherine-hc6` (P9)
- Honest limits carried forward: bounds are inputs; `UNKNOWN` is a
  first-class result (label `unknown` + scope), not a failure
