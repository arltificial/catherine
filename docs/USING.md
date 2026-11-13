# Using catherine — agents, humans, and the honest boundary

Status: usage guide v1 (2026-11-11) · audience: anyone about to point catherine
at a problem — human, or an agent with a shell. Companions: `PROPOSAL.md`
(why), `THEORY.md` (the mathematics), `ref/category_theoretic_modeling_
methodology.md` (the modeling procedure), `DESIGN.md` §5 (validation plan).

> **Epistemic status of this document.** The runs in §Probe evidence are
> **EXECUTED** — captured from the shipped CLI on 2026-11-13 and cited to
> the ledger entries they produced. The worked equipment example below is
> still an **ILLUSTRATED** pre-format sketch (it predates the `.ct.json`
> format and is not runnable); it is marked as such and must not be cited
> as EXECUTED.

## The one mental model

You write a **model document** (`.ct.json`: a schema — objects, arrows, path
equations — plus an instance: rows). catherine **executes** the declared
constraints against the instance and returns **finding-typed diagnostics,
each carrying the row or path that proves it** — or an honest
`UNKNOWN {scope}` where bounded search could not decide. Then you refine.
The loop is seconds, local, offline, deterministic: same input, same
findings, always (E5).

```
propose (.ct.json) → catherine check → findings with witnesses → refine → …
```

LLMs propose; this tool evaluates (C5). It never guesses, and every answer
carries its justification status: PROVED / EXECUTED / ILLUSTRATED /
HYPOTHESIZED / UNKNOWN.

## For agents

### Recipe A — model a requirement before building it

You are about to implement a feature described in prose. Before writing code:

