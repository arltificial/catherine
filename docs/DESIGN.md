# Catherine — CT modeling workbench design

Status: draft v2 · 2026-11-10 · evidence-linked design for a browser-runnable,
agent-usable category-theory modeling tool, deliberately simpler than Catlab.

Naming: *catherine* — cat, from *categories*. The pun is the scope warning:
this is the small cat, not the whole zoo.

Vendored references: `docs/ref/category_theoretic_modeling_methodology.md`
(copy of the methodology v1.0 artifact; source path in the citation table).
All other sources are cited by conversation ID + provider; transcripts live in
the cotorreo corpus on pinky (`raw/_conversations/<provider>/<id>/transcript.md`).

## 0. Evidence base (from the pinky/cotorreo corpus)

Conclusions this design commits to, each with provenance:

| # | Conclusion | Source |
|---|-----------|--------|
| E1 | CatColab (Topos Institute) already does Rust-core + browser CT modeling on double-categorical theories; it is an application, not a reusable small crate; differentiation must be demonstrated | claude `e7bfb3b5` ("Category theory software primitives and Rust tools") |
| E2 | A *general* ACset library is overkill for a fixed modeling domain; a small typed-instance core suffices (schema as data, instances as `Vec<Option<usize>>` morphism tables, union-find pushouts). The 400–800-line estimate was for a **fixed-schema** kernel; specodelic's generic acset crate (~3.3k lines: schema 946, instance 925, writer 631, query 264) is bigger because it adds a lossless writer, generic schemas, and parity harness — the honest lesson is *kernel small, crate bigger because of writer + tests* | claude `e7bfb3b5` + `specodelic/src/acset/` |
| E3 | Declarations do not enforce; checks do. Catlab ACSet path equations are documentation, not enforced (smoke gate S3; Catlab docs). Any tool whose value proposition is consistency discovery must execute checks, not just represent equations | matadero `series/ct-catlab/SERIES.md`; chatgpt `6ae94740` ("Design Modeling Libraries") |
| E4 | Categorical modeling buys: state-shape invariants, construction-time rejection of invalid states, spec-ambiguity discovery, and a *unique* data-migration axis (loud lost-data counterexamples, unsupported-inverse proofs). It buys nothing for rule-based logic (kata-6 killed), never beat a competent conventional suite on detection rate, and costs ≈1–3× LOC | matadero `series/ct-catlab/REPORT.md` (validated 2026-11-02) |
| E5 | The workbench design: tiny semantic kernel (finitely-presented schema: objects, arrows, path equations; finite instances), separate syntax/semantics/verification/epistemic layers, deterministic inquiry operations, result labels PROVED/EXECUTED/ILLUSTRATED/HYPOTHESIZED/UNKNOWN, LLM proposes / deterministic tools evaluate, composition-as-pushout only when justified | chatgpt `6ae94740` |
| E6 | Methodology v1.0 defines the human/agent procedure the tool must support: freeze baseline → semantic graph → parallel paths → justified constraints → derive consequences & attack → evolution/views → refactor and stop; findings must be distinguished (violation / contradiction / underconstraint / missing / redundant / unupdatable) | `docs/ref/category_theoretic_modeling_methodology.md` (vendored); chatgpt `6adf915c` §8 |
| E7 | Agent-facing language choice is a training-data question: OCaml and other low-resource languages mean higher repair cost for agent contributors. Honest read: Rust is adequately resourced **but not free** — it produced the only 3 failures in the cited 600-run benchmark; semantic regularity and predictable evaluation matter more than type-system sophistication, so the kernel must stay small and regular | grok `0ad787db` ("Programming Languages AI Software") |
| E8 | Incremental extraction with parity gates works: specodelic acset was extracted behind a graph-parity gate + rename-oracle; Tambor analysis established "falsifiable migration experiment" as the pattern | claude `e7bfb3b5`; chatgpt `6adf3073` ("Assess Rust Migration") |
| E9 | An earlier recommendation for this exact project was TypeScript-first with the Rust/WASM core as a later optimization | chatgpt `6ae94740` §7 |

## 1. Problem statement and hypothesis

Agents and humans need to model a problem before building software. The
ct-catlab series validated that categorical modeling has *bounded, real*
value (E4) but the only usable tools are either heavyweight (Catlab: Julia,
macros, research-scale) or embedded in someone else's app (CatColab). There
is no small, dependency-light, browser-runnable kernel that:

