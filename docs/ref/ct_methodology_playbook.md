# Category-theoretic modeling playbook — by problem type, no tool required

Status: v1 (2026-11-11) · audience: anyone who ships schemas, specs, or
integrations — engineer, analyst, or agent. Drills run ~30–90 minutes each;
the modeled attack costs 1–3× a conventional review (measured — budget for
it up front, see §7). Tool-free companion to
`category_theoretic_modeling_methodology.md` (the method) and
`../../THEORY.md` (the mathematics). Everything here works with paper, a
text editor, and an adversarial reader. Where a deterministic checker would
automate a step, the drill below says so — the discipline is what matters,
not the automation.

**The one rule this playbook inherits** (methodology doc): *use the smallest
mathematical structure that produces falsifiable design insights; elegance
by itself is not evidence of value.* And its shadow rule: for rule-heavy
logic — parsing, formatting, scoring — stop. Model it conventionally. This
is a measured boundary, not a taste (see §9).

## 0. The manual discipline (read once, use everywhere)

Without a tool, four habits replace enforcement:

1. **Witnesses, always.** Every claim gets a concrete row or path that proves
   it. "This equation is violated" is not a finding; "row 3: staff s1 in
   Unit A, equipment e3 in Unit B" is. Hand-computed witness = a derivation,
   labeled PROVED (with the arithmetic shown) — never claim EXECUTED unless
   a program actually ran it.
2. **Label everything** (methodology taxonomy): PROVED (assumptions +
   derivation shown) · ILLUSTRATED (example) · HYPOTHESIZED · UNKNOWN.
   Violations, contradictions, and redundancy can be *proved* by hand.
   Under-specification and missing requirements can only be *hypothesized* —
   they are domain questions. Never blur the two.
3. **The adversarial read.** The deterministic evaluator you do not have is
   substituted by an independent cold read: hand the model + the requirements
   to another person or agent with zero context and the instruction "attack
   this model: find the instance that breaks it, the question it cannot
   answer, the constraint it makes impossible." Dispatch to two independent
   readers; findings both miss are UNKNOWN, findings only one finds are
   candidates for dispute. (This is the propose/evaluate split done with
   people.)
4. **The findings ledger.** Every modeling session ends with a table:
   claim | evidence type | derivation or witness | counterexample | verdict.
   Keep it in the repo next to the model. It reviews like code.

**An empty ledger is a result, not a failure.** A disciplined pass that
yields no findings means the spec was sound — record that and move on.
The source experiment series pre-registered it: "a negative or inconclusive
verdict is a valid, expected outcome." Do not manufacture findings to
justify the session; manufactured findings are the failure mode, not the
empty ledger. When adversarial readers disagree, the owner arbitrates with
a witness row — disputed entries land in the ledger as HYPOTHESIZED,
never silently dropped.

## 1. Requirements & spec review — before anything is built

**Use when:** a spec, PRD, or set of user stories must become a design, and
you suspect the seams between assumptions.

**Drill:**
1. Freeze the baseline: restate the behavior with examples and invariants in
   plain language. Record ambiguities as UNKNOWN. No formalism yet.
2. Draw the semantic graph: entities as objects, functional relationships as
   arrows (many-to-many? introduce a relationship object — arrows are
   functions). Separate identities, events, states, derived quantities.
3. **Parallel-path inventory — the core move.** List every pair of paths
   with the same source and target. For each, ask exactly: *must they agree,
   might they agree, or should they differ?* Every "must" becomes a declared
   equation; every "might" becomes a recorded domain question; every
   "should differ" is a warning not to equate.
4. Hand-chase each declared equation on 3–5 concrete rows, including one
   deliberately awkward row. Arithmetic shown. A false equation rules out
   legitimate states — expect to withdraw at least one.
5. Adversarial read (habit 3). Ledger (habit 4).

**What you get:** the spec's unwritten decisions, surfaced as questions with
counterexamples — the ambiguity discovery effect. What you do *not* get:
proof that no requirement is missing (see §9).

**Micro-example:** "An assignment's staff unit must equal its equipment's
unit" (Johnson & Dampney's running example). Paths: `staffUnit ∘ staff` and
`equipmentUnit ∘ equipment` from Assignment to Unit. Must they agree?
The requirements never said — the equation makes cross-unit assignments
impossible, and the awkward row (temporary delegation, shared equipment)
forces the domain decision *before* code exists.

## 2. Data models & schema design — databases, JSON shapes, API payloads

**Use when:** designing tables, document shapes, or message contracts where
reference integrity and identity matter.

