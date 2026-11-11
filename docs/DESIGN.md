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
| E10 | **wild** (`charly/wild`) models software contracts as a *category* in-suite: accretion between revisions is a thin category (preorder) with checkable laws, demand-restriction is a functor, identity is canonical-JSON + sha256 hashing — implemented in Rust (`formats.rs` canonicalizer is byte-for-byte normative; `extract`/`check` runtime-tested). Compose/registry tiers are normative-only (4 scenarios). Specodelic acset has cycle/reachability machinery but **no path-equation execution** — that remains genuinely new code | `wild/README.md`, `wild/openspec/specs/wild-core/spec.md`, `wild/src/formats.rs` |

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
**License: Apache-2.0** — matches specodelic/wild/genesis, the code donors
(E2, E7, E10); extracted Apache-2.0 code cannot be relicensed to MIT.

### Reuse map (what ports, what is new)

| Source (Apache-2.0) | → catherine | Gate |
|---|---|---|
| specodelic `src/acset/instance.rs` (925 LOC) | kernel instance tables | graph-parity on specodelic corpus (E8) |
| specodelic `src/acset/query.rs` (264 LOC) | analysis traversals (reachability, fan-in) | parity + property tests |
| specodelic acset openspec contracts (`acset-core`, `acset-writer`) | P0 spec base — executable contracts, adapt not author | `ah check` green |
| wild `src/formats.rs` canonical-JSON + sha256 | model-document hashing; ledger content-addressing (P6) | byte-for-byte normative port + tests |
| wild accretion-laws discipline (identity/canonical/invariant table) | `.ct.json` format_version policy | schema-review checklist |
| specodelic `src/acset/writer.rs` (631 LOC) | **dropped** — no lossless text round-trip needed (JSON model docs, not markdown) | n/a — scope reduction |
| specodelic `src/acset/schema.rs` (946 LOC) | **partial** — no equations exist there; path-equation presentation is new code | new property tests |

Simulation-before-Rust pattern (wild `prototype/wild_cat_sim.py`):
pre-implement P8 rule-lens laws as a Python sim to de-risk the semantics.

**Suite plumbing: `genesis-vibes` — CLI crate only.** The charly-vibes
cross-cutting foundation (stable, crates.io `0.13`) owns the JSON envelope
(`ok/envelope_kind/data/warnings/hints` — the exact envelope every suite
agent already speaks), verbosity/format clap structs (`guide`), doctor,
suggestions, and config. Adopted for `catherine-cli` (P3) so agents get
identical envelope semantics to turu/whisper/ddl. The kernel crates
(core/analysis/inquiry) and `catherine-wasm` stay **genesis-free** — domain
logic per genesis's boundary rule, and keeps the kernel portable/pure for
WASM. Stable modules only for v1 (the `new` ones — doctor/cli/status —
wait until they graduate).

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

Dependency direction: `cli`/`wasm` → `inquiry` → `analysis` → `core`;
`catherine-cli` additionally consumes `genesis-vibes` (envelope/guide only
— never leaks into the kernel). The web UI depends only on the wasm exports
and the model document format — it is never the source of truth (E5).

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
  invariants (kata-8/9 used some injectivity-flavored constraints) is
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

### 4.4 Rule lens (P8, post-v1 — the HYPOTHESIZED capability)

