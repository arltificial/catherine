# Catherine theory — the mathematics behind the tool, and why it helps

Audience: an engineer or agent who wants to understand *what* categorical
machinery the tool uses and *why each piece earns its place* — grounded in
the source papers and in the ct-catlab empirical record. No prior category
theory assumed; every construction is introduced with a running example.

Companions: `PROPOSAL.md` (why the tool exists), `DESIGN.md` (how it is
built), `ref/category_theoretic_modeling_methodology.md` (the modeling
procedure the tool serves).

## 1. Why category theory at all

The founding observation (Johnson & Dampney, from industrial consulting at
Telecom Australia and Caltex): **the act of drawing relationships precisely
— before writing software — exposes inconsistencies, gaps, and
over-complications that prose and box-and-arrow sketches hide.**

Category theory contributes one disciplined idea: model what the system
*knows* as objects, model relationships as arrows, and then **investigate
the consequences of the relationships** — which paths must agree, which
structures must exist, which combinations are permitted. A diagram stops
being a picture and becomes a set of obligations that can be *executed*.

That yields five distinguishable questions the tool is built around:

1. **Meaning** — what does the model actually represent?
2. **Consistency** — which different paths must produce the same result?
3. **Completeness** — what relationships, entities, or cases are forgotten?
4. **Behavior** — which changes preserve meaning and constraints?
5. **Evolution** — how do we modify or connect systems without breaking
   guarantees?

These are *not* the same problem, and the theory supplies a different
construction for each. The rest of this document walks through each
construction: what it is, what it buys, and where it has been validated.

## 2. Schemas: finitely-presented categories

**What it is.** A schema is three finite lists: objects (sorts of things:
`Staff`, `Equipment`, `Unit`, `Assignment`), generating arrows (relationship
functions: `assignment.staff ↦ Staff`, `equipment.unit ↦ Unit`), and path
equations (declarations that two arrow sequences compose to the same
function). A *path* is a composable sequence of arrows
(`Assignment → Staff → Unit`).

**What it buys.** Relationships become first-class named data — not match
arms in code, not foreign keys buried in SQL DDL. The schema *is* the
vocabulary of the domain, and everything downstream (checks, findings,
migrations) is computed from it. Johnson & Rosebrugh's sketch data models
established this as a data-specification formalism equivalent in power to
relational schemas but with functions (not just tables) as primitive.

**Lineage:** Johnson & Rosebrugh, *Sketch Data Models, Relational Schema and
Data Specifications*; Johnson, *A Categorical Logic for Information Systems*
(the logic of an information system should follow from the categorical
operations actually needed — the anti-overengineering argument our non-goals
list inherits).

## 3. Instances: functors into Set

**What it is.** A concrete state of the system is an *instance*: each object
interpreted as a set of rows, each arrow as an actual function between those
rows. Formally, an instance is a functor `I : 𝒞 → Set` from the schema
category to sets and functions (Spivak's C-sets; AlgebraicJulia's ACsets are
the efficient in-memory implementation — typed tables of `Vec<Option<usize>>`
with `None` = dangling).

**What it buys.** Construction-time shape: an instance either *is* a valid
interpretation of the schema or it isn't, and the invalid cases become
findings at construction — not test failures later. In the validated katas,
state-shape invariants (frozen quality, ledger conservation, command
vocabularies) held as *static* invariants that no per-test assertion
provides: they guard **every future client** of the model, not one test's
inputs.

**Lineage:** Patterson, Lynch & Fairbanks, *Categorical Data Structures for
Technical Computing* (ACsets); Spivak's C-sets. Specodelic's Rust acset
module (schema 946 + instance 925 LOC) is our donor implementation.

## 4. Path equations: commutative diagrams as obligations

**What it is.** A path equation declares that two routes through the schema
compute the same thing:

```
unitOfStaff ∘ staff = unitOfEquipment ∘ equipment
```

