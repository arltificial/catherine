#+TITLE: Rule of 5 — value proposition & constraints of catherine (design review)
#+DATE: 2026-11-24
#+OPTIONS: toc:2

* Status
Evidence for the decision memo (~notes/decision-memo-2026-11-24.org~). Where
this review and the memo differ, the memo governs.

* Work reviewed
Artifact under review: the *value proposition and constraint set* of catherine as
articulated across ~README.md:5-31~, ~docs/PROPOSAL.md:88-155~, ~docs/DESIGN.md:64-86~,
~docs/USING.md:95-102,217-226~, ~docs/ref/expressibility-dispositions.md~, checked
against the shipped CLI (0.4.1).

Mode: universal Rule of 5 (advisory — the reviewed docs were not edited).
Prompt intent: ensure the design states a *clear, non-generic* value proposition
and constraints before any further capability proposals.

Validation: ~TYPESAFE_API_KEY~ was set, so all CRITICAL/HIGH findings went through
the TypeSafe verification pass (one parallel request, ~jev-latest~); results in
the verification table. MEDIUM/LOW findings are self-reported (UNVERIFIED).

* STAGE 1: DRAFT

Assessment: the design has an unusually disciplined constraint culture — the
limits are registered, executable, and CI-gated — but the *headline* proposition
is written in the vocabulary of any model checker, while the non-generic
differentiators live two sections lower. The constraint set is real but homeless:
three lists, no index, no trigger.

[DRAFT-001] [HIGH] - README.md:11-14 (one-sentence value proposition)
Description: The one-sentence proposition is "turns 'do these declarations
actually hold?' into an executed, witness-backed check". Every model checker,
schema linter and property-test harness can claim that. The two properties that
make catherine *not* generic are present in the same document but not in the
proposition: (a) loud lost-data counterexamples on schema migration
(README.md:25, PROPOSAL C3 "a capability gap conventional tooling cannot match"),
and (b) the executable limits registry (README.md:165-170) that makes "knows its
limits" a claim with a test suite behind it.
Recommendation: make the proposition name its *domain of claims* (referential
path agreement over finite, stored references) and its *one unmatched axis*
(schema evolution / data loss). Keep the generic clause as a subordinate clause
at most.