kata-6 killed the *schema-lens* rule table at the triviality floor (7-line
conditional; checks fired, cost didn't) — it did not refute categorical
structure in rules. The rule lens models rule systems directly:

```rust
pub struct RulePresentation {
    pub domain: Ident,                 // X: the input object
    pub rules: Vec<Rule>,
    pub policy: Policy,                // PriorityOrder | FirstMatch | DeclaredDisjoint
}
pub struct Rule {
    pub predicate: Predicate,          // decidable expr over attribute columns
    pub action: String,
    pub order: usize,                  // position in the priority chain
}
```

Semantics: rules are **subobjects** R_i of the input domain X; the action
assignment must become a total function on X; the **resolution policy** is
order-enrichment on the family. Effective region E_i = R_i \\ ⋃_{j<i} R_j.

Findings (each maps onto an existing finding kind, so no new epistemic
machinery is needed):

| Finding | Condition | Bug class |
|---|---|---|
| gap | ⋃R_i ⊊ X | input class with no rule — partiality explicit |
| overlap | R_i ∩ R_j ≠ ∅, no policy for the intersection | nondeterminism ("who wins?") |
| shadow | E_i = ∅ | dead rule, derivable from earlier rules + policy |
| derivable | R_k = R_i ∩ R_j (boolean combination) | simplification candidate |

Two properties make this more than a linter:

1. **Equivalence-checkable simplification.** Two rule tables are equivalent
   iff they induce the same total action on X; `distinguish` produces a
   witness input where they differ. "Collapsed 3 rules into 2 + precedence,
   no distinguishing witness in-domain" is compliance, proven — FizzBuzz's
   FizzBuzz-rule is derivable from Fizz ∧ Buzz + precedence.
2. **Rule discovery support.** Examples (input→output pairs) + a declared
   predicate vocabulary: candidates come from the agent/human, the tool
   checks partition equivalence against the examples; uncovered examples
   are gap findings. C5's loop applied to rule tables.

For enumerable X with decidable predicates every finding is exact via the
P4 bounded-search machinery — the rule lens is a presentation over the
same kernel, not a new engine. This also resolves the attribute-alphabet
open question in the rules' favor: predicates require int/bool/string
comparisons, ∧ ∨ ¬, mod — a tiny decidable expression language, not a
logic.

**Epistemic status: HYPOTHESIZED.** Precedent above the floor exists (Rust
match exhaustiveness/reachability; DMN hit-policy conflict detection) but
no ct-catlab-class evidence does. Validation (P8): one kata above the
floor (10–20 compound rules + priority policy, e.g. pricing/eligibility
table) + one second negative control near the floor. Killed if unique
detections don't appear above the floor.

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

  Kata roster (grounded: `ak/matadero/series/ct-catlab/SERIES.md` — the
  canonical numbering this repo uses):

  | Kata | Domain | ct-catlab verdict |
  |---|---|---|
  | kata-1 | Bowling — temporal semantics, construction-time inexpressibility | survived (scoped) |
  | kata-2 | Gilded Rose — frozen-quality + post-concert invariants | survived (scoped) |
  | kata-3 | Vending Machine — bounds, command vocabulary, collision | survived (scoped) |
  | kata-4 | bank ledger shape invariants | survived (scoped) |
  | kata-5 | Bowling win-legality — history-dependent | survived (scoped) |
  | kata-6 | FizzBuzz — negative control | killed |
  | kata-7 | roman numerals — canonical round-trip | survived (scoped) |
  | kata-8 | spec-ambiguity discovery (holdout failure exposed a spec ambiguity) | survived |
  | kata-9 | delegated implementation — shipment↔reservation ACset invariants | survived |
  | kata-10 | migration axis — lost-data + unsupported-inverse | survived (split) |

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
- **Pilot corpus — bogart** (`at/bogart`, P9): the first real-domain
  dogfood. Bogart's openspec prose invariants map directly onto kernel
  findings, demonstrating the tool on a live repo beyond toy katas:

  | Bogart spec invariant (prose) | Catherine model | Finding class |
  |---|---|---|
  | "every FTS hit resolves to text_blocks rows carrying edition and page anchors" (corpus-storage) | equation: `block → document → edition = block → edition`; totality of `block → document` | violation (drifted denormalized anchor) + dangling |
  | "a section page never fabricates an aviso identity" (stable_ids) | identity-domain disjointness (aviso vs section-acquisition id formats) as a rule-lens table (P8 preview) | overlap/shadow on identity domains |
  | `capture_ambiguous` / `capture_unavailable` (backfill selector) | selector→capture relation: multiple matches without tie-break policy = overlap; no match = gap | overlap + gap |
  | raw working corpus vs derived serving projection | schema map + Δ migration; lost-data counterexamples | migration axis (P7) |
  | "FTS layer always runs, structured layer on identity-covered docs, `layer` records which fired" (alerts-matching) | tiered rule table with priority policy + declared coverage | rule lens (P8) full example |

  Bogart is Clojure — the pilot also demonstrates language-neutrality
  (a Clojure repo's invariants, checked by a Rust kernel over JSON).
  Deliverable: `.ct.json` models checked in bogart's CI (`catherine check`
  exits per finding class) + a pilot report recording which findings were
  real vs noise.

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

1. P0 kernel skeleton: model document format + parser + round-trip
   property test, **starting from specodelic's acset-core executable
   contracts** (adapt, don't author). Red: malformed presentations rejected
   with line-accurate errors (golden fixtures incl. duplicate idents, unknown
   arrows, ill-typed equations).
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
9. **P8 rule lens** (gated on P6 having recorded the schema-lens negative
   control first — no goalpost moves mid-experiment): rule presentation,
   findings, equivalence checking, with its own two-kata validation as
   pre-registered in §4.4. Killed honestly if it earns nothing above the
   floor.
10. **P9 bogart pilot** (gated on P3 CLI + P4 findings): port the pilot
   table above into real `.ct.json` models under bogart's CI; record which
   findings fired on the live corpus and which were noise — the first
   evidence outside the kata lab.

## 7. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Scope creep toward Catlab (rewriting, sketches, fibrations) | Non-goals §2 are release criteria, not aspirations; every PR maps to P0–P7 |
| The UI grows into the product before the kernel is validated | P5 is *after* CLI; web is a viewer until P6 passes |
| WASM/TS boundary leaks Rust types into ad-hoc JS | single `wasm-bindgen` surface, JSON-only payloads, TS types generated from the model document schema |
| Repeating Catlab's enforcement gap (E3) | checks are the core deliverable (P1/P2); equations without execution = bug |
| Overclaiming (the corpus's own failure mode: verdict/record contradictions, ct-catlab REPORT methodology §1) | ledger + validator-style checks from day one; findings labels mandatory; every record content-addressed (canonical-JSON + sha256, E10) |
| License drift across extracted Apache-2.0 code | repo is Apache-2.0 (matches donors); NOTICE/attribution preserved on ported files |
| Reuse circularity (catherine kernel flowing back into specodelic) | one-way extraction for v1; flow-back is a separate decision after P6, not a design assumption |
| Agent ergonomics sacrificed for human UI | CLI is a first-class artifact (P3), not an afterthought; regularity over type ceremony (E7) |
| Kernel expressiveness gap vs validated kata invariants | scoped P6 fixtures + recorded limitation (§4.1), not silent failure |

## 8. Open questions

1. ~~Attribute alphabet~~ **resolved by the P8 rule lens** — predicates
   require int/bool/string comparisons + ∧ ∨ ¬ + mod, so attribute columns
   with a decidable expression language enter the kernel design (P8 scope;
   v1 kata fixtures don't demand them before that).
2. ~~License~~ **resolved: Apache-2.0** (matches code donors; was MIT before
   any code existed — zero-cost switch).
2. Should `alternatives`/`questions` scaffolds ship in v1 at all, or stay
   spec-only until the deterministic ops are validated by P6? (Current lean:
   spec-only.)
3. Naming: crate prefix `catherine-*` vs shorter binary name (`ctn`?).
4. Repo layout confirmed: this repo (`at/catherine`) is home for crates +
   web; openspec change for P0 to be created when design is approved.
