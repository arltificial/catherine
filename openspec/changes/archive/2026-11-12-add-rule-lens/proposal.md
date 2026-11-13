# Change: Add the rule lens — rules as subobjects with a resolution policy (HYPOTHESIZED)
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

kata-6 killed the *schema-lens* rule table at the triviality floor, but
did not refute categorical structure in rule systems *above* the floor.
The rule lens models rule tables directly — rules as subobjects of the
input domain, action as a required total function, resolution policy as
order-enforcement — turning rule-table bug classes (gap, overlap, shadow,
derivable) into exact findings, and rule-table simplification into
provably-equivalent rewrites. **Status: HYPOTHESIZED** — no
ct-catlab-class evidence exists; precedent is Rust match exhaustiveness
and DMN hit-policy conflict detection. This change pre-registers its own
validation-or-death protocol. Gated on P6 `survived` *and* the P6
schema-lens negative control recorded first. Beads ticket:
`catherine-nq5` (P8; blocked by `catherine-alr`).

## What Changes

- `RulePresentation` kernel types: input domain X, rules (decidable
  predicates over attribute columns + action + order), policy
  (`PriorityOrder | FirstMatch | DeclaredDisjoint`)
- Attribute columns + tiny decidable predicate expression language
  (int/bool/string comparisons, ∧ ∨ ¬, mod) — the DESIGN §8.1 resolution;
  additive format extension
- Effective-region computation `E_i = R_i \ ⋃_{j<i} R_j`; findings:
  **gap** (⋃R_i ⊊ X), **overlap** (R_i ∩ R_j ≠ ∅ without policy),
  **shadow** (E_i = ∅), **derivable** (R_k = R_i ∩ R_j) — each mapped
  onto the existing finding taxonomy (see design.md), no new epistemic
  machinery
- Equivalence checking: two rule tables equivalent iff same total action
  on X; `distinguish` (P4) supplies the differing witness — simplification
  proven, not asserted
- Rule discovery support: given input→output examples + declared
  predicate vocabulary, candidate tables (from agent/human) are checked
  for partition equivalence; uncovered examples are gap findings
- For enumerable X: every finding exact via P4 bounded-search machinery —
  a presentation over the same kernel, not a new engine
- **Simulation before Rust** (wild `prototype/wild_cat_sim.py` pattern):
  Python sim of the laws de-risks semantics before kernel code
- **Pre-registered validation**: one kata above the floor (10–20 compound
  rules + priority policy, e.g. pricing/eligibility) + one near-floor
  negative control; killed honestly if unique detections don't appear
  above the floor
- Openspec capability `rule-lens`

## Impact

- Affected specs: `rule-lens` (new capability; consumes `analysis`,
  `findings`, `model-document`)
- Affected code: `crates/catherine-core` (columns, predicates),
  `crates/catherine-analysis` (enumeration-backed findings); additive
  format extension; CLI surface
- Beads: implements `catherine-nq5` (P8, HYPOTHESIZED); the last kernel
  capability — P9 pilot may consume it
- Kill switch: if the above-floor kata shows no unique detection, the
  ledger records `killed` for the rule lens and the feature is documented
  as a negative result — scope retreat, not scope creep