(the Johnson & Dampney running example: an assignment's staff unit must
equal its equipment's unit). This single equation constrains *every valid
assignment forever*.

**What it buys.** Three distinct outcomes from one declaration:

- **Consistent** — both paths agree on the current instance: obligation met.
- **Violation** — they disagree: finding with the offending row attached
  (`lhs=Unit#1, rhs=Unit#2`), i.e. a *witness*.
- **Underconstraint** — the two paths exist but no equation was declared:
  the tool raises the domain question *"should these agree?"* This is the
  discovery mechanism: in kata-8 this pattern exposed a genuine ambiguity
  the spec never answered. The question is raised *by the model*, not by a
  test author.

**Critical empirical caveat** (declarations don't enforce — ct-catlab smoke
gate S3; Catlab's own docs): Catlab's ACSet path equations are
documentation, not enforcement — the library does not evaluate them on
instances. Executing the equations *is* catherine's core deliverable (P1–P2):
declarations don't enforce; checks do.

## 5. The findings taxonomy: inconsistency ≠ incompleteness

Category theory's deeper gift to design is separating failure modes that
prose collapses into "something's wrong" (ChatGPT conv `6adf915c` §8;
methodology v1.0 "distinguish the findings"):

| Finding | Meaning | What mathematics can establish |
|---|---|---|
| violation | a declared equation fails on a row | witness row + both path values |
| dangling | an arrow points to nothing | reference integrity as a *value*, not an exception |
| contradiction | constraints admit no instance | bounded search: no model satisfies the declarations |
| underconstraint | parallel paths, no declared equation | multiple interpretations remain possible — needs a domain decision |
| missing | no representation of a needed concept | *requires domain knowledge* — only hypothesized, never deduced |
| redundant | a stored path is derivable by composition | provable eliminability |
| unupdatable (post-v1) | a view update has no valid propagation | Johnson–Rosebrugh view-update theory |

The discipline: the tool may *prove* violations, contradictions, and
redundancy; it may *hypothesize* underconstraint and missing (they are
domain questions); it must never blur the two. Hence the epistemic labels
(PROVED / EXECUTED / ILLUSTRATED / HYPOTHESIZED / UNKNOWN).

**Lineage:** Johnson & Rosebrugh, *Fibrations and Universal View
Updatability* — the view-update question shifts from "how do we implement
this update?" to "**does the model contain enough information to determine
what this update means?**" Their machinery (fibrations/opfibrations,
universal properties for preferred updates) is post-v1 precisely because it
is the heaviest construction for the narrowest finding.

## 6. Migration: the one capability with no conventional counterpart

**What it is.** A map between schemas induces three data migrations
(Spivak's functorial data migration): Δ (reindex — reshape data, invent
nothing), Σ (left Kan extension — aggregate/collapse), Π (right Kan
extension — pair/filter by universal constraints). catherine ships Δ first.

**What it buys.** Schema evolution with proofs instead of hope:

- **Loud lost-data counterexamples**: the migration provably discards rows
  or fields — exhibited concretely, not discovered in production.
- **Unsupported-inverse proofs**: "you cannot round-trip this mapping" is
  *proven*, not suspected.

kata-10 validated this axis as a genuine *capability gap*: conventional
refactoring has no behavior to compare — it is not a speed contest.
Practical target already identified: bogart's raw working corpus vs its
derived serving SQLite projection.

**Lineage:** Spivak, *Functorial Data Migration*; the Σ ⊣ Δ ⊣ Π adjoint
triple (Fong & Spivak, *Seven Sketches*).

## 7. The rule lens: subobjects and resolution policy (P8)

**What it is.** A rule system is: an input domain `X`; rules as **subobjects**
`R_i ↪ X` (decidable predicates); an action assignment that must become a
total function on `X`; and a **resolution policy** — priority order,
first-match, or declared disjointness (the same three DMN hit policies).
The *effective region* of rule i is `E_i = R_i \ ⋃_{j<i} R_j`.

**What it buys.** The classic rule-table bug classes become exact findings
on enumerable domains:

- **gap** — `⋃R_i ⊊ X`: an input class with no rule (partiality explicit)
- **overlap** — `R_i ∩ R_j ≠ ∅` with no policy: nondeterminism ("who wins?")
- **shadow** — `E_i = ∅`: a dead rule, derivable from earlier rules
- **derivable** — `R_k = R_i ∩ R_j`: a *simplification candidate*

Two properties lift this above linting:

1. **Equivalence-provable simplification.** Two rule tables are equivalent
   iff they induce the same total action on `X`; `distinguish` exhibits a
   witness input where they differ. "Collapsed 3 rules into 2 + precedence,
   no distinguishing witness" is compliance, *proven*. FizzBuzz's
   `15 → "FizzBuzz"` rule is derivable from `3 → "Fizz"` ∧ `5 → "Buzz"`
   under a precedence policy.
2. **Rule discovery.** Given examples and a declared predicate vocabulary,
   candidates come from the agent/human; the tool checks partition
   equivalence against the examples; uncovered examples fall out as gap
   findings.

**Honest status: HYPOTHESIZED.** kata-6 killed the *schema-lens* rule table
at the triviality floor: its checks (overlap/coverage/domain) fired 3/3 on
FizzBuzz, but trivial tests catch the same faults on a 7-line conditional,
so the machinery earned nothing *there*. The claim above the floor (10–20
compound rules + policies, where humans lose track of effective regions) is
pre-registered, not validated — precedent exists (Rust's match
exhaustiveness/reachability warnings; DMN hit-policy conflict detection)
but no ct-catlab-class evidence. P8 validates or kills it honestly, with
its own near-floor negative control.

## 8. What category theory does NOT buy (validated negatives)

These are load-bearing conclusions, not humility theater — each is
executable evidence from the ct-catlab series:

- **No detection-rate win.** In every validated run the categorical arm
  never caught *more* faults than a competent conventional suite; its edge
  was *how* (schema-level, reusable across clients), not *how many*.
- **Nothing at the rule-logic floor.** kata-6 (FizzBuzz): zero unique
  detections at several× overhead.
- **History resists shape.** Transition-*history* properties (kata-5's
  win-legality, kata-1's temporal semantics) resist state-shape checks even
  where state shapes don't — a validated boundary of the kernel's lens.
- **Cost is real.** ≈1–3× LOC for the categorical arm including checks.
- **Declarations ≠ enforcement.** The smoke gate proved Catlab equations are
  unevaluated; every enforcement claim that validated required explicit
  executable checks.

This is why the FizzBuzz negative control is a *release gate*: a tool that
cannot say "modeling is a waste of time here" is a ceremonial math
generator.

## 9. Reading order (from the corpus synthesis, conv `6adf915c` §11)

1. **Johnson & Dampney — On the Value of Commutative Diagrams in Information
   Modelling.** The empirical seed: commutative diagrams improved
   entity-relationship models in industrial practice; static/dynamic
   constraints; the timetable example. (§3.1, §4, §5)
2. **Johnson & Rosebrugh — Universal View Updatability.** EA sketches,
   queries, updates that are impossible or ambiguous; fibrations.
3. **Johnson & Rosebrugh — Sketch Data Models, Relational Schema and Data
   Specifications.** Translation and comparison of models; non-equivalent
   schemas.
4. **Johnson — A Categorical Logic for Information Systems.** The logic
   should follow from the operations actually needed — the
   anti-overengineering principle.
5. **Spivak — Functorial Data Migration.** Schemas as categories, instances
   as functors, Δ/Σ/Π as adjoints.
6. **Patterson, Lynch & Fairbanks — Categorical Data Structures for
   Technical Computing.** ACsets: the efficient in-memory implementation
   catherine's instance tables descend from.
7. **Seemann — *From design patterns to category theory* series**
   (blog.ploeh.dk, 2017–2025): the *design-level* mirror of this thesis
   — law-governed universal abstractions replacing ambiguous design
   patterns at the API level; mapped to this codebase in
   `docs/ref/ploeh-grounding.md`.

How the threads relate: **Johnson** supplies the design methodology
(sketches + universal properties as thinking tools); **Spivak** supplies the
algebraic simplification (schemas as functors, migrations as Kan
extensions); **AlgebraicJulia** supplies the implementation substrate; and
**wild** (same org) supplies in-suite proof that a category with checkable
laws is sound engineering practice — accretion of contract revisions is a
thin category (preorder), demand-restriction is a functor, identity is
canonical-JSON hashing (implemented, byte-for-byte normative). ACsets
implement Spivak's model — *not* the full Johnson sketch methodology — which
is why enforcement and findings are catherine's contribution rather than
redundant with any of them.

## 10. Glossary

| Term | One-liner |
|---|---|
| schema | finite presentation: objects + arrows + path equations (a finitely-presented category) |
| instance | concrete state: each object ↦ rows, each arrow ↦ functions (a functor `𝒞 → Set`) |
| path | composable sequence of arrows (`Assignment → Staff → Unit`) |
| path equation | declaration that two paths compose to the same function (a commutative diagram) |
| witness | the concrete row/instance that demonstrates a finding — findings always carry one |
| subobject | a predicate over a domain, viewed as an object inclusion (`R ↪ X`) |
| effective region | `E_i = R_i \ ⋃_{j<i} R_j` — where rule i actually fires |
| Δ / Σ / Π | the migration functors induced by a schema map: reindex / aggregate / pair-and-filter — with Σ ⊣ Δ ⊣ Π forming an adjoint triple (Σ and Π are left/right Kan extensions along Δ) |
| Kan extension | the universal "best approximation" of extending data along a map — Σ and Π are left/right Kan extensions along Δ |
| fibration | the machinery for view updates: which base changes lift to a view change, uniquely or at all (post-v1) |
| epistemic label | how an answer was established: PROVED / EXECUTED (in scope) / ILLUSTRATED / HYPOTHESIZED / UNKNOWN |
