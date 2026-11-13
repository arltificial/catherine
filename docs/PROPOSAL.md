# Catherine proposal — what we are building and why

Status: proposal v2 (Rule-of-5 converged) · 2026-11-10 · audience: anyone
joining the project — human or agent — who must be able to state what this
tool is, who it is for, and why it deserves to exist, without reading code.
Companions: `DESIGN.md` (the how), `docs/ref/category_theoretic_modeling_
methodology.md` (the method the tool serves).

## 1. The problem

Modeling a problem before building it is still an artisanal act. When an
agent or a human tries to understand a domain — a workflow, a set of rules,
a bureaucracy, a data flow — the available supports are: prose documents
(easy to produce, silent about contradictions), diagrams (reviewed by
eyeball, enforced by no one), and tests (which encode assumptions rather
than surface them). The failure mode is always the same: the seams between
what was *assumed* and what was *declared* go unexamined until code meets
reality.

What's missing is a **thinking prosthetic at the moment of modeling**: an
artifact you can hand your claims to, that executes them, tells you which
ones contradict, which are missing, which are redundant — and answers with
witnesses instead of vibes.

The ct-catlab kata series (10 katas, validated 2026-11-02) pinned down
where this pays off in software terms:

- **Discovery pressure is real.** In kata-8, the categorical arm failed one
  of its own sealed holdout requirements — and that failure exposed a
  genuine ambiguity in the spec itself. The act of modeling forced a domain
  question nobody had asked. That is the thinking value, measured.
- **Invalid states are rejected too late.** Conventional suites catch
  invalid states *behaviorally* (runtime, per test). Nothing rejects them
  *at construction time* for every future client of the model.
- **Schema migration loses data quietly.** Kata-10: conventional tooling
  has no counterpart to "this refactor loses data" or "this mapping cannot
  be inverted" — you find out in production.

The twin gap is **enforcement-as-default + packaging**. The math is settled
(Johnson's sketches, Spivak's functorial migrations, AlgebraicJulia's
ACsets) — but Catlab's own docs state ACSet path equations are
documentation, not enforcement; CatColab is an application, not a kernel;
CQL is a standalone database DSL. Nobody ships the small, executable,
JSON-speaking artifact an agent can call mid-conversation or a human can
open in a browser tab.

## 2. Conclusions we are building on

From the cotorreo corpus and the ct-catlab series; full citations in
`DESIGN.md` §0.

| # | Conclusion | What it means for the tool |
|---|-----------|---------------------------|
| C1 | Declarations don't enforce; checks do (Catlab equations are docs-only) | Executed checks ARE the product. A schema without execution is a bug. |
| C2 | CT's validated value is bounded: state-shape invariants, discovery pressure, migration axis; nothing for rule-based logic *at the triviality floor* (kata-6 killed the schema-lens rule table on a 7-line conditional — the checks worked, the cost didn't); never a detection-rate win | The tool must know its limits and say "model this conventionally" (the negative control is a feature). The rule lens for systems *above* the floor is a pre-registered hypothesis (P8), not a claim. |
| C3 | The migration axis is a capability gap conventional tooling cannot match (loud lost-data counterexamples, unsupported-inverse proofs) | Migration (Δ) is the flagship scenario, not a bolt-on. |
| C4 | A fixed-schema kernel is small; specodelic's 3.3k-line acset is big because of generics + lossless writer + parity harness | The kernel stays fixed-scope; smallness is a feature and a falsifiable claim. |
| C5 | "LLM proposes, deterministic tools evaluate" + epistemic labels (PROVED/EXECUTED/ILLUSTRATED/HYPOTHESIZED/UNKNOWN) | The tool is the evaluator in an agent loop; it never guesses; every answer carries its justification status. |
| C6 | Discovery pressure is the recurring benefit: modeling forces questions conventional schemas never ask | Output is *questions with witnesses*, not just pass/fail. |
| C7 | Parity-gated extraction works — specodelic already runs a validated Rust acset core | Not starting from zero: the instance-table core is proven Rust. |
| C8 | CatColab proves Rust→WASM CT modeling in the browser — as an app | We occupy the layer below: the kernel an app like CatColab could embed. |
| C9 | Agent-contributed code punishes low-resource languages and irregular APIs | Rust (well-resourced), small regular JSON-first API. |

**Why now:** the validated boundary is two weeks old (E4/REPORT 2026-11-02);
a proven donor kernel exists (specodelic acset, C7); and the
Rust-in-browser path is de-risked by CatColab (C8). All three converged
only recently — a year ago this proposal would have been a research bet
instead of an engineering plan.

## 3. What we are designing

One sentence: **a small deterministic kernel that takes a model document
(schema + instance as JSON), executes its declared constraints, and returns
finding-typed diagnostics with witnesses — callable from a CLI by agents
and openable in a browser by humans.**