[DRAFT-002] [HIGH - REVIEW_REQUIRED] - PROPOSAL.md:88-94, DESIGN.md:77-86, USING.md:217-226
Description: The constraint set exists in three separate lists with three
different headings, three different granularities (feature list / deferred
non-goals / user-facing "does NOT do"), and no cross-index. Nothing states which
list is normative when they disagree (they do: see CORR-003, CORR-004). For a
project whose stated risk is overreach ("No universal ontology or multi-logic
translation framework", DESIGN.md:81), the anti-overreach list is itself
unanchored.
Recommendation: one normative constraints table (single file, e.g.
~docs/VALUE.md~ or a README section), every other list reduced to a pointer to
it, with each constraint tagged by *what it forbids* and *what it would unlock*.
TypeSafe verification confidence 0.77 (< 0.8 gate) — see verification table; my
mechanical evidence for it is a direct count of the three lists, but the
"homelessness" characterization is a judgment, not a fact.

[DRAFT-003] [MEDIUM] - PROPOSAL.md:126-140 (the alternatives comparison table — README.md has no such table) vs PROPOSAL.md:144-154
Description: The comparison table's differentiator column asserts "Knows its
limits: yes — negative-control verdict" for catherine and "no" for every
alternative. The negative control is real, but it is a *project process*
(ledger entries, release gates), not a product feature a user can observe in the
tool. In a table whose purpose is product comparison, that is an overclaim.
Recommendation: split the cell into "process: release-gated negative control"
and "product: executable limits registry (README.md:165-170)"; the second is
the row that genuinely differentiates.

Shape Quality: GOOD

* STAGE 2: CORRECTNESS

[CORR-001] [HIGH] - README.md:22, PROPOSAL.md:109-110 vs crates/catherine-analysis/src/search.rs:40-56
Description: The docs promise the search surface produces "the smallest breaking
instance" (README.md:22) and "the smallest instance that breaks the assumption"
(PROPOSAL.md:109-110). The shipped ~search~ surface does the opposite: its
~SearchOutcome::Witness~ is documented in the implementation as "The smallest
fully resolved instance *satisfying* every accepted equation". The breaking
instance is produced by ~check~ over a supplied instance, and by ~diff~ as a
separating witness (README.md:26 says this correctly). The docs conflate two
different surfaces.
Evidence (live): ~catherine search broken.ct.json --rows A=1 --rows B=2~ →
~outcome: "witness"~, witness = ~{"f":[0],"g":[0]}~ (satisfies the equation),
while the supplied instance violates it (~check~ on the same document → exit 3,
"violation: eq-0 f ≠ g").
Recommendation: reword README.md:22 to "bounded *satisfying*-instance search
(smallest model of the schema within bounds) — with ~diff~ supplying the
breaking/separating instance"; fix PROPOSAL.md:109-110 the same way.

[CORR-002] [HIGH] - README.md:30-31, USING.md:97-99 & 224-226, PROPOSAL.md:113-114, DESIGN.md:68-69
Description: The value proposition claims the *tool* says "model this
conventionally" on rule-based/trivial problems. It does not: the phrase exists
only in documentation, and the FizzBuzz negative control is silent by design
(exit 0, empty output). The behavior the proposition attributes to the product
is a human reading USING.md. This is the sharpest honesty defect in the design
because the honesty claim *is* the product.
Evidence (live): ~grep -rn "model this conventionally" crates/~ → no matches;
~catherine check fixtures/katas/fizzbuzz.ct.json~ → exit 0, empty stdout,
~data.findings=[]~, ~warnings=[]~; zero-row document likewise silent.
Recommendation: either (a) implement the verdict — the envelope already carries
~warnings~/~hints~ channels, and a document with zero executed constraints, or
a below-floor rule table, could emit "this document has no executable
constraint — model it conventionally"; or (b) restate the claim in the docs as
what it is: a *process* rule enforced by release gates. (a) is one small
warning; (b) is one sentence. Currently the docs assert a third thing (product
behavior) that does not exist.

[CORR-003] [MEDIUM] - README.md:27 vs PROPOSAL.md:62 & 94
Description: The rule lens' epistemic status contradicts itself across the
value-proposition documents: README.md:27 says "P8, executed; scoped to extracted
tables"; PROPOSAL.md:62 says "a pre-registered hypothesis (P8), not a claim";
PROPOSAL.md:94 lists it under "Post-v1, named now so scope is honest". The
PROPOSAL scope note (line 8-13) points to README as authoritative, so a reader
following the proposal is misinformed about the status of a shipped surface.
Recommendation: in PROPOSAL §3/§4, annotate the rule lens with its current
status ("shipped; scoped to extracted tables; hypothesis for authored tables")
and keep the scope note.

[CORR-004] [MEDIUM] - PROPOSAL.md:88-92, DESIGN.md:~55-62 vs README.md:~140-152
Description: The proposal and design describe v1 as "five executed/deterministic
finding types" and the CLI's exit table now has eight classes (0-8, including
lost data). The proposal's own scope note flags the drift, but the drift is
precisely in the section a reader treats as the definition of the product.
Recommendation: fold the exit-code table into the normative constraints document
and have PROPOSAL/DESIGN point at it.