1. Restate the domain as objects and arrows ("Assignment has `staff` and
   `equipment`; each belongs to a `Unit`"). Record ambiguities as UNKNOWN —
   do not resolve them silently.
   **Arrow granularity (PROPOSED, structural claims only):** declare one
   arrow per **stored, independently writable** reference. Before you model,
   name the storing column for each arrow; if more than a third have none,
   you are modeling the plan, not the system — stop. This governs
   *structural* claims: a storage-only model is blind to derivation and
   selection defects (executed negative test, 2026-11-13), so declare those
   relations explicitly instead of dropping them. Full rule, limits and the
   measurement: `ref/ct_methodology_playbook.md` §2.
2. Write the smallest schema that captures the *claims* the requirements
   make, plus a tiny instance of what they *imply* (3–10 rows, including
   one deliberately awkward row).
3. `catherine check model.ct.json`
4. For each finding, act on its type — see the finding table below.
5. Iterate until the findings are quiet or the remaining UNKNOWNs are real
   domain questions. **Those questions are the deliverable** — modeling
   forces spec questions nothing else asks (C6; kata-8's holdout failure
   exposed a genuine spec ambiguity).

### Recipe B — verify code claims against a model

You generated code that "implements the rules." Encode the rules as path
equations, extract a small instance of what the code actually produces (rows
in a fixture, a table dump), and check. A `violation` finding is a bug with
a witness row — take the row to the code, not a vibe.

### Recipe C — check a refactor before trusting it (P7 + P10)

Refactoring schemas? Run the Δ migration check: it emits **loud lost-data
counterexamples** and unsupported-inverse proofs — the one capability axis
conventional tooling has no counterpart for (E4, kata-10). Schema diffs
classify the edit: `catherine diff old.ct.json new.ct.json --rows Object=4`
reports *accreted* (constraints only added), *weakened* (with a witness
instance the new schema accepts and the old rejects), *equivalent-in-scope*
(no separating instance within the bounds), or *cosmetic* (presentation
only, zero findings). A weakened edit is a finding-bearing run (exit 3).

### Reading the findings

| Finding | Meaning | What to do |
|---|---|---|
| `violation` | a declared equation is false at a concrete row, or a declared key (`schema.keys`, 1.3) repeats a tuple | fix the data or the declaration; the witness names the row/equation or the two colliding rows |
| `dangling` | an arrow points at a row that does not exist | the claim references something absent — missing FK, dead reference |
| `contradiction` | no instance can satisfy the constraints (bounded search) | the *spec* is impossible as declared; negotiate which constraint yields |
| `underconstraint` | parallel paths exist with no declared equation *(hypothesis)* | a domain question: must they agree, might they, or should they differ? Declare an equation only if the domain says so |
| `redundancy` | a declared path is derivable from others | simplification candidate — remove only if observable meaning is preserved |
| `UNKNOWN {scope}` | bounded search hit its cap | the honest answer; widen scope explicitly or record it as unknown. Never reported as "clean" |

**Agent etiquette.** Do not add an equation just to silence an
`underconstraint` — a false equation rules out legitimate states. Do not
report a model as "verified" unless you ran the check and can quote the
output. Do not rewrite the requirements to make the model pass (kata
protocol rule 2: the baseline is frozen before formalisms enter).

### When the tool tells you to go away

On rule-based or trivial problems (parsing, formatting, scoring rules) the
correct use of catherine is to hear "model this conventionally" and comply —
the FizzBuzz negative control is a release gate (E4: kata-6 killed at a
7-line conditional; the checks worked, the cost didn't). If your problem is
a rule table *above* the triviality floor, the rule lens is the relevant
surface — but only for **extracted** tables (prose/PDF sources no DMN
analyzer ingests). Authored decision tables are commodity (Calvanese et
al., arXiv:1603.07466; Signavio, OpenRules). The extracted-table evidence,
including what the lens adds over a sampled conventional review, is the
probe ledger (`../fixtures/probes/rule_lens/README.md`, entries under
`ledger/probes/rule-lens-extracted/`).

### Recipe D — the seam agreement dbt cannot express

dbt's `relationships` test checks that one column's values exist in
another table (a single-path existence check). It cannot state that two
*composed* paths must agree. catherine can: declare both paths as an
equation and get the offending row.

`crates/catherine-cli/tests/fixtures/seam_multi_path_agreement.ct.json`:
a roll's frame must equal the frame of that frame's first roll
(`roll_frame ≡ roll_frame∘first_roll∘roll_frame`). Every single-path
reference resolves — a per-column FK check passes clean — but the
composed paths disagree at one row:

```text
$ catherine check crates/catherine-cli/tests/fixtures/seam_multi_path_agreement.ct.json --json
{"ok":true,...,"data":{"findings":[{"label":"executed","kind":"violation",
 "witness":{"equation":0,"row":2,"lhs":1,"rhs":0}}]},"warnings":[]}}
```

Honest boundary: parity with FK/relationships testing on single-path
existence (both detect the missing reference); unique on composed-path
agreement, where dbt has no operator. This is a modelling check on a
declarative document, not a drop-in warehouse test — extraction from the
warehouse is your adapter's job (`catherine` does not connect to SQL).

## For humans

- **Review models like code.** `.ct.json` files live in the repo; they diff
  in PRs; the schema graph and findings are the review surface. The web view
  (P5) is read-mostly: load a model, see the SVG schema graph, the findings
  panel, and the questions the model forces you to answer.
- **The value is thinking pressure.** The act of declaring paths exposes
  questions a back-of-the-napkin diagram hides (Johnson & Dampney's original
  empirical claim, from industrial elicitation; echoed — by a different
  instrument — in kata-8's sealed-holdout failure). The findings panel is a
  checklist of decisions you have not made yet.
- **The modeling procedure is documented separately**
  (`ref/category_theoretic_modeling_methodology.md`): freeze the baseline →
  semantic graph → inspect parallel paths → add only justified constraints →
  attack the model → evolution/views → refactor and stop. The tool executes
  steps 3–5; steps 0–1 and the stopping judgment remain yours.

## What it enables

Each line carries its license (see `PROPOSAL.md` §2):

| Capability | License |
|---|---|
| Invalid states rejected **at construction time**, for every future client of the model — not per-test at runtime | E4 |
| Spec-ambiguity discovery: modeling surfaces questions nobody asked | E4, C6 (kata-8) |
| Migration safety: loud lost-data counterexamples, unsupported-inverse proofs — unique axis, no conventional counterpart | E4, C3 (kata-10) |
| Equivalence-checked simplification: "collapsed 3 constraints into 2, no distinguishing witness" is compliance, proven | `distinguish`, C2 |
| Verified agent work: schema edits classified accreted / weakened / equivalent (P10, gated) | E4 + wild accretion laws (E10) |
| Honest uncertainty: `UNKNOWN {scope}` is first-class; budget exhaustion is never reported as complete not-found | C5, E5 |

## What it does NOT do

Equally load-bearing — each line with its source:

- **No theorem proving, rewriting engines, wiring diagrams, multi-logic
  translation, persistence, or servers.** Release criteria, not aspirations
  (PROPOSAL §3).
- **Nothing for rule-based logic at or below the triviality floor.** It says
  "model this conventionally" — and that verdict is a feature with a
  release gate (E4, kata-6).
- **Never a detection-rate win over a competent conventional test suite.**
  catherine complements tests (construction-time rejection, discovery
  pressure, migration); it does not out-detect them (E4). Keep your tests.
- **Formal consistency cannot detect missing requirements.** A model can be
  satisfiable and still omit a real-world rule (methodology findings
  taxonomy). Missing requirements need adversarial examples, domain
  questions, and independent requirements — use Recipe A's step 5 and the
  methodology's attack step.
- **History-dependent / temporal behavior is a validated expressibility
  limit**, not a target (kata-1/5: partial, recorded as gap — Bowling's
  mid-game scoring needs history the schema cannot see).
- **No Σ/Π migrations yet.** Δ first (P7); Σ/Π only after Δ is validated —
  "Π does not yet" (DESIGN §4.3). Kan extensions are named, gated, not shipped.
- **No sketch cones yet.** The vendored methodology's step-3 constructions —
  pullbacks for compatibility, monomorphisms for uniqueness, coproducts for
  exhaustive alternatives (Johnson & Rosebrugh sketch data models) — are
  advisory only in v1: the kernel executes **path equations**, not limit/colimit
  cones (THEORY §9: ACsets implement Spivak's model, not the full Johnson
  sketch methodology — enforcement is the contribution). Declaring a cone in
  this version checks nothing; treat it as prose until cones ship.
- **No view-update checking.** Whether an update against a model has a
  determinate meaning (the Johnson–Rosebrugh `unupdatable` finding) is
  post-v1; do not promise it.
- **Not a runtime monitor.** Offline CLI, seconds per check, no daemon, no
  coupling to a live filesystem or database (grounded instances are extracted
  *outside* the kernel — bogart's job, C4/E2).
- **No silent anything.** No silent lost data, no silent scope collapse, no
  silently re-run invalid experiments (ledger anti-goal), no silent
  constraint-weakening once P10 lands.
- **No knowledge-store ambitions.** Claims, lineage, provenance ledgers live
  in bajan/wild, which consume catherine verdicts — catherine emits verdicts
  only (program boundary, `bd memory: program-boundary`).

## Captured runs (EXECUTED, 2026-11-13)

Three real runs, captured from the shipped CLI. They are the evidence the
claim-scoping in `README.md` cites; the ledger entries are
`ledger/probes/migration/0002-06f4dd84.json` and
`ledger/probes/rule-lens-extracted/0002-c3316a38.json`.

Migration lost data — the dropped `RawCapture` table (abridged to the
first three lost rows; the run emits six, one per row, exit 8):

```text
$ catherine migrate fixtures/probes/migration/floor-visible-loss.ct.json --json
{"ok":true,...,"data":{"rows":{"ServingDocument":8,"ServingEdition":4,
 "ServingPage":16},"lost":[{"label":"executed","kind":"lost_row",
 "witness":{"object":"RawCapture","row":0}},{"label":"executed",
 "kind":"lost_row","witness":{"object":"RawCapture","row":1}},
 {"label":"executed","kind":"lost_row","witness":{"object":"RawCapture",
 "row":2}},...]}}
```

Executed check — a committed kata fault fixture, the equation violation
with its witness row (exit 5):

```text
$ catherine check fixtures/katas/bowling-fault-miswired-slot.ct.json --json
{"ok":true,...,"data":{"findings":[{"label":"executed","kind":"violation",
 "witness":{"equation":0,"row":1,"lhs":1,"rhs":0}}]},"warnings":[]}}
```

Rule lens on an extracted table — a widened ZD cap rule shadows the two
later ZD rows (exit 7):

```text
$ catherine rules fixtures/probes/rule_lens/extracted/faults/05-shadow-widen-zona-2022-10.ct.json \
    --values alcance=general,zona_desfavorable \
    --values fecha=2022-10-01,2022-12-01,2023-01-01 --max-candidates 64 --json
{"ok":true,...,"data":{"outcome":"analyzed","findings":[
 {"label":"executed","kind":"redundant","witness":{
  "derivable":["zona_desfavorable_2022_12_01"],
  "reducing":["zona_desfavorable_2022_10_01"]}},
 {"label":"executed","kind":"redundant","witness":{
  "derivable":["zona_desfavorable_2023_01_01"],
  "reducing":["zona_desfavorable_2022_10_01"]}}]}}
```

## Worked example (ILLUSTRATED)

Equipment assignments — the methodology doc's micro-example:

```json
{"objects": ["Assignment", "Staff", "Equipment", "Unit"],
 "arrows": {"staff": ["Assignment", "Staff"],
            "equipment": ["Assignment", "Equipment"],
            "staffUnit": ["Staff", "Unit"],
            "equipmentUnit": ["Equipment", "Unit"]},
 "equations": [["staffUnit", "staff"], ["equipmentUnit", "equipment"]],
 "instance": {"Assignment": [{"id": 1, "staff": "s1", "equipment": "e1"},
                             {"id": 2, "staff": "s2", "equipment": "e2"},
                             {"id": 3, "staff": "s1", "equipment": "e3"}]}}
```

```text
$ catherine check assignments.ct.json
violation: eq-0 staffUnit∘staff ≠ equipmentUnit∘equipment
  at row Assignment#3: lhs=Unit#1 rhs=Unit#2
underconstraint: parallel paths Assignment→Unit via staff/equipment
  have no declared equation? (hypothesis, domain question)
```

Wait — both findings at once? Yes, and that is the point: the violation
proves the declared rule bites (row 3 cross-units), and the underconstraint
asks the question the requirements never answered: *must* staff and
equipment units always agree, or is cross-unit assignment sometimes legal
(temporary delegation? shared equipment?). If the domain says "sometimes
legal," the equation is wrong — remove it and add the `Authorization`
object the requirements were missing. The model did its job: it found the
unasked question before any code existed.

And the anti-example, per the release gate: hand catherine FizzBuzz and the
expected outcome is the verdict "categorical modeling adds nothing here."
If it ever says otherwise, that is a bug in the tool, not a triumph.
