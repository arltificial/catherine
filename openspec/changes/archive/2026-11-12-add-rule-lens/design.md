# Design: add-rule-lens

## Decision 1 — findings map onto the existing taxonomy (the mapping, fixed)

§4.4 says rule-lens findings "map onto an existing finding kind"; the
mapping is pinned here so P2's enum is never broken:

- `gap` → **underconstraint** variant (a declared-totality domain with an
  uncovered input class is an underconstrained totality claim),
  `hypothesized` until totality is declared
- `overlap` → **violation** of the declared policy when the policy is
  `DeclaredDisjoint` (a declared constraint failing on concrete rows);
  for `PriorityOrder`/`FirstMatch`, overlap is legal by construction and
  is *not* a finding — shadow/derivable carry the consequence
- `shadow` → **redundant** (rule i dead: derivable from earlier rules +
  policy; witness is the effective region proof)
- `derivable` → **redundant** (boolean-combination derivability, witness
  `by` the combining rules)

Consequence: no new Finding variants; the rule lens is a producer over
the P2 taxonomy — the "no new epistemic machinery" claim is enforced by
construction.

## Decision 2 — the predicate language is the attribute-alphabet answer

DESIGN §8.1 resolved the attribute question via this capability: columns
are int/bool/string; predicates are a tiny decidable expression language
(comparisons, ∧ ∨ ¬, mod) — deliberately not a logic, not pluggable
strategies. String comparison is byte-wise (UTF-8, locale-independent,
deterministic) — no collation. Enumerable-domain exactness depends on
decidability; every extension to the language must re-justify it.

## Decision 3 — policy semantics are total and conventional

`PriorityOrder`: lowest index wins on the intersection.
`FirstMatch`: rule order as written wins. `DeclaredDisjoint`: intersections
are violations (see Decision 1). Effective regions are computed under the
chosen policy; no implicit policy exists — a table without a declared
policy gets gap/overlap analysis only, never silent precedence.

## Decision 4 — validation is two katas, pre-registered, killable

Above-floor kata: 10–20 compound rules + priority policy where effective
regions are humanly untrackable (pricing or eligibility table from the
bogart/corpora domain). Near-floor negative control: a small table where
trivial tests suffice — expected verdict "no advantage". Unique-detection
failure above the floor ⇒ ledger `killed` for the rule lens; the code
stays (it is correct, just not valuable there) and the negative result is
the deliverable. Order: negative control may run after the above-floor
kata here (unlike P6) because the hypothesis under test is specifically
about above-floor behavior — the protocol is pre-registered in this file
before any kata runs.

## Decision 5 — Python sim first

The wild `prototype/wild_cat_sim.py` pattern: implement effective-region
and equivalence laws in Python against hand-computed tables before Rust.
Sim/kernel divergence is a bug in one of the two; hand-computed tables
are the oracle. De-risks the only genuinely new semantics in the change.
