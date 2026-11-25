# kw9 wave report — ct-math-foundations on catherine's three axes

**TL;DR:** Epic `catherine-kw9` (closed 2026-11-25) turned the cotorreo
`ct-math-foundations` wave into licensed probes and executed them. Verdict:
catherine's modeled lens earned **no unique detections** over a conventional
script on the one real domain tested — fresh-domain probe and three-arm bench
both **killed**. The wave's research verdict (CT pays in 3 niches only) stands,
and catherine now holds its own measured null data point where the literature
has none.

This topic covers the wave's conclusions, per-probe verdicts, and where every
claim is evidenced. You should understand catherine's verdict-only boundary
(findings + witnesses + epistemic labels; no persistence/claims stores) and
the domain-probe ledger discipline (RED pre-registration, floor control,
executed baseline).

## Wave verdict (research side)

Source: cotorreo `raw/ct-math-foundations/` (one dispatch prompt, five
conversations, 2026-11-24/25) + `WAVE.md` synthesis rules.

Convergent provider verdict — category theory pays in **three niches only**,
not as a general foundation:

| Niche | Mechanism | Tooling |
|---|---|---|
| Data migration / integration | functorial migration checks a schema mapping preserves constraints | CQL |
| Model composition via interfaces | operads + decorated cospans prove how components wire | AlgebraicJulia |
| Design checklist | "what is the composition law, the identity, what is preserved?" | none needed |

Elsewhere simpler tools dominate: accounting = group + constraints; workflows =
state machine + model checker; legal = defeasible rules over bitemporal data.

Two caveats bind every use of this verdict:

1. **Singly-sourced consensus** (WAVE.md rule 1): providers share a canon
   (Spivak/CQL, Fong–Spivak, Milewski, Jacobs); agreement is not independent
   evidence.
2. **No controlled comparison exists in the literature** of categorical vs
   conventional implementations. Claude's report labels this UNKNOWN.

## Executed verdicts (catherine side)

| Slice | Ticket | Verdict | Evidence |
|---|---|---|---|
| 6 wave findings → CLI verbs + fixtures + registry rows | kw9.2/.3/.7/.8/.9/.10 | landed | `b7eaf82`; 135 suites green |
| Granularity lint vs bogart pilot | kw9.4 | **killed** — lint silent at pilot scale (cap 13 > 3 arrows); ticket's predicted flagging falsified, recorded as discrepancy; rule stays PROPOSED | `3a0b37c`; `ledger/probes/granularity/0002` |
| Fresh-domain precision probe (U1–U2), selector-state domain | kw9.5 | **killed** — 602-row collision slice fires zero findings; the 3 known-real selector collisions are inexpressible on the check surface (no cardinality claim) | `c1b7cb3` (RED) + `39bc444` (GREEN); `fixtures/probes/selector-state/README.md` |
| Three-arm bench protocol | kw9.6 | pre-registered, 8-row expectations, arithmetic uniqueness | `07a1f97`; `fixtures/probes/bench/README.md` |
| Three-arm bench run | kw9.11 | **killed** (parity null result) | `1a719aa`; `crates/catherine-ledger/ledger/probes/bench/selector-state/000{1,2}-*.json` |
| Artifact capture (claude report body) | kw9.1 | done — captured by the catitas hourly sync, no HITL needed | corpus `raw/_conversations/claude/00831713-*/artifacts/` |

## Bench result (the differentiation demo)

Pre-registered on the selector-state domain (bogart census slice; the kw9.5
verdict made it the informed pick: known real defects, real rows, custody
recorded). Arms per claude's §10 protocol shape — every experiment gets an
**opponent** built without seeing the formal model:

| Arm | TP | FP | Outcome |
|---|---|---|---|
| A — conventional census script (`prototype/bogart_selector_bench.py`) | E1–E3 (3/3) | 0 | floor 0/127; S1 → 4 collisions |
| B — committed model battery (`catherine check`) | ∅ | 0 | silent on all E-rows; fires only seeded S2 |
| C — LLM-drafted model (kw9.9 recipe) | ∅ | 0 | loop converged at draft-v1; S2 + overconstraint controls fire |

Unique detections vs arm A: none, from either modeled arm. Decision-rule row 1
(parity) → **verdict killed**; the categorical claim stays UNKNOWN-labeled.
Sanity: S1/S2 fire where required, floor silent, no E6 FizzBuzz alarm.

### Disclosures (recorded in ledger 0002 and the bench README)

- Arm C's drafter was this session's model, not an independent session; it had
  read the slice earlier while building arm A, so the draft was not strictly
  prose-blind (schema drafted from the prose block only; custody diff clean).
  A clean re-run needs a fresh-context drafter for arm C only; protocol and
  fixtures are in place.
- Five discrepancies recorded: S1 pair shape, exit-1-vs-3 dangling
  registration, un-filled pre-registration commit hash (artifacts pinned by
  sha256 instead), selector-set derivation from capture/partition columns,
  drafter disclosure.

## What the null result does and does not establish

Does: on one bounded real domain, with an executed conventional baseline, the
model-check lens produced no finding the script missed; two known-real defect
classes are inexpressible on the check surface.

Does not: prove catherine's lens is useless — one domain, one defect family;
the rule-lens/policy machinery (where the selector-collision class actually
lives) was out of scope for this epic and remains the natural next target.

## Pointers

- Wave synthesis and rules: cotorreo `raw/ct-math-foundations/WAVE.md`,
  `verification.md`; distilled per-provider reports in
  `distilled/ct-math-foundations/`.
- Claude's full report (§§0–12, 10-experiment protocol): corpus artifact
  `2GmVcWwdLR5fYEuKii1cnc`, canonical file sha256 `32b67d1311cd2be3` (a
  truncated `-2` regen, sha `26bf79b676cc60da`, ends at §8 — cite the first).
- Expressibility dispositions (kw9.10 rows): `docs/ref/expressibility-dispositions.md`.
- Probe precedents: `fixtures/probes/*/README.md` (RED/GREEN discipline,
  `openspec/specs/domain-probes/spec.md` for verdict ceilings).