**Drill:** graph first (tables/classes = objects, foreign keys/lookups =
arrows), then declare the equations the invariants claim, then hand-chase.
Add only justified structure: a pullback means "pairs whose images agree"
(compatibility); a monomorphism means uniqueness — in v1-of-your-head, write
these as explicit named constraints with a valid and an invalid example each
(methodology step 3). For exhaustive alternatives, say so explicitly ("Order
is exactly one of: draft, placed, cancelled") — undeclared exhaustiveness is
where invalid states breed.

**What you get:** invalid states rejected *at design time* by inspection —
every future client inherits the constraint. The discipline: for each
declared equation, write down one valid instance and one instance that
breaks it. If you cannot write the breaker, you have not understood your own
constraint.

**Micro-example (real):** "every FTS hit resolves to a text block carrying
edition and page anchors" becomes `block → document → edition =
block → edition` plus totality of `block → document`. Hand-chase on a
denormalized row that drifted: both sides computed, mismatch = violation
with witness — the same finding a checker would emit, derived by hand.

**Granularity rule (PROPOSED — not yet validated on a second domain).** Model
one arrow per relation that exists as a **stored, independently writable
reference**; do not encode relations that exist only in prose, plans, or
derived views. Pre-build test: for every arrow you intend to declare, name
the storing column (or table) it reads. If more than a third of the arrows
have no storing column, you are modeling the plan, not the system — stop and
re-scope before spending the check budget. The one-third threshold is
arbitrary, calibrated against the bogart pilot's 2/69 finding triage; treat
it as a smell threshold, never as a validated cutoff. The rule is
**PROPOSED**: it has not been tested on a second domain.

## 3. State-shape invariants — what must be true whenever the system is running

**Use when:** any stateful system (orders, ledgers, reservations, sessions)
and you want the valid states stated once, not enforced per-code-path.

**Drill:** states as objects, observations/attributes as arrows out of the
state. For each invariant ("a shipped order always has a carrier and a
ship-date"), ask: is this a *path equation*, a *totality* (arrow defined for
all rows), or a *disjointness*? Write each as a checkable claim with a
violating witness. Hand-chase the awkward state (partially-refunded order).

**Boundary:** history-dependent rules ("score so far", mid-game bonuses)
are a validated expressibility *limit* — a state shape cannot see history.
Record them as a gap in the ledger; do not contort the model.

## 4. Which quantities compose? — metrics, aggregations, reporting

**Use when:** someone says "just sum the per-item metrics" and the metric is
not additive (percentiles, VaR, ratios, deduplicated counts), or when a
reporting layer must combine per-team numbers into a company number.

**Drill:** for each metric, classify its composition behavior —
*additive* (sums across any partition), *monoidal-with-structure* (sums with
carrying: weights, pairs), or *emergent* (defined only on the whole:
percentiles, VaR, distinct counts). (For the curious: additive = sum over
any partition; the middle class is a monoid — e.g. an average is the pair
(sum, count), merged pairwise; the last class has no merge at all.) The
classification is the deliverable:
it tells you which aggregations can be computed per-partition and merged
(safe to parallelize) and which must be recomputed globally (unsafe to
parallelize — the bug where distributed percentiles are averaged). Attach a
two-row witness where the naive merge gives a wrong answer; that witness
ends the argument.

**What you get:** modularity decisions (which metrics live on the part, which
on the whole) with proofs instead of vibes. This is a *composition* question,
the tool-free heart of "classify metrics categorically."

## 5. Evolution & migration — schema changes, data migrations, compatibility

**Use when:** changing a schema, merging two datasets, deprecating a field,
or versioning a contract.

**Drill:** write the map between old and new explicitly (what goes where).
Then answer, in writing, per field: **lost** (present before, absent after —
name every such field), **invented** (new, no source), **duplicated**, and
**invertible?** (can you go back?). The lost-data list *is* the deliverable —
it is exactly the counterexample list a migration checker would emit
(kata-10's verdict: conventional tooling has no counterpart). Then: is the
old→new map structure-preserving (equations preserved), and does the
migration preserve meaning or merely format? "A canonical mathematical
migration is not necessarily an acceptable business operation" — say which
you did.

**Rule:** accretion is safe-ish (constraints only added); every dropped
constraint needs a named reason in the ledger. An unexplained weakening is a
finding against the editor, whoever it is — human or agent.

## 6. Integration & alignment — two systems, two vocabularies, one workflow

**Use when:** connecting systems whose entities "obviously" match but were
designed apart (status enums, identity formats, temporal semantics).

**Drill:** never merge on string equality. Write a small shared interface
object: for each shared concept, a *crosswalk row* — identity scheme,
predicate meaning, authority, scope, temporal semantics (valid-time vs
known-time are two dimensions; do not conflate), permissions. Then decide
pushout (true merge, only if the crosswalk is agreed and evidence-bearing)
or span (keep both, translate at the edge — the default). Composition rule:
an adapter describes what it can *preserve*, not its JSON shape.

**What you get:** the integration contract as a checked artifact instead of
a hunch that the two `status` enums match.

## 7. Auditing a proposed design (including LLM-generated ones)

**Use when:** a design lands from anyone — colleague, contractor, agent —
and you must review it, not admire it.

**Drill (attack the model, in order):**
1. Violating instances: construct the awkward row for each equation.
2. Unexpected consequences: derive what the equations *imply* (compose them);
   is anything now impossible that the domain needs?
3. Redundancy: derive each declared path from the others; what is eliminable?
4. Under-specification: two materially different valid interpretations?
   (the adversarial read finds these)
5. Missing relationships: for each real example the domain produces, is it
   representable? Unrepresentable example = missing concept (hypothesis —
   needs a domain decision).
6. Falsifiable exit: "smallest next experiment or justified stopping
   decision" (methodology §required-answer 7).

**Calibration:** a competent conventional review finds the same bugs in
total; the modeled attack finds *structural* ones earlier (impossible
specs, silent constraint loss, non-compositional metrics) and — measured —
costs 1–3× more effort (E4). Budget accordingly: spend it on the seams
(evolution, integration, agent-generated designs), not on CRUD screens.

## 8. Reverse-engineering a system that already exists

**Use when:** the artifact is on disk — code, schema, data, docs — and you
need its inconsistencies and gaps, not a redesign. Model it from **both
directions** and diff: the divergences between the two models are the
findings. Neither direction alone finds them.

**Bottom-up (from the artifact).** Extract objects and arrows from what
*is*: tables/classes → objects; foreign keys, lookups, function calls →
arrows; unique constraints → declared equations; NOT NULL → totality; check
constraints → equations. Include behavior the code performs but no one
documented ("failed payment returns the order to pending" is an arrow in
the state graph). One extraction table per object: arrow | from | to |
total? | unique? | witness (file:line). Every extraction is labeled PROVED
— with the witness, or it isn't in the model. Then hand-chase each candidate
equation against **real rows pulled from the actual data** — invented rows
only find the violations you already imagined.

**Top-down (from the claims).** The docs, READMEs, wikis, type names, and
prose say what the system is *supposed* to be. Model those claims the §1
way (freeze baseline, semantic graph, parallel paths). Every claim is
HYPOTHESIZED until a bottom-up witness confirms it.

**Reconciliation — where the findings live.** Diff the two models,
direction by direction:

| Divergence | Finding class | Witness |
|---|---|---|
| top-down claims it, bottom-up lacks it | **phantom requirement** (spec'd, not built) | the missing FK / unimplemented branch |
| bottom-up has it, top-down lacks it | **undocumented behavior** (built, not spec'd) | file:line — often the most valuable findings: the system knows something the docs don't |
| both have it, shapes differ (endpoint, totality, equation on one side only) | **inconsistency** — strongest class | the row that satisfies one model and violates the other |
| both agree | confirmed core — anchor here | any real row |
| neither has it, domain needs it | **missing concept** (§1's question machinery) | the unrepresentable real example |

**Direction order:** bottom-up first when data exists (the artifact cannot
lie about what it stores); top-down first when the system is spec-rich and
code-light — there the top-down model audits *internal consistency of the
claims themselves* (a spec can contradict itself before a single line of
code exists; that audit once found several contradictions in a 3,500-line
spec whose implementation was two stub classes). Both directions converge
on the same reconciliation table.

**Boundary:** model a frozen revision — pin the commit. A codebase mid-
refactor is a moving bottom-up target; expect the model to expire and date
it.

## 9. When NOT to use any of this

- **Rule-heavy logic** (parsing, scoring, pricing below ~a dozen compound
  rules, formatting): killed at a 7-line conditional in a controlled
  comparison — the checks fired, the cost didn't. Conventional: tables +
  tests. Exception: a large rule table *with a priority policy* — then the
  four-region drill earns its keep: list rules as predicates over the
  input, mark regions: **gap** (no rule fires), **overlap** (two fire, no
  policy), **shadow** (dead rule), **derivable** (boolean combination).
  These four are findable by hand on an enumerable input and are the bugs
  that survive testing.
- **History-dependent behavior** (see §3 boundary).
- **When a test suite already pins the behavior.** Modeled checks never
  out-detect a competent conventional suite; they move detection earlier and
  expose structure. If the tests exist and are trusted, the marginal value
  is the *questions*, not the checks — decide if that's worth an afternoon.
- **When nothing will evolve or integrate.** The durable payoffs are
  evolution (§5) and composition (§4/§6). A one-shot script with no seams
  wants types and tests, not a model document.

## 10. The stopping rule

Stop extending the model when additional structure yields no new discovery,
clarity, or guarantee. Concretely: three consecutive ledger entries that say
"no new finding" means the model is done; record what remains UNKNOWN and
ship. Complexity added past that point is decoration, and decoration is the
failure mode this whole discipline exists to prevent. And the symmetrical
rule: if the very first pass finds nothing, close the ledger and do
something else — that too is the methodology working.

## Lineage

Method: Johnson & Dampney (commutative diagrams in information modelling —
the assignment example, §1), Johnson & Rosebrugh (sketch data models; view
updatability — the determinate-meaning question, §5/§6), Johnson (categorical
logic — the anti-overengineering principle, §9/§10), Spivak (migration as
Δ/Σ/Π — hand versions in §5), Patterson/Lynch/Fairbanks (ACsets — what a
checker would automate). The reverse-engineering drill (§8) descends from a
bottom-up extraction of a real Rust/SQLite codebase (observed facts separated
from proposed extensions; missing constraints reported as findings) and a
top-down pilot that ported a live repo's prose-spec invariants into checked
models. Full anchors: `../../THEORY.md` §9. Validated boundary and effort
costs: `ak/matadero/series/ct-catlab/REPORT.md`.