("Finding-typed": diagnostics classified by kind, each carrying the concrete
row/path that proves it.)

In scope for v1: finitely-presented schema (objects, arrows, path
equations); typed instance with dangling-as-value; executed path-equation
checks; **five executed/deterministic finding types** — violation,
dangling, contradiction (bounded search), underconstraint (hypothesis on
parallel paths), redundancy — plus bounded counterexample search and Δ
migration. Post-v1, named now so scope is honest: *missing-concept*
scaffolds, *view-update* analysis (unupdatable finding, Johnson–Rosebrugh
fibrations), and the **rule lens** (P8: rule systems as subobjects of the
input domain with a resolution policy — see `DESIGN.md` §4.4).

Out of scope (release criteria, not aspirations): theorem proving, rewriting
engines, wiring diagrams, multi-logic translation, persistence/servers.

## 4. Value proposition

### For an agent (the primary user)

Modeling becomes a **propose → execute → refine** loop. The agent writes a
20-line JSON schema of what the requirement *claims* and a tiny instance of
what it *implies*, then runs `catherine check`. Back come specific,
witness-backed findings: this constraint is violated by row #3; these two
parallel paths were never declared equal; this arrow dangles; this
declaration is derivable (redundant); here is the smallest instance that
breaks the assumption — or `UNKNOWN {scope}` where bounded search couldn't
decide. Each loop is seconds, local, offline. Agents generate plausible
models fast and verify poorly; catherine is the deterministic evaluator
that never hallucinates a verdict, and on rule-based problems (parsing,
formatting, scoring) it says so: model it conventionally.

### For a human