[CORR-005] [MEDIUM] - PROPOSAL.md:145, DESIGN.md:68, THEORY.md:72
Description: The strongest claim in the proposal pivots on "state-shape domains"
— a term with eleven occurrences and zero definitions across README/docs.
Recommendation: define it once, concretely ("a domain whose claims are about
the *shape and agreement of stored references*, not about computation over
values"), and reuse the definition verbatim everywhere else.

Correctness Quality: GOOD

Convergence check (after Stage 2)
- New CRITICAL issues: 0
- Total new issues: 5 (stage 2) vs 3 (stage 1)
- New issues vs previous stage: +67%
- Estimated false positive rate (TypeSafe-measured): 0%
- Status: CONTINUE

* STAGE 3: CLARITY

[CLAR-001] [HIGH - REVIEW_REQUIRED] - PROPOSAL.md:145, DESIGN.md:68, THEORY.md:72
Description: The applicability criterion the whole boundary rests on
("state-shape domains") is never defined, so a reader cannot answer the
question the product exists to answer: *is my problem in scope?*
Recommendation: one definition, one place, reused by reference; plus a 4-question
self-check ("does your claim compare two stored paths? is it about rows
existing/being unique? does it survive a schema change?") that maps to the
surfaces that do answer it.
TypeSafe confidence 0.77 (< 0.8) — the factual half (11 uses, 0 definitions) is
mechanically verifiable; the "reader cannot answer" half is a judgment.

[CLAR-002] [MEDIUM] - README.md:30-31, USING.md:95-102
Description: "The honest boundary is part of the product" is a claim about a
*behavior* with no described mechanism, no invocation, and no output shape. A
reader looking for "how do I hear the boundary?" finds prose. Even after
CORR-002 is fixed by restating, the boundary needs an artifact: e.g. a
~catherine scope~ surface (or a ~notes~ field) that classifies a document
against the four applicability questions above.
Recommendation: give the boundary an interface, or state plainly that it is a
human rule with a release gate behind it.

[CLAR-003] [LOW] - README.md:28, PROPOSAL.md:65 & 232, USING.md:30
Description: The epistemic-label vocabulary varies: four labels in README.md:28
and PROPOSAL.md:232 (PROVED/EXECUTED/HYPOTHESIZED/UNKNOWN), five in
PROPOSAL.md:65 and USING.md:30 (adding ILLUSTRATED). ILLUSTRATED is a
*document* status, not a tool finding label, so the sets are not the same kind
of thing.
Recommendation: one table, two rows — tool labels vs document labels.

Clarity Quality: GOOD

Convergence check (after Stage 3)
- New CRITICAL issues: 0
- Total new issues: 3 vs 8 previous (+37.5%)
- False positive rate: 0%
- Status: CONTINUE

* STAGE 4: EDGE CASES

[EDGE-001] [HIGH - REVIEW_REQUIRED] - README.md:30-31, USING.md:97-99
Description: "Clean" is ambiguous in three ways that all render identically:
(a) a below-floor/rule-based document — nothing fires; (b) a constraint-free
document — nothing fires; (c) a zero-row instance — every equation is evaluated
over zero rows and never fires (registered gap ~vacuous-clean-instances~). In
all three, exit 0 + empty envelope. A user cannot distinguish "checked and
clean" from "nothing was checked", which is the exact failure mode the product
claims to prevent ("formal consistency cannot detect missing requirements").
Recommendation: emit an explicit scope signal on every run — a count of
executed constraints and rows evaluated (e.g. ~data.scope = {equations_checked:
n, rows_evaluated: m}~) so that "clean over zero rows" and "clean over 12 rows"
are distinguishable; a vacuity warning when an accepted equation covers no
row. This is the smallest change that converts the honesty claim from a
process promise into a product behavior.
TypeSafe confidence 0.24 (< 0.8 gate) — keep and escalate.

[EDGE-002] [MEDIUM] - README.md:22 & 26
Description: The two surfaces that can answer "is this schema satisfiable?" /
"did this edit weaken the model?" default to ~unknown~ unless the user supplies
bounds: ~search~ on the repo's own contradiction fixture returns "unknown —
explored 256 candidates"; ~diff~ on the ~weakened~ fixture returns "unknown —
budget expired after 0 candidates" unless ~--rows Node=2~ is passed (with it:
"weakened — a v1-enforced constraint is not enforced by v2"). The honest
non-answer is correct behavior, but the *first-contact* experience of the two
strongest analytical surfaces is a shrug with no hint about the missing bound.
Recommendation: derive default row bounds from the document's own instance
when the model carries one, and say in the ~unknown~ payload which bound would
have to be supplied.

[EDGE-003] [MEDIUM] - docs/PROPOSAL.md:88-94 vs README.md:165-170
Description: The registry of expressibility limitations is the design's most
distinctive asset and it is *not* part of the constraint list a newcomer reads
(PROPOSAL §3, DESIGN §2). The constraints say "what we will not build"; the
registry says "what we cannot express". Those are different constraints, and
only the second one is enforced by a test suite.
Recommendation: in the normative constraints document, link the registry as the
*executable* half of the constraint set — a capability that has no registry row
has not been stated.

Edge Case Coverage: GOOD

Convergence check (after Stage 4)
- New CRITICAL issues: 0
- Total new issues: 3 vs 11 previous (+27%)
- False positive rate: 0%
- Status: CONTINUE (no CRITICAL; the three new items are refinements of the
  honesty axis rather than a new class)

* STAGE 5: EXCELLENCE

[EXCL-001] [HIGH] - the proposition itself
Description: The value proposition should be re-anchored on what is not
generic. Draft (advisory wording, not applied):
  "Catherine checks *referential* claims about finite data — 'these two stored
   paths must land on the same row', 'this variant set is closed', 'this
   refactor loses no data' — and answers with a witness row instead of a
   boolean. On schema evolution it has no conventional counterpart: it proves
   data loss before you ship it. On everything else (rule logic, arithmetic,
   time, cardinality) it says what it cannot do, and the limits are themselves
   a tested registry."
Recommendation: land this as the one-sentence version in README.md:11-14, with
the generic clause demoted to a subordinate clause, and add the "referential
claims" qualifier to PROPOSAL §4.

[EXCL-002] [HIGH] - the anti-generic gate (new artifact)
Description: The design currently protects against overreach by *discipline*
(release criteria, phase map, negative controls). A machine-checkable gate would
be stronger and would fit the project's own method: every new capability must
name (a) which of the three axes it serves — referential agreement / schema
evolution / executable-limits — and (b) which registry row it closes or which
new claim it adds. A capability that serves none of the three, or that would
make the tool "work" on problems where the conventional route is correct, is
refused by default — exactly the treatment the rule lens received (pre-registered
hypothesis, scoped to extracted tables only).
Recommendation: add the gate as a short section of the normative constraints
document and enforce it in review, not in code.

[EXCL-003] [MEDIUM] - the capability backlog, re-filtered
Description: The evaluation's earlier ranked backlog must be filtered through
the proposition, otherwise the project drifts toward a generic checker. Filtered:
  - *Keep* (serve a named axis): identity path form (unlocks inverse/identity
    agreement laws — pure referential); vacuity diagnostic (honesty axis);
    ~migrate~ functoriality + lost-constraint checks (the unmatched
    evolution axis); default row bounds for ~search~/~diff~ (makes the honest
    unknown actionable).
  - *Conditional*: cardinality/existence predicates. These are state-shape
    claims ("one owner per order"), but they widen the constraint language
    toward general constraint solving. Only admissible once "state-shape
    domain" is defined (CLAR-001) and with an explicit triviality-floor test.
  - *Refuse for now*: a general value/multi-valued-cell type. That is a data
    model, not a kernel; it is the clearest route to "generic tool".
Recommendation: do not open the conditional bucket until the scope definition
lands; the two "keep" items at the top of the migration axis are the ones that
strengthen the proposition rather than dilute it.

Excellence Assessment:
- Structure: GOOD (constraint culture is strong; constraint *placement* is weak)
- Correctness: GOOD (two documentation/behavior mismatches, both HIGH, both fixable in text)
- Clarity: GOOD (one undefined pivot term, one missing interface)
- Edge Cases: GOOD (the "clean means nothing checked" ambiguity is the important one)
- Overall: GOOD

Production Ready: WITH_NOTES

* Final Report

**Work Reviewed:** design documents (value proposition & constraint set) — README.md, docs/PROPOSAL.md, docs/DESIGN.md, docs/USING.md, docs/ref/expressibility-dispositions.md
**Convergence:** Stage 5 (no CRITICAL at any stage; every stage surfaced a distinct class of issue; Stage 5 produced polish only)

**Summary**

Total Issues by Severity:
- CRITICAL: 0
- HIGH: 7 (DRAFT-001, DRAFT-002*, CORR-001, CORR-002, CLAR-001*, EDGE-001*, plus EXCL-001/EXCL-002 as recommendations)
- MEDIUM: 7 (DRAFT-003, CORR-003, CORR-004, CORR-005, CLAR-002, EDGE-002, EDGE-003)
- LOW: 1 (CLAR-003)
- REVIEW_REQUIRED (TypeSafe confidence < 0.8): 3 — DRAFT-002 (0.77), CLAR-001 (0.77), EDGE-001 (0.24)

**What's Working Well**

1. Executable limits registry — docs/ref/expressibility-dispositions.md:68-75 + ~just check-gap-claims~ + ~cargo test --test expressibility_gaps~ (6/6 green) — the only design in this space whose *limitations* are tested; this is the strongest anti-generic asset and it is not yet part of the constraint list (EDGE-003).
2. Pre-registered honesty discipline — PROPOSAL.md:62 (C2), the FizzBuzz release gate, the stop condition at DESIGN.md:71-75 — the project can kill itself, which is the opposite of generic drift.
3. Epistemic labels on every answer — README.md:28 — makes "I don't know" a first-class output instead of a failure mode.
4. Comparison against prior art with parity stated, not inflated — PROPOSAL.md:118-140, PROPOSAL.md:225-232 — "the differences are witness typing, ambiguity-as-output, bounded honesty, and packaging — not a claim of greater modeling power" (PROPOSAL.md:236).

**Top 3 findings**

1. CORR-002 — the tool does not say "model this conventionally"; the docs say it does — README.md:30-31, USING.md:97-99, 224-226, PROPOSAL.md:113-114, DESIGN.md:68-69
   Impact: the honesty claim is the product's differentiator, and it is currently asserted about behavior that does not exist; a reader who trusts it will sit waiting for a verdict the tool never emits.
   Fix: implement the verdict as a ~warnings~/~hints~ line (a document with no executed constraints or a below-floor table says so), or restate it as a process rule. Do not leave the docs asserting product behavior.

2. DRAFT-001 + CLAR-001 — the headline proposition is generic ("do these declarations actually hold?"), and the scope pivot term "state-shape domain" is never defined — README.md:11-14, PROPOSAL.md:145, DESIGN.md:68
   Impact: the design's own anti-generic discipline is not visible in the sentence a newcomer reads first; and nobody can place their own domain inside or outside the boundary.
   Fix: rewrite the one-liner around referential claims + the unmatched evolution axis (EXCL-001 wording), and define "state-shape domain" once.

3. EDGE-001 — "clean" is indistinguishable from "nothing was checked" in three distinct ways (below-floor, constraint-free, zero-row) — README.md:30-31, USING.md:97-99
   Impact: directly contradicts the documented truth that "formal consistency cannot detect missing requirements"; the tool's silence is doing the opposite of the honest-boundary claim.
   Fix: put an execution scope on every answer (constraints checked, rows evaluated) and warn on vacuity.

**Stage-by-stage quality**
- Stage 1 (Draft): GOOD — discipline strong, headline weak
- Stage 2 (Correctness): GOOD — two HIGH doc/behavior mismatches
- Stage 3 (Clarity): GOOD — one undefined pivot term
- Stage 4 (Edge Cases): GOOD — the "clean means nothing checked" ambiguity dominates
- Stage 5 (Excellence): GOOD — the sharpening is two sentences of proposition plus one gate

**Recommended Actions**

1. Fix the two HIGH doc/behavior mismatches (CORR-001 search semantics, CORR-002 "model this conventionally") — text-only, no code.
2. Rewrite the one-sentence value proposition around referential claims + the lost-data axis (EXCL-001), and define "state-shape domain" once (CLAR-001).
3. Create one normative constraints surface (DRAFT-002) that indexes PROPOSAL §3, DESIGN §2, USING "does NOT do", *and* the expressibility registry (EDGE-003), with the anti-generic gate (EXCL-002).
4. Put execution scope on every answer (EDGE-001) — smallest code change with the largest honesty payoff.
5. Re-filter the capability backlog through the gate (EXCL-003): keep the identity-path, vacuity, migrate-functoriality and default-bounds items; park cardinality/existence until the scope definition lands; refuse the general value type.

**Verdict**

READY (for the docs) / NEEDS_REVISION (for the two HIGH mismatches before the proposition is quoted externally)

**Rationale:** The design's constraint culture is stronger than most projects'
and its limits are executable — the thing that keeps this tool from being
generic already exists. What is missing is that the *headline* says "generic
model checker" and the *boundary* is asserted as tool behavior it does not have;
both are small, text-and-one-warning fixes, not redesign.

* Escalation note (protocol)
Three HIGH findings fell below the 0.8 TypeSafe confidence gate (DRAFT-002 0.77,
CLAR-001 0.77, EDGE-001 0.24) and are marked REVIEW_REQUIRED: their factual
halves are mechanically verifiable (three lists; eleven uses and zero
definitions; three silent exit-0 runs), but their characterizations are
judgments. They are routed for human ratification rather than counted as
settled.