1. takes a **model document** (`.ct.json`: schema + instance),
2. **executes** the declared constraints against the instance (E3),
3. surfaces **finding-typed** results instead of a boolean,
4. speaks JSON fluently so agents can drive it without a UI and humans can
   drive it without an agent.

The finding taxonomy (which layer computes what — see §4):

| Finding | Meaning | Computed in |
|---|---|---|
| violation | a declared path equation fails on a row | core (P1–P2) |
| dangling | an arrow value points to no existing row | core (P1) |
| contradiction | declared constraints admit no instance (bounded search) | analysis (P4) |
| underconstraint | parallel paths exist with no declared equation; domain question raised | inquiry (P4/P6) |
| missing | domain concept absent — heuristic scaffold, hypothesis-labeled | inquiry (post-v1, spec-only in v1) |
| redundant | a declared path is derivable from declared equations | analysis (P4) |
| unupdatable | a view update has no valid propagation (Johnson–Rosebrugh view-update theory) | **post-v1** — requires morphisms of instances / fibrations; explicitly out of scope |

**Hypothesis (falsifiable).** A browser-runnable kernel with enforced
path-equation checks and finding-typed output helps an agent or human find
more specification defects earlier than schema-less inspection, on
state-shape domains, at acceptable effort cost — and *says so* (recommends
conventional modeling) on rule-based domains per E4.

**Stop condition (project-level, per E6 "refactor and stop").** The project
is done when P6 produces its ledger entry — survived or killed. It is
*killed* if the ported katas show the tool finds no defect class that
schema-less conventional inspection misses, at acceptable effort; killed
means stop building, write the negative result, keep the ledger.

## 2. Non-goals (deliberately deferred, per E5 §9)

- No general theorem prover, SMT integration, or categorical rewriting engine.
- No new database, persistence layer, or server; files first.
- No universal ontology or multi-logic translation framework.
- No Julia/Catlab interop in v1 (JSON only; CQL/Catlab are references, not deps).
- No replacing CatColab; differentiating claim is *enforcement + findings +
  agent-first ergonomics*, not breadth of mathematics.
- No wiring diagrams / string diagram rendering in v1.
- No view-update analysis in v1 (see finding table: unupdatable is post-v1).

## 3. Technology decision: Rust core → WASM, thin web UI

The E9 recommendation was TypeScript-first. We depart from it, and the
departure must be justified, not assumed:

| Factor | TS-first | Rust→WASM (chosen) |
|---|---|---|
| Reuse | none available | specodelic acset core (E2) is Rust and already validated by parity gates |
| Browser | native | proven by CatColab (E1); `wasm-bindgen` + serializable JSON |
| Agent CLI | separate impl or Node runtime | one crate, same binary logic headless (E8: parity-gated core) |
| Types | gradual | algebraic data types make the six findings unrepresentable-as-invalid (mirrors E2's "invalid states unrepresentable at construction") |
| Ecosystem for UI | strongest | sufficient: plain TS/HTML over WASM exports; no Leptos/Yew needed in v1 |
| Agent writability | strong | adequate per E7, *with the caveat stated*: Rust produced the only benchmark failures (E7); mitigation is a small, regular, JSON-first API — regularity over type ceremony |

**Decision:** Rust workspace, `wasm-bindgen` boundary, UI as a thin static
site (TypeScript + SVG rendering of the schema graph), JSON as the only
model interchange format. **OCaml was considered and rejected on E7
grounds**: the corpus contains no OCaml CT implementation experience to
reuse (the only OCaml-adjacent finding is that low-resource languages cost
agents more repair cycles), and agent contributors are a first-class user.

Testing tooling: `proptest` for round-trip and parity properties;
golden-file fixtures for parser errors and findings; `insta`-style snapshots
are deliberately avoided — findings carry witnesses and must be asserted
structurally, not byte-compared.

## 4. Architecture

One workspace, five crates, three artifacts (lib, wasm, static site):

```
crates/
  catherine-core/      schema, instance, paths, equations, checks, findings
  catherine-analysis/  bounded search, counterexamples, redundancy
  catherine-inquiry/   deterministic question ops (E5 table), finding labels
  catherine-wasm/      wasm-bindgen bindings; JSON in/out; no logic of its own
  catherine-cli/       headless runner for agents/CI; same core, no wasm
web/                   static site: TS + SVG; loads .ct.json files; no server
```

Dependency direction: `cli`/`wasm` → `inquiry` → `analysis` → `core`.
The web UI depends only on the wasm exports and the model document format —
it is never the source of truth (E5).

### 4.1 Core kernel (what "simpler than Catlab" means concretely)

The entire kernel surface in v1, as Rust types — this is the size claim,
falsifiable at review:

```rust
pub struct Schema {
    pub objects: BTreeMap<Ident, ()>,          // sort names
    pub arrows: BTreeMap<Ident, Arrow>,        // generating morphisms
    pub equations: Vec<Equation>,              // path == path, with status
}
pub struct Arrow { pub src: Ident, pub tgt: Ident }
pub struct Equation {
    pub lhs: Path, pub rhs: Path,
    pub rationale: String,
    pub status: ConstraintStatus,              // Proposed | Accepted
}
pub struct Path(pub Vec<Ident>);               // composable arrow sequence

pub struct Instance {
    pub rows: BTreeMap<Ident, Vec<RowId>>,     // per-object row ids
    pub values: BTreeMap<Ident, Vec<Option<RowId>>>, // per-arrow table; None = dangling
}
pub enum Finding {
    Violation { equation: usize, row: RowId, lhs: String, rhs: String },
    Dangling  { arrow: Ident, src: RowId },
    Redundant { path: Path, by: Path },
    // contradiction / underconstraint arise at analysis/inquiry layers
}
```

Rules:

- No macros, no sketch generics, no double categories. Attribute columns are
  a small alphabet (string/int/bool) attached to objects, added only when a
  P6 fixture demands it (open question §8.1 resolved by fixture, not guess).
- Uniqueness/mono-style constraints (injective arrows): **not** in the v1
  kernel. The P6 fixture set is therefore scoped to
  path-equation-expressible findings; the expressibility gap vs full ACset
  invariants (kata-08/09 used some injectivity-flavored constraints) is
  recorded as a known limitation in the ledger, not silently papered over.
- `check(schema, instance) -> Vec<Finding>` executes every equation on every
  applicable row (E3). Empty schema or empty instance → zero findings, OK.
  Self-loop arrows are legal.
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*`; duplicates in objects/arrows or
  equations referencing unknown arrows are parse-time errors with
  line/column (golden malformed fixtures in P0). Equations are type-checked
  at parse time (path endpoints must match).
- `Migration`: Δ (reindex along a schema map) in v1; Σ/Π only after Δ is
  validated (E4: migration is the unique capability axis — it earns its
  place; Π does not yet).
- Model document format: `.ct.json`, versioned via `format_version` from
  day one. Policy: additive changes are minor versions; breaking changes are
  a new extension, never in-place mutation of v1. A JSON Schema validation
  artifact is published alongside for editors/agents.

### 4.2 Analysis (deterministic only)

- Bounded instance search: small finite models satisfying/violating
  equations. Search bounds are explicit inputs; results are a witness or
  `UNKNOWN { scope }` — never a naked "no violation found".
- Counterexample minimality: smallest violating instance within bounds.
- Redundancy: path-equivalence via normal-form rewriting under declared
  equations.
- `distinguish`: bounded behavioral distinguishability of two models over an
  explicit observation set (not schema equivalence — that is undecidable in
  general and out of scope).

### 4.3 Inquiry (the thinking layer)

The **epistemic layer** (E5): every result carries not just a value but a
*justification status* — what was proved, what was executed within a scope,
what is merely hypothesized. Deterministic operations in v1:
`parallelPaths`, `consequences`, `counterexample`, `redundancy`,
`distinguish`, `impact`. The heuristic scaffolds (`alternatives`,
`questions`, `compare`) emit *hypotheses with rationale*, never deductions
(E5: "LLMs should propose; deterministic tools evaluate") — they stay
spec-only until P6 validates the deterministic ops (open question §8.2).
`UNKNOWN` is a first-class result, not a failure.

## 5. Validation plan (reuse the ct-catlab harness discipline)

- **Negative control first**: FizzBuzz kata fixture must produce the verdict
  "categorical modeling adds nothing here" (E4: kata-6 killed). If the tool
  cannot say that, it is a ceremonial math generator, not a workbench.
- **Port 3 validated katas as browser fixtures**: Bowling (temporal
  semantics — expected partial: history-dependence is a validated CT limit),
  Gilded Rose (state-shape invariants), Vending Machine (strongest validated
  structural result, 6/6 ≍ 6/6). Success = the tool reproduces the validated
  findings that are expressible as path equations + totality, finding-typed,
  without Julia; the expressibility gap is recorded, not hidden (§4.1).
- **Agent-loop walkthrough (the P3/P6 acceptance story)**:

  ```
  $ catherine check bowling.ct.json
  violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item
    at row Assignment#3: lhs=Unit#1 rhs=Unit#2
  underconstraint: parallel paths Assignment→Unit via staff/equipment
    have no declared equation? (hypothesis, domain question)
  $ # agent proposes equation, tool re-checks, ledger updated — repeat
  ```

- **Parity gate on extraction**: pieces ported from specodelic acset
  (instance tables, query closures) must show identical findings on the
  specodelic corpus before rewrite (E8 pattern); property tests stay green.
- **Evaluation dimensions** (from E5 §8): requirements discovery, injected
  defect detection, complexity, cognitive load, explainability, composition,
  effort — measured against the same scenarios with and without the tool;
  custody discipline per ct-catlab (disclosed single-author is acceptable,
  claimed independence is not).

## 6. Phased plan (TDD; one red→green→refactor cycle per ticket)

Each phase is one beads ticket (`bd`), one red→green→refactor cycle, one
ledger entry after the phase's validation run (verdict vocabulary: killed /
survived / inconclusive / invalid-run) — ct-catlab record discipline applied
to the tool itself.

1. **P0 kernel skeleton**: model document format + parser + round-trip
   property test. Red: malformed presentations rejected with line-accurate
   errors (golden fixtures incl. duplicate idents, unknown arrows, ill-typed
   equations).
2. **P1 instance + executed checks**: instance tables, dangling-as-value,
   path-equality execution with witness rows. This is the E3 bet; if
   nothing else ships, this must.
3. **P2 findings taxonomy**: violation/dangling/redundant as an enum with
   witnesses; golden fixtures per finding type.
4. **P3 CLI**: `catherine check model.ct.json` + `catherine findings` for
   agent use; exit codes mirror finding classes; JSON and human output.
5. **P4 analysis**: bounded search + counterexamples + redundancy +
   `underconstraint` detection; scope-honest `UNKNOWN`.
6. **P5 wasm + minimal web UI**: static site, load/save `.ct.json`, SVG
   schema graph, findings panel; no editing UI beyond a JSON textarea in v1;
   row-count cap at the boundary with a typed error.
7. **P6 kata validation**: the three ported katas + negative control,
   results recorded in the ledger. **This is the project stop condition.**
8. **P7 migration axis**: Δ only, with loud lost-data counterexamples
   (kata-10 pattern) as fixtures.

## 7. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Scope creep toward Catlab (rewriting, sketches, fibrations) | Non-goals §2 are release criteria, not aspirations; every PR maps to P0–P7 |
| The UI grows into the product before the kernel is validated | P5 is *after* CLI; web is a viewer until P6 passes |
| WASM/TS boundary leaks Rust types into ad-hoc JS | single `wasm-bindgen` surface, JSON-only payloads, TS types generated from the model document schema |
| Repeating Catlab's enforcement gap (E3) | checks are the core deliverable (P1/P2); equations without execution = bug |
| Overclaiming (the corpus's own failure mode: verdict/record contradictions, ct-catlab REPORT methodology §1) | ledger + validator-style checks from day one; finding labels mandatory |
| Agent ergonomics sacrificed for human UI | CLI is a first-class artifact (P3), not an afterthought; regularity over type ceremony (E7) |
| Kernel expressiveness gap vs validated kata invariants | scoped P6 fixtures + recorded limitation (§4.1), not silent failure |

## 8. Open questions

1. Attribute alphabet: is string/int/bool enough for the kata fixtures, or do
   we need enums at the schema level in v1? (Lean: defer until a fixture
   demands it.)
2. Should `alternatives`/`questions` scaffolds ship in v1 at all, or stay
   spec-only until the deterministic ops are validated by P6? (Current lean:
   spec-only.)
3. Naming: crate prefix `catherine-*` vs shorter binary name (`ctn`?).
4. Repo layout confirmed: this repo (`at/catherine`) is home for crates +
   web; openspec change for P0 to be created when design is approved.
