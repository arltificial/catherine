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