The same artifact, openable in the browser: schema graph, findings panel,
and the questions the model forces you to answer. The value is **thinking
pressure** — the act of declaring paths exposes the questions a
back-of-the-napkin diagram hides (Johnson & Dampney's original empirical
claim; re-measured by kata-8's spec-ambiguity discovery). Files stay
`.ct.json` in your repo; they review like code; the web view is
read-mostly (JSON editing).

### How it differs from the alternatives

| | Catherine | Catlab/ACSets | CatColab | Plain types+tests | Alloy/Quint |
|---|---|---|---|---|---|
| Equations | **executed** with witness rows | declared, docs-only by default | model-level analysis | implicit in assertions | temporal/relational constraints |
| Output | finding-typed, epistemic labels | library values | interactive analyses | booleans/panics | counterexamples (behavioral) |
| Agent ergonomics | CLI + JSON, seconds, offline | Julia + macros | web app | excellent, no structural lens | external toolchain |
| Migration | Δ with lost-data counterexamples | implemented, but locked in the Julia research stack | partial | none — silent data loss | n/a |
| Size/deploy | kernel-scoped, auditable, WASM | research-scale | application | minimal | bounded-check toolchain |
| Knows its limits | yes — negative-control verdict | no | no | n/a | no |

Against Alloy/Quint specifically: they check *behavior within bounds*;
catherine checks *structural and semantic declarations* (path agreement,
construction-time shape, migration safety). Orthogonal lenses; a bounded
checker is a plausible analysis backend later, not a competitor now.

### The honest boundary (part of the proposition)

The strongest claim we can make is the one ct-catlab validated: on
**state-shape domains**, construction-time rejection and schema-level
invariants are worth ≈1–3× LOC; on **schema evolution**, there is no
conventional counterpart; on **rule-based logic**, there is no validated
advantage *at the triviality floor* (kata-6: a 7-line conditional is
correctly served by trivial tests; the rule-lens checks fired but earned
nothing) — and the tool says so. Real domains mix rules and state: the
rule parts stay conventional until the rule lens (P8) proves otherwise,
the state-shape parts go through the kernel. We claim a
*discovery and enforcement* advantage with witnesses — we do **not** claim
to find more defects than a competent conventional test suite.

### One-sentence version

**Catherine turns "do these declarations actually hold?" from a
human-reviewed diagram into an executed, witness-backed check that any
agent or browser can run — and tells you when modeling is a waste of
time.**

### Thirty-second version

Agents and humans make implicit claims about a domain — "assignments stay
within a unit", "this refactor preserves data". Today those claims are
checked late (tests), weakly (review), or not at all. Catherine is a small
local kernel that takes those claims as a JSON schema, executes them
against a concrete instance, and returns finding-typed diagnostics with
witnesses: violations, dangling references, redundancy, underconstrained
paths, and — uniquely — migration counterexamples that prove data loss
before it happens. Every answer is labeled with how it was established
(proved / executed-in-scope / hypothesized / unknown). Built on the
ct-catlab validated boundary: real for state-shape and migration problems,
honestly useless for rule-based ones — and the tool says which is which.

## 5. What success means (falsifiable)

1. P6 ledger entry exists with a negative-control verdict (FizzBuzz → "no
   categorical advantage" verdict produced *by the tool*). If P6 is
   **killed**, the project stops and records the negative result — that is
   the pre-committed stop condition, not a failure mode.
2. Three ported katas (Bowling, Gilded Rose, Vending) reproduce their
   validated findings as executed, witness-backed checks — without Julia.
3. A spec-ambiguity discovery is reproduced at least once (kata-8
   pattern): the tool's underconstraint/parallel-path findings surface a
   genuine domain question the input spec never answered.
4. The agent loop (propose schema → check → refine) completes in seconds,
   fully offline, with no model-training or API dependency.
5. We do **not** claim: higher detection rates than conventional suites,
   general CT coverage, or value on rule-based domains.

## 6. Position in the ecosystem

- **specodelic** — donor of the proven Rust acset core (parity-gated
  extraction); catherine's kernel may flow back as its modeling substrate.
- **ct-catlab** — supplier of validated expectations: the kata fixtures are
  the acceptance tests, the ledger discipline is the process.
- **CatColab / Catlab / CQL** — prior art and references; not dependencies,
  not competitors at this layer. Catherine is the embeddable kernel layer
  below CatColab's application layer.
- **genesis-vibes** — the suite's shared CLI foundation (envelope, verbosity,
  doctor); consumed by `catherine-cli` only, so agents get the exact JSON
  envelope semantics every charly-vibes tool already speaks. Kernel stays
  suite-dep-free.
- **wild** (`charly/wild`) — in-suite prior art discovered during the doc
  review: contracts modeled as a category with checkable accretion laws;
  canonical-JSON + sha256 identity is *implemented* and ported into this
  design (reuse map, DESIGN §3). Compose/registry tiers are normative-only —
  cited as design, never as validated evidence. Catherine remains the suite's
  first *general-purpose* CT modeling kernel; wild validates that
  category-with-checkable-laws works inside this org's engineering practice.
- **Methodology v1.0** (`docs/ref/`) — the human procedure the tool
  mechanically supports (freeze baseline → semantic graph → parallel paths
  → justified constraints → derive & attack → evolve → stop). Catherine is
  the "derive & attack" step made executable.

## 7. CQL / functorial data model: head-to-head and patent exposure

The formalism catherine implements **is** the functorial data model: schemas
as categories (multigraphs with path equations), instances as functors to
Set, migration as Δ. CQL (Conexus AI) is the mature implementation of that
formalism; it is prior art, not a dependency.

| Capability | CQL / functorial data model | catherine |
|---|---|---|
| Schema + path equations | Yes (the schema is a category) | Yes — JSON documents, format 1.0–1.3 |
| Equation enforcement | Yes, inside its query engine/DSL | Yes, on a finite instance, per row |
| Witnesses | Query results | Named witness per finding (`equation`, `row`, `lhs`, `rhs`; lost row/cell; rule name) |
| Underconstraint / ambiguity | Not an output | Domain question with the parallel path and the missing equation |
| Bounded search | N/A | Smallest counterexample, or `UNKNOWN {scope}` — never a naked "no violation found" |
| Epistemic labels | N/A | PROVED / EXECUTED / HYPOTHESIZED / UNKNOWN on every answer |
| Migration | Δ, Σ, Π | Δ only, with loud lost-data enumeration and non-invertibility proofs |
| Packaging | Standalone database/DSL (JVM toolchain) | Fixed-scope Rust kernel + CLI + wasm, JSON envelopes, no DB/server |
| Agent use | Via its own tooling | One command mid-conversation; the same path the browser view runs |

Parity is narrow and stated: both enforce declared equations and both can
migrate along a Δ. The differences are witness typing, ambiguity-as-output,
bounded honesty, and packaging — not a claim of greater modeling power.

**Patent exposure (assessment, not legal advice).** Conexus AI holds granted
US patents on a "data migration and integration system" — US 11,256,672 B1
(granted 2022-02-22) and US 11,720,535 B2 (granted 2023-08-08) — with a
further application US 2024/0119034 A1 published, all naming Daimler and
Wisnesky among the inventors. Catherine contains no CQL code, API, or
configuration; it is an independent Apache-2.0 implementation of published
mathematics (Spivak's functorial data migration). That does **not** settle
patent risk: method claims can read on independent implementations, and the
repo carries no patent grant from Conexus. Required posture: (1) no public
novelty claim about functorial migration without citing this section; (2)
any distribution beyond internal/engineering use — commercial, hosted, or
binary redistribution — must be cleared by counsel against the claims above
before it happens; (3) if counsel's answer is non-obvious or negative, the
feature ships as documentation/recipe and the claim is withdrawn. Escalate
to the human owner; do not self-clear.
