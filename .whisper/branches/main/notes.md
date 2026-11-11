- 2026-11-11T00:09:36Z [id:be553ad04993b42c4dea41874130addd0c120559079e00f76a0211a4911c02bb] Session 2026-11-10/11 (tracked summary; detailed decision records in private zone):
  - Investigation: Michael Johnson CT conversations traced (chatgpt 6adf915c origin -> ct-kata01 topic -> matadero ct-catlab series) -> validated boundary E1-E9 distilled to docs/PROPOSAL.md.
  - Design converged via Rule-of-5: docs/DESIGN.md v2 (kernel, findings taxonomy, Rust+WASM+genesis CLI, P0-P9).
  - Rule lens added as P8 (rules = subobjects + resolution policy; kata-6 reframed as triviality-floor claim; HYPOTHESIZED with pre-registered validation).
  - genesis-vibes adopted for CLI only (P3); bogart = pilot corpus (P9): FTS-anchor equation, identity-domain disjointness, capture selector overlap/gap, raw-vs-derived migration, alerts tiered rules.
  - Theory doc: docs/THEORY.md (constructions, what they buy, validated negatives, reading order, glossary).
  - Tracking: 10 beads tickets exported fresh; ledger discipline inherited from ct-catlab; whisper decisions in private zone.
- 2026-11-11T01:07:30Z [id:db0b413e75618c73f71f688dd713311d0539093d0fd84155a420eb3fada822a0] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-11T00:00:00Z (session started 2026-11-10T23:03Z)
  Repo: arltificial/catherine (private, github, host alias git@at)
  Branch: main
  HEAD: 34da242 (docs: ecosystem reuse pass wild+specodelic)
  Session: 01a1c7b3-6455-77b1-9137-b19a0b6d5418
  Task: catherine — CT modeling workbench: investigation, design, docs, tracking
  
  Objective: design (not yet build) a browser-runnable CT modeling kernel simpler than Catlab, agent-first, validated against ct-catlab evidence.
  
  Current phase: design complete, docs converged (Rule-of-5 x3), tracking wired; implementation NOT started.
  
  Current state:
  - Docs: README, PROPOSAL, THEORY, DESIGN (all cross-linked, gates green); docs/ref/ methodology vendored.
  - Beads: 10 tickets P0-P9 full chain (P0=catherine-bpz only unblocked); export fresh (.beads/issues.jsonl tracked).
  - Reuse map in DESIGN §3: specodelic acset instance/query port behind parity gates; acset-core openspec contracts = P0 spec base (adapt, don't author); writer.rs DROPPED (~631 LOC); path-equation execution is NEW code (verified specodelic has none); wild formats.rs canonical-JSON+sha256 ports to model hashing + P6 ledger content-addressing.
  - License switched MIT -> Apache-2.0 pre-code (donors specodelic/wild/genesis all Apache).
  - whisper: decisions in private zone (topics: ct-tooling, rule-lens, tooling); branch note tracked.
  
  Exact next action: start P0 (catherine-bpz): import specodelic acset-core contracts as spec base, scaffold Rust workspace (crates/catherine-core), write .ct.json format + parser with round-trip proptest; RED = malformed-presentation golden fixtures (duplicate idents, unknown arrows, ill-typed equations, line-accurate errors).
  
  Key decisions/constraints:
  - Executed checks are the product (Catlab equations are docs-only — E3). Kernel crates + wasm stay genesis-free; genesis-vibes only in CLI (P3), stable modules only.
  - Five executed findings in v1 (violation/dangling/contradiction/underconstraint/redundant); missing + unupdatable post-v1; rule lens P8 HYPOTHESIZED (subobjects + resolution policy), gated on P6 recording schema-lens FizzBuzz negative control FIRST (no goalpost moves).
  - P6 is the project stop condition: killed verdict => stop, record negative result.
  - Non-goals are release criteria: no prover/rewriting/diagrams/DB/server; UNKNOWN with scope is first-class; proptest not insta-snapshots.
  - Bogart pilot P9 (catherine-hc6, gated P3+P4): 5 mappings in DESIGN §5 (FTS-anchor equation, identity-domain disjointness, capture selector overlap/gap, raw-vs-derived migration, alerts tiered rules).
  
  Ticket ownership: all 10 tickets open, unassigned, single-author (sasha); no third-party custody claims.
  
  Validation: ah check clean; ddl doctor repo-level clean (global: funkalization install fails, ~/.ddl/config.toml pending — machine-level, not blockers); gates green on every commit; beads export verified fresh.
  
  Unresolved/blockers: none for P0. Open questions in DESIGN §8: naming (catherine-* vs short bin), scaffolds ship-in-v1 (lean: spec-only).
  
  Refs: DESIGN.md §3 (reuse map), §4.1 (kernel types), §5 (pilot table), §6 (phases P0-P9), §8 (open questions); PROPOSAL §4 (value prop); THEORY (constructions + negatives).
- 2026-11-11T13:59:57Z [id:467fe93dd1e21f328878949e11814612a06d8de33a3fba709c5603a1b1b5f148] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-11T13:59:37Z
  Repo: arltificial/catherine (private, github, host alias git@at)
  Branch: main
  HEAD: 9e52e78 (spec: P0-P9 openspec change proposals + rule-of-5/issue-review fixes)
  Session: 01a1ca76-992e-77b1-9137-b19cefaf1211
  Task: catherine — CT modeling workbench: proposals authored, reviewed, fixed; implementation not started
  Previous entry: db0b413e75618c73f71f688dd713311d0539093d0fd84155a420eb3fada822a0
  
  Objective: build catherine (browser-runnable CT modeling kernel, executed checks, finding-typed output) per docs/DESIGN.md; this session turned the converged design into executable change proposals and fixed every review finding.
  
  Current phase: commit (proposals + review fixes + beads metadata committed as 9e52e78). Implementation NOT started.
  
  Current state:
  - 10 openspec changes under openspec/changes/, one per beads ticket P0-P9, ALL openspec validate --strict green: add-model-document-kernel (P0/bpz), add-executed-checks (P1/0gk), add-findings-taxonomy (P2/1x7), add-check-cli (P3/86i), add-bounded-analysis (P4/1ay), add-wasm-web-view (P5/9b5), add-kata-validation-ledger (P6/9vv), add-delta-migration (P7/alr), add-rule-lens (P8/nq5, HYPOTHESIZED), add-bogart-pilot (P9/hc6).
  - Rule-of-5 + tracer-bullet review done; 5 HIGH findings TypeSafe-verified (T1 0.87, T2 0.88, T4 0.84; T3/T5 verified but below 0.8 gate — mechanically backed, fixed anyway): exit-code precedence; P6 verdict = derived from tool findings via recorded procedure (tool emits findings, not verdicts); JSON Schema scoped to syntactic surface; underconstraint witness = tagged union ParallelPaths|UncoveredInputs (P8 gap maps onto UncoveredInputs); bogart pilot phased (models 1-3 at P3+P4, 4 post-P7, 5 post-P8). Also: parity gate mechanics (fixtures/parity/), canonical row-id normalization for hashing, ident-charset/empty-doc/zero-bound/map-totality edge cases, kata roster grounded in DESIGN §5 from matadero SERIES.md (kata-1 Bowling, kata-2 Gilded Rose, kata-3 Vending, kata-6 FizzBuzz), kata-08→kata-8 numbering normalized, evidence-ID legends in all proposals.
  - Beads: all 10 tickets tagged (files, base_commit 9e52e78, lane 9x afk + hc6 review, complexity, openspec_change, anti_goals); 9vv description rewritten (ledger/ path, verdict procedure); hc6 rewritten (phasing, pinned-rev CI distribution); nq5 carries P7→P8 ordering rationale. Export committed.
  - ah check overlay: expected no-toml findings only (8 scenarios in model-document) — contracts are born at apply time via `ah scenario new`.
  
  Exact next action: claim catherine-bpz, apply add-model-document-kernel: `ah scenario new` per model-document scenario → RED golden fixtures (duplicate idents, unknown arrows, ill-typed equations, invalid ident charset, bad status, unsupported version; line/column errors) → GREEN (parse, canonical serialization byte-stable, wild formats.rs canonical-JSON+sha256 port, round-trip proptest + row-id-canonicalization property) → REFACTOR (shared error type; NOTICE attribution).
  
  Key constraints: E3 executed checks are the product; kernel crates genesis-free (genesis-vibes in CLI only, stable modules); adapt specodelic acset-core contracts, don't author; proptest not insta-snapshots; Apache-2.0, NOTICE preserved on ported code; ConstraintStatus semantics deferred to P1 (P0 fixes format field only: enum {proposed,accepted}, default accepted); P6 is the stop condition; no claims of detection-rate wins.
  
  Ticket ownership: all 10 open, unassigned, single-author (sasha); catherine-bpz the only ready.
  
  Validation: openspec validate --strict 10/10 green; bd ready = catherine-bpz; ah check no-toml expected; commit 9e52e78 clean tree.
  
  Unresolved: none blocking P0. DESIGN §8 open questions (naming catherine-* vs short bin; scaffolds spec-only lean) unchanged, non-gating. Review items accepted-not-fixed: DRAFT-001 (no change index; openspec list suffices), SCOPE-002 (P2 preparatory taxonomy — justified API stabilization).
  
  Refs: openspec/changes/*/proposal.md + design.md (P0, P6, P8); docs/DESIGN.md §5 kata roster table; review reports in session 01a1ca76 transcript.
