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
- 2026-11-11T18:52:56Z [id:a7406fee8a17d21ba14ebb7e6928da7e54dbd4fc7494a464f0b961f2751e5623] (#resume-catherine-methodology-docs) Format: 1
  UTC: 2026-11-11T15:55:00Z
  Repo: at/arltificial/catherine (~/para/areas/dev/gh/at/catherine)
  Branch: main · HEAD 2206ddd · pushed
  Session: pi 01a1caee-7be2-77b1-9137-b19e99139bbe
  Previous entry: none
  
  OBJECTIVE: corpus-grounded CT methodology docs for catherine + expansion triage.
  PHASE: commit (docs pushed; nothing in flight).
  
  STATE (all committed/pushed to origin/main):
  - docs/USING.md — agent recipes, human flow, enablement (E-licensed), honest boundary; Johnson fixes landed (sketch-cone boundary, kata-8 "echoed" wording, view-update line). Examples ILLUSTRATED until P3 captures runs.
  - docs/ref/ct_methodology_playbook.md — tool-free drills by problem type (§1–§7), NEW §8 reverse-engineering (top-down+bottom-up reconciliation: phantom requirements / undocumented behavior / inconsistencies), §9 when-not, §10 stopping; bias/amnesia review fixes (negative-result license, cost in header, dispute protocol, four-region drill naming, monoid gloss).
  - PDF: ~/Downloads/ct_methodology_playbook.pdf (6pp A4, pandoc+weasyprint).
  - Beads: catherine-dvx (P10 schema-diff, blocked-by alr+9vv, E4-licensed, ≤400 LOC), catherine-ga7 (bogart emits .ct.json, blocked-by hc6, external_repo at/bogart); memories expansion-licensing + program-boundary; hc6 closure-semantics note; alr/nq5 re-anchored.
  - NOT filed (rejected): P12 property-oracle (unlicensed per E4), P13 claims ledger (persistence OOS + bajan boundary).
  - P3 CLI (catherine-86i) implemented by orchestrator subagent — committed+pushed as feat(cli) 2206ddd (not validated by this session).
  
  NEXT ACTIONS (unblock-time, none urgent):
  1. When P6+P7 land: scaffold openspec change add-schema-diff for catherine-dvx.
  2. Before catherine-ga7 starts: add quantified Must gate to seeded-defect validation (e.g. N/N injected mismatches detected with witnesses) + file-headers on new bogart files.
  3. Vendor Johnson papers/DOI links under docs/ref/ (EDGE-001, open).
  4. USING.md: flip ILLUSTRATED banner to captured runs once P3 CLI is verified.
  5. Open LOW polish: J&D "useful redundancy" nuance; "relational schema with executable semantics" framing.
  
  VALIDATION: docs reviewed via rule-of-5 (USING vs Johnson: NEEDS_REVISION→fixed; playbook bias/amnesia: fixes applied); git push OK 3fe1b62..2206ddd. P3 code NOT validated here.
  
  CONSTRAINTS: corpus ~/cotorreo read-only (AGENTS.md contract; cite conv IDs, never mtimes); catherine emits verdicts only (program-boundary memory); expansion tickets need cited E-license + P6 gate.
  
  UNKNOWN: whether P3 run left tasks.md checkboxes accurate; whether ~/Downloads PDF regeneration wanted after future doc edits.
- 2026-11-11T22:09:14Z [id:69456a7f5efdd142faa33df8f9c60244182868274d0023fa803f525216ea955a] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-11T22:08:52Z
  Repo: arltificial/catherine (private, github, host alias git@at)
  Branch: main
  HEAD: 00ecf43 (feat(analysis) P4 + export refresh)
  Session: unknown (omp orchestrator session; full per-ticket trails in .wai/projects/catherine/)
  Task: catherine — CT modeling workbench: P0-P4 implemented and gate-shipped; chain 5/10
  Previous entry: 467fe93dd1e21f328878949e11814612a06d8de33a3fba709c5603a1b1b5f148
  
  Objective: build catherine (browser-runnable CT modeling kernel, executed checks, finding-typed output) per docs/DESIGN.md. This session orchestrated P0-P4 end-to-end via epic-orchestrator (wai pipeline) with tdd-ro5 discipline embedded in committed per-ticket briefs.
  
  Current phase: between runs (P4 ship-complete; STOP boundary honored). Implementation THROUGH P4 done; P5-P9 + hc6 pilot not started.
  
  Current state:
  - Shipped: P0 kernel (f2c27a8), P1 executed checks (809be72), P2 findings taxonomy (dc4c95e), P3 CLI (b427810 user commit + 0a5ec98 review closure), P4 bounded analysis + gate remediation union (e324f30) + export (00ecf43). Beads closed: bpz/0gk/1x7/86i/1ay; export fresh at 00ecf43.
  - Working CLI: cargo run -p catherine-cli -- check|findings|search|redundant|underconstraint|distinguish|query; exit codes 0/1/2/3/4/5/6/7 (min-over-class; exit table byte-pinned by test).
  - pretender.toml switched mode=gate (user decision): every future commit gated on lint-pattern compliance (abc/lines/cyclomatic/cognitive/nesting/file_lines); 73 violation sites remediated to green this session.
  - rename-invariance now a proven property: canonical/label.rs tie-break fixed (order-free), persisted proptest regression kept; P0 CORR-001 documented-risk closed; parse/ assigns RowIds sorted-name order (kept deliberately).
  - models.yml cap: ~/.omp/agent/models.yml caps openrouter z-ai/glm-5.3-flash maxTokens at 12288 (credit-survival). Cap binds at SESSION CREATION — resume-after-change keeps stale 131072 window; spawn FRESH agents after config changes. Raise/remove cap after real OpenRouter top-up.
  - OpenRouter credits exhausted 3x this session; balance was ~12k affordable at last check. Top up before next run.
  
  Exact next action: fresh `wai pipeline start epic-orchestrator` for P5 (catherine-9b5: wasm + minimal web UI; only main-chain ready ticket). hc6 (bogart pilot models 1-3, gates P3+P4) ALSO armed — user picks order at STOP pacing (user says continue per ticket). Orchestration pattern per ticket: claim bd + pipeline start + gates baseline + planner task writes brief (committed) + ONE serial implementor spawn per brief + lead-side verify (REVIEW_REQUIRED) + ship checkpoint (user approval) + bd close + export + STOP.
  
  Key constraints: user approves every commit (ship checkpoint); push never without explicit instruction; bd close+export orchestrator-reserved (export: `bd export > .beads/issues.jsonl` then `git add -f` — global gitignore covers .beads); kernel crates genesis-free (genesis-vibes in catherine-cli only); findings never without witnesses; UNKNOWN{scope} never collapsed; proptest not snapshots; model roles all glm-5.3-flash parent default (user dropped deepseek routing; qwencloud unreachable in omp).
  
  Ticket ownership: 9b5 + hc6 ready/unassigned (single-author sasha); user filed P10 catherine-dvx + catherine-ga7 adjunct (out of chain scope, unassigned).
  
  Validation: at 00ecf43 all green — fmt 0, clippy 0, workspace tests 0 (95+), pretender gate 0, openspec validate --strict 10/10, just ci 0, DESIGN §5 walkthrough byte-identical (exit 3/0), scope-honesty probes (contradiction exit 5 only after real exhaustion; budget-2 -> UNKNOWN+scope exit 0; zero rows -> UNKNOWN empty scope).
  
  Unresolved/blockers: OpenRouter balance low (top up before P5; cap makes refill last ~10x). /tmp tmpfs near quota — just ci needs TMPDIR=$PWD/target/tmp. CHANGELOG Unreleased started at P3 (keep ticking per ticket). pretender gate will bite any future unremediated code.
  
  Refs: .wai/projects/catherine/runs/{catherine-bpz,0gk,1x7,86i,1ay}.state (checkpoint trails); briefs in .wai/projects/catherine/briefs/; docs/DESIGN.md §6 phases + §5 walkthrough; openspec/changes/* (P5=add-wasm-web-view, P6=add-kata-validation-ledger STOP CONDITION, P8=add-rule-lens HYPOTHESIZED).
- 2026-11-11T22:16:31Z [id:8b97052adcb6f53b849b213385b7abfe08c86ecf4a4a4c884d829aace11652c6] (#failure-taxonomy) catherine review failure taxonomy (5 classes, gates filed): A fail-open boundaries at layer seams (silent drop/reinterpret -> labeled error or warning; gate: negative-matrix test catherine-348) | B prose-code contract drift (docstring claims untested; gate: ah scenarios for module claims catherine-za6) | C documented distinctions without fixture coverage (gate: variant-coverage ratchet catherine-3zs) | D un-mechanized soundness assumptions (gate: property ratchets catherine-1ml) | E toolchain regeneration erodes invariants (gate: self-defending hook check catherine-u6y). Meta-pattern: all bugs lived where new layers met the kernel; kernel itself fail-closed. Rule: never bare-edit generated files — pair with doctor check or upstream.
- 2026-11-12T00:51:25Z [id:84ea7114ac118c4834e730e7f702783aa6133205e61aaaa00ca478a0f236922a] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-12T00:51:25Z
  Repo: arltificial/catherine (private, github, host alias git@at)
  Branch: main
  HEAD: 268c142 (ci: pinned toolchain workflow + standard-conformance gate)
  Session: omp orchestrator (this conversation; waves delegated to task agents)
  Task: catherine — CT modeling workbench: P0 bug wave + P1 prevention/gates + uh0 + CI all closed and committed
  Previous entry: 69456a7f5efdd142faa33df8f9c60244182868274d0023fa803f525216ea955a
  
  Objective: continue the catherine build per docs/DESIGN.md; this session closed every bug ticket born from the P0/P1 fix reviews and landed the CI gate.
  Current phase: commit (all landed; nothing in flight).
  
  Current state:
  - Session commits: 9baca0e 51v KNOWN_MINOR gate + spec amend; 8f3fbcc 18t schema_mismatch + 0kw rows warning; 18c37a4 u3k is_clean accepted-only; a748895 u6y hook-swallow-guard + pre-push ah check --run-tests + .espectacular/state gitignored; ef25028 348 negative_matrix (25 rows, 7 surfaces) + query --follow warning (new bug found+fixed); 518ffef za6 docs-as-contracts scenarios/downgrades; 2c4e6a5 uh0 stale version hint + human-mode warnings (print_run_warnings); 7213481 fmt drift rewrap; 268c142 6p5 CI.
  - Tickets CLOSED: 18t u3k 0kw 51v u6y 348 za6 uh0 6p5. biv CLOSED by decision reversal (SemVer stays for crates.io; wild shadow-checks only via re-file — do NOT implement without new ticket).
  - Gates now: lefthook pre-commit = bd + ah check + pretender (40/400 line caps) + spk-lint + hook-swallow-guard (fails commits if bd init regen strips _lh_exit capture in .beads/hooks/* — remediation text printed by the gate); pre-push adds ah check --run-tests. CI: .github/workflows/ci.yml → scripts/ci-install.sh (pins read from versions.ddl.toml; bd sha256-verified) + just ci-full (fmt/clippy -D warnings/tests + openspec validate --strict + ah check). Standard-conformance test pins pin-syntax + issues.jsonl integrity (bd has no no-db validation — documented fallback).
  - Conventions established: unknown_rows_warning/unknown_follow_warning on warnings channel + print_run_warnings human mode; KNOWN_MINOR=0 in parse/schema.rs (raise when additive semantics land); docs-as-contracts practice in .wai/resources/reflections/module-header-claims-are-contracts.md; negative-matrix rule in docs/ro5u-checklist.md (new flag/surface without a matrix row is incomplete).
  - Remaining open: 9b5 (P1, wasm+web UI feature), P2: hc6 pilot, 7e8 transitive underconstraint, mrq rewrite docs, 1ml WL ratchet, 3zs prevention-C, w6z vampiro adoption. Concurrent omp session exists: it lands beads-export chore commits (7a7a61f, fb2f21c pattern) — leave .beads/issues.jsonl dirty to it; export refreshed at 27 issues.
  
  Ticket ownership: all open tickets unassigned, single-author (sasha); no third-party custody.
  
  Validation: cargo test --workspace green; ah check 0/0; just ci-full green; lefthook validate clean; actionlint clean; all commits passed full pre-commit gate.
  
  Unresolved/blockers: ci.yml never ran on a real runner — first push to main is its first live run; runner-specific failures fix in scripts/ci-install.sh. Upstream beads issue (bd init regen swallows exit codes) drafted, NOT filed.
  
  Exact next action: user decides lane — push main (first live CI run) and/or start 9b5 or a P2 ticket. Push requires explicit user authorization.
  
  Refs: docs/DESIGN.md §6 phases; openspec/changes/*; docs/ro5u-checklist.md; .wai/resources/reflections/; scripts/ci-install.sh; .espectacular/changes/add-bounded-analysis/.
- 2026-11-12T02:32:43Z [id:6b4f65bedb40a546686cfa837247e0f2ad4c52ad80dbb54e01245f30bc989681] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-12T02:35:00Z
  Repo: arltificial/catherine (private, github, host alias git@at)
  Branch: main
  HEAD: 59a6072 (fix(ci): ensure wasm32 target for the pinned toolchain inside check-wasm)
  Session: omp orchestrator (renew + orchestrate; user directive: push + implement all remaining tickets, tdd-ro5u, commit per ticket)
  Task: catherine — pushed main; closed mrq, 7e8, 9b5 (wasm+web), 1ml; CI lane hardened; hc6 partial
  Previous entry: 84ea7114ac118c4834e730e7f702783aa6133205e61aaaa00ca478a0f236922a (superseded by 6247ad93ae0a6d5ac36284b66d466dba57c1cc4c849a612422e4c17081fecfa5)
  
  Objective: implement ALL remaining tickets per tdd-ro5u with a commit after each; push lane first.
  
  Current phase: between tickets — BLOCKED on OpenRouter credits for subagent orchestration; context past the 40% repo line (recommend wai close + /clear resume, which this entry enables).
  
  Current state:
  - Closed+committed: mrq b1493e4 (leftmost-outermost wording; ro5u at .wai/projects/catherine/reviews/catherine-mrq-ro5u.md), 7e8 dc3f130 (direct-only related() documented+pinning test; wai design artifact), 9b5 c81a31c (wasm boundary + web; headless smoke PASS w/ screenshot; ro5u at openspec/changes/add-wasm-web-view/ro5u-review.md; boundary outcome discriminator added; browser glue web/main.mjs + just web-build; wasm-bindgen CLI 0.2.130 sha-verified in target/tools), 1ml bc6fc18 (symmetric-structure ratchet split into tests/roundtrip_symmetric.rs, explicit 32-case envelope sizes 2..=4 — 256-case draft was 428s; ro5u at reviews/catherine-1ml-ro5u.md; CFI generator deferred LOW). Beads export swept into the amended feature commits (c81a31c, bc6fc18) — 19 closed / 8 open.
  - CI lane: three runner failures diagnosed+fixed — openspec 127 (PATH: ci-install.sh now writes GITHUB_PATH, fe802c4), rustfmt version skew on rolling stable (pinned rust-toolchain.toml 1.98.1, 4b71ef8), wasm32 E0463 (check-wasm now runs rustup target add, 59a6072). The 59a6072 CI run was IN FLIGHT at handoff — verify green before pinning any newer rev into bogart CI.
  - Pretender/clippy gates bite test files too: file_lines 400, fn caps 40/abc/cyclomatic 3/nesting 2 — roundtrip split + builder-table refactor were forced by them; expect the same on 3zs/w6z additions.
  - ro5u artifact convention deviation: change-dir artifacts when the change exists, else .wai/projects/catherine/reviews/<id>-ro5u.md.
  - Briefs READY (untracked, commit with their tickets): 3zs, w6z, 9vv (grounded in ak/matadero/series/ct-catlab SERIES.md — NOTE: proposal says Vending 6/6, series ledger says F2/F3/F5 caught + F6 refuted; ledger is ground truth), hc6.
  - hc6 PARTIAL: subagent died at ~40% on OpenRouter 402 (affordable 831 tokens vs stale 131072 window). Transcript history://BogartHc6Implement. Evidence so far: RED fixture fired (exit 3); live census 2 ambiguous + 67 gap selectors; bogart tree state UNKNOWN/uncommitted — inspect ../bogart git status + .wai research file before resuming. models 4/5 are P7/P8-gated adjuncts, out of scope.
  - BLOCKER: OpenRouter credits exhausted — fresh agent spawns 402. Tickets 3zs, w6z, hc6 (resume), then chain 9vv (STOP CONDITION — killed verdict stops the project; P7 alr, P8 nq5, dvx, ga7 gated on verdict) need subagents (user top-up) or explicit inline order.
  
  Exact next action: user tops up OpenRouter credits (models.yml cap 12288 makes refill last ~10x; fresh agents then spawn correctly) OR directs inline continuation; then resume at 3zs (brief ready), w6z (vampiro 0.5.0 installed; pin + lefthook pre-push + feedback dry-run saved), then 9vv, then hc6 resume; verify the 59a6072 CI run green first.
  
  Ticket ownership: all open tickets unassigned, single-author (sasha); 1ml/9b5/mrq/7e8 claimed+closed by this session.
  
  Validation: just ci green locally at bc6fc18 pre-justfile-fix (only delta = idempotent rustup line); boundary/parity/vitest/tsc green; wasm smoke screenshot archived; export fresh at bc6fc18 content.
  
  Constraints: push authorized for the CI-fix lane + ticket commits (done); bogart work NEVER pushed; user ship-checkpoint replaced by per-ticket commit directive; P6 verdict procedure + stop condition stand; no claims of detection-rate wins; kernel crates genesis-free.
- 2026-11-12T19:58:07Z [id:6247ad93ae0a6d5ac36284b66d466dba57c1cc4c849a612422e4c17081fecfa5] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-12T14:05:00Z
  Repo: arltificial/catherine (private, github, host alias git@at)
  Branch: main
  HEAD: ca67edf (chore(beads): export — close catherine-dvx (last ticket))
  Session: omp orchestrator (long campaign session; waves delegated to task agents, several phases completed inline when credit windows died mid-dispatch — deviations disclosed in per-ticket ro5u artifacts)
  Task: catherine — ALL 12 TICKETS CLOSED: campaign complete (user directive: push + implement all remaining tickets, tdd-ro5u, commit per ticket)
  Previous entry: 6b4f65bedb40a546686cfa837247e0f2ad4c52ad80dbb54e01245f30bc989681
  
  Objective: build catherine per docs/DESIGN.md to completion — done; bd ready reports "No open issues".
  
  Current phase: commit (everything landed and pushed; nothing in flight in the repo).
  
  Current state — verdicts and headline landings:
  - P6 catherine-9vv (STOP CONDITION) executed: FizzBuzz negative control ran FIRST and self-negated (killed, zero unique detections); ledger verdict SURVIVED (scoped); P7/P8 gates flipped per the recorded procedure; crates/catherine-ledger + ledger/ entries #1-#5.
  - P7 catherine-alr: format 1.1 schema maps (hash-neutral, pinned model_hash), pullback reindex with loud lost-data enumeration, Δ∘Δ′ return-identity proof (ignored test un-ignored + identity regression), catherine migrate CLI (exit 8 lost-data, 0 lossless). Multi-arrow target paths are a DOCUMENTED phase-B limit (valued cells recorded as lost, cells None — module doc in migrate/transport.rs).
  - P8 catherine-nq5 (HYPOTHESIZED): format 1.2 rule presentation (KNOWN_MINOR=2, hash-neutral), sim-oracle parity engine, catherine rules CLI; pre-registered validation verdict SURVIVED-SCOPED — 12-fault matrix conventional 7/12 vs lens 12/12, 5 unique ALL structural redundancy (caveat recorded in ledger/rule-lens entry, never rounded); near-floor control killed 0/3; ledger/rule-lens/ is a sibling ledger (P6's sealed rollup forbids appends).
  - P9 catherine-hc6 (bogart pilot, models 1-3) closed: bogart commits fd07973,4864809,943a300,4f400ea; triage 2 real (bogart-4wt filed in bogart bd) / 67 noise accounted.
  - P9 adjunct catherine-ga7 closed: spec-driven grounded-instance harness (bogart commits bc51d1e,a8dafd8); byte-identical parity with the pilot model; seeded must-gate 7/7 with witnesses; ZERO catherine kernel delta.
  - P10 catherine-dvx closed (LAST): schema-diff classification accreted/weakened-with-witness/refactor-equivalent/cosmetic-silent over the shared bounded engine; catherine diff CLI; kernel delta 0.
  - P5 catherine-9b5: wasm boundary + web view (headless smoke verified); P2 wave mrq/7e8/1ml/3zs/w6z all closed (ratchets + gates).
  - CI lane hardened earlier: GITHUB_PATH for npm prefix, pinned rust-toolchain.toml 1.98.1, wasm32 target in check-wasm.
  
  Exact next action: NONE pending on the ticket chain. Candidates when asked: (a) confirm the GitHub Actions run on the latest pushes is green (gh run list); (b) bogart's 6 commits remain UNPUSHED (user-gated policy — never push without explicit instruction); (c) optional future work: multi-arrow path composition in Δ reindex (phase-B limit), no other follow-ups filed.
  
  Essential constraints: push authorized only as already done (main pushed through ca67edf); bogart pushes never; gates green at every ticket commit (fmt/clippy -D warnings/workspace tests/wasm/openspec strict/pretender); pretender caps forced several decompositions (file ≤400, fn ≤40, app cyc ≤10, test-fn cyc ≤3); commit-per-ticket with ro5u artifacts (openspec/changes/<change>/ro5u-review.md or .wai/projects/catherine/reviews/<id>-ro5u.md); bd close+export orchestrator-owned; 11/11 openspec changes strict-valid.
  
  Validation: just ci EXIT=0 at the last feature commit a286401; the final two commits after it are export chores (issues.jsonl only); openspec validate --changes strict 11/11.
  
  Unresolved: GH Actions status for the last pushes (not polled to completion); bogart repo has 2 pre-existing unrelated dirty files (not ours). (supersedes 6b4f65bedb40a546686cfa837247e0f2ad4c52ad80dbb54e01245f30bc989681)
- 2026-11-12T20:48:57Z [id:07c7b0a91a635464b4d6855ad330894eac7f263e2aa3968f0ebb6bc237e8afdb] (#resume-catherine-design) Format: 1
  Timestamp: 2026-11-12T20:48:47Z
  Repo: arltificial/catherine (private, github, host alias at)
  Branch: main
  HEAD: ca67edf (equals origin/main; working tree clean except this handoff append)
  Session: omp renew session (no session id exposed)
  Task: catherine campaign post-completion verification; bogart publish-unit regression test committed
  Previous entry: none (prior campaign record 6247ad93ae0a6d5ac36284b66d466dba57c1cc4c849a612422e4c17081fecfa5 stays valid, not superseded — different session)
  
  Objective: close out the "all 12 tickets closed" state, verify CI, and decide the deferred Δ multi-arrow-path question.
  Current phase: commit/close — catherine has nothing in flight; no code changed this session.
  
  Current state:
  - CI: both pending runs at ca67edf and a286401 concluded success (43293107492 at 20:10Z, 43293048032 at 20:13Z). All earlier pushes green except 43267211010 (migrate CLI), superseded by green efbc99f run. Item (a) from the previous handoff is CLOSED.
  - Δ multi-arrow paths (item c) DEFERRED to another session by user. Analysis done, no code: parse accepts any non-empty list of declared target arrows (parse/schema_map.rs::declared_path, "typing check is phase B" — never added); transport.rs::reindex_arrow treats len>1 as non-transportable and turns every valued cell into LostCell. No fixture or spec sentence covers len>1. Consequence: a malformed single-arrow map (wrong endpoint/start) looks identical to designed loss — class-A seam. Forward composition is ill-defined (needs intermediate rows = invention, or writes into tables owned by other arrows). Options: A pin+validate (typing pass, reason-bearing witness, 4th fixture, spec sentence — recommended, small); B reject len>1 at parse (breaking parse change vs additive policy); C true pullback direction (target instance + map -> source instance, chained lookup, dangling = None — coherent home for composition, medium cost). Do not build forward composition.
  - Bogart (host alias at, ../bogart, main): another live session committed c5e24e6 (unit PATH + EnvironmentFile) and 1459d8b, then pushed both. This session added c93eabe "test(systemd): pin the publish unit's PATH and MCP credential env (a4h.4)" — 12 lines in scripts/test_publish_timers.py, RED proven against b9a0b4c's unit, 7/7 green at HEAD. Local main is ahead 1 of origin/main, NOT pushed (explicit user instruction). 13 dirty files remain from the parallel session (publish_daily.py, clj builder/tests, docs) — untouched.
  
  Exact next action: on request, (i) push c93eabe when the user authorizes (push was explicitly declined this session); (ii) start option A for the multi-arrow limit; (iii) yesterday's publish timer first live run at 2026-11-13 06:00:59 -03 — inspect journalctl --user -u bogart-publish.service and tail data/backfill/publish-state.jsonl.
  
  Constraints: never push bogart without explicit instruction; the bogart working tree belongs to a concurrent session — do not touch its dirty files; no ticket claims this session; bd ready in catherine is empty.
  Validation: catherine just ci not rerun (no code change this session); CI green as above; category: verified externally.
  Unresolved: multi-arrow decision (deferred); bogart b92 stale daily_pickup fixtures; /opt/bogart closure at c310063; openspec archive for add-norma-observation-bridge (bogart-ovo).
- 2026-11-13T14:41:00Z [id:0d4f41db12488655c0a32fdb5a751f8a5193892339cd1f20a0cd3f7ee116ae85] (#resume-catherine-1od) Format: 1
  Timestamp: 2026-11-13T14:39:53Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 0d77836
  Session: 01a1d07d-554a-7105-ad3e-07e6f0475d61 (pi, qwen3.8-max)
  Previous entry: none
  
  Objective: implement openspec change add-domain-probe-validation (beads catherine-1od, P11) — probe ledger directories on the rule-lens reuse pattern + baseline-arm validator rule + two inaugural probes (migration Δ at bogart scale; rule lens on extracted tables). Session that produced it: surveyed tool status, created pinky dossier ct-modelable-domains, authored+ran the 12-provider catitas dispatch, synthesized results, filed the change (Ro5 revision 2) and 5 tickets.
  
  Current phase: proposal complete (committed), implementation not started. Next session starts at RED.
  
  Current state:
  - openspec/changes/add-domain-probe-validation/ — proposal/design/tasks/spec delta, revision 2 post-Ro5, openspec validate --strict green; ah check --changes shows the expected 9 no-toml findings (contract stubs pinned to task 1).
  - Beads open: catherine-1od (P11 probes, this task), catherine-p8d (P1 docs drift: README "implementation not started"/rule-lens "hypothesized"/USING ILLUSTRATED banner — independent, quick), catherine-y4h (P2 seam fixture + dbt-contrast recipe), catherine-0g1 (P2 CQL/FDM head-to-head + Conexus patent note), catherine-kq1 (P2 granularity rule PROPOSED + bogart re-model test). Export fresh (.beads/issues.jsonl, 32 issues).
  - Evidence: pinky dossier ct-modelable-domains (distilled; Tasks wired to the 5 ids); distilled provider answers in ryan/archive/catitas/data/distilled/ct-modelable-domains/ (ryan archive-commit still pending — rides next routine catitas data sync).
  
  Exact next action: catherine-1od task 1 RED — validator tests in crates/catherine-ledger: (a) non-empty unique_detections without committed baseline arm → refused; (b) cited-only baseline + survived + unique claim → refused (inconclusive ceiling); (c) regression pins: probe-dir sequence without self-negating floor control fires existing NegativeControlMissing/ControlDidNotSelfNegate; unregistered probe id → empty expectations, no verdict. Generate the 9 contract stubs: ah scenario new add-domain-probe-validation domain-probes … (per scenario, tests pointing at the RED tests). Then bd update catherine-1od --claim.
  
  Key decisions/constraints:
  - NO new entry kind — kata shape reused in ledger/probes/<name>/ (rule-lens precedent: ledger-check ledger/rule-lens green, own killed near-floor control, executed baseline prototype/rule_lens_baseline.py). Baseline-arm rule applies ledger-wide (kata entries too; existing entries comply).
  - Baseline arms: committed + version-pinned + executed BEFORE the run; cited-only caps at inconclusive; absent caps at parity. Probe 2's manual review must be frozen into scripted assertions first; task 8 has a corpus-check fallback (inconclusive naming the gap, never fabricate a table).
  - Sealed kata rollup: byte-identity test, never recomputed. Probe verdicts gate no phases.
  - Quarantine: openrouter/mistral/nemotron dispatch numbers are fabricated-citation suspect — never cite downstream.
  - Tidy First: task 11 refactor (ah check green) is a separate commit from feature work.
  - Authorization: conservative git profile — commits fine (established this session), no push/deploy without explicit ask. bd changes require export + git add -f + commit invariant.
  
  Ticket ownership: all 5 open tickets unassigned, single-author (sasha/arltificial); no third-party custody.
  
  Validation: openspec validate --strict green; ah check = 9 expected no-toml (proposal stage); pinky audit green for the dossier; cargo test not run this session (no code changes).
  
  Unresolved/blockers: none for task 1. Open deps: probe 2 needs a real policy table in bogart/Normai corpus (unverified — corpus check is task 8's first step). Migration baseline tool choice (Atlas vs Flyway vs frozen script) undecided — decide at task 6, pin the revision.
  
  Refs: openspec/changes/add-domain-probe-validation/{proposal,design,tasks}.md + specs/domain-probes/spec.md (9 scenarios); pinky ledger/topics/ct-modelable-domains.org (synthesis + Tasks); crates/catherine-ledger/src/{validate,entry,catalog,harness}.rs; ledger/rule-lens/ (the pattern to copy); fixtures/migration/bogart_rehearsal.ct.json (task 5 build-up base).
- 2026-11-13T15:36:29Z [id:814d7b42509d9276501caec442b722878ef187606ba3efffb739098dc1dd57bd] (#resume-catherine-1od) Format: 1
  Timestamp: 2026-11-13T15:36:17Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 3e93468
  Session: unknown (this session; harness did not expose an id)
  Previous entry: none (the topic's earlier entry 0d4f41db belongs to another session — not superseded by this snapshot)
  
  Objective: implement openspec change add-domain-probe-validation (beads catherine-1od, P11) — probe ledger directories on the rule-lens reuse pattern, ledger-wide baseline-arm refusals, two inaugural probes (migration Δ at bogart scale; rule lens on extracted tables).
  
  Current phase: green (tasks 1-4 of 11 complete and committed; task 5 onwards is evidence production, not mechanism).
  
  Current state:
  - Task 1 RED: BaselineMode/BaselineArm on Comparison.baseline (serde-skipped when None so every sealed id survived); refusals declared; pins in tests/baseline_arm_red.rs, tests/probe_regression_red.rs; 9 domain-probe contract stubs live in .espectacular/changes/add-domain-probe-validation/domain-probes/ (NOT .espectacular/domain-probes/ — the overlay reads change-scoped, deployed copies surface as orphan-toml until apply).
  - Task 2 GREEN: baseline_violations over effective entries (UniqueWithoutBaseline / CitedBaselineCannotSurvive); supersession is now a usable correction path after a rollup (effective-entry filtering for claim rules, integrity rules on all entries, effective rollup must be last with set-equality coverage); ledger-record pinned to refuse the same shapes. Shipped ledger/rule-lens/0002 corrected append-only by 0004 (executed prototype/rule_lens_baseline.py arm, sha256:36f6b208…, 7/12 faults: 01,02,03,06,08,11,12) + 0005 (replacement rollup). 0001-0003 and the P6 kata ledger byte-identical.
  - Tasks 3-4: catalog::PROBE_IDS = [probe-migration (lost-data, non-invertibility, floor-visible-loss), probe-rule-lens-extracted (gap, overlap, shadow, derivable, floor-near-trivial)], probe_catalog_hash content address, KataRecord.catalog_hash + CatalogHashMismatch (retrofit-proof), harness::derive_probe_verdict ladder, probe_ledger_roundtrip tests incl. kata-ledger byte-identity.
  - Commits this session: 7f921eb (ploeh grounding docs), ec7aae5 (bead claim), 408c1ac (tasks 1-2), 3e93468 (tasks 3-4). Tree clean except the turu-managed .whisper/branches/main/notes.md mirror (left uncommitted; session-close artifact).
  
  Exact next action: task 5 — bogart-scale migration fixtures under fixtures/probes/migration/ built up from fixtures/migration/bogart_rehearsal.ct.json, with the pre-registered lost-data / non-invertibility expectations committed before any run and the floor-control fixture (a loss fully visible to row-count/checksum). Parallel-ready: task 6 baseline harness (must be committed + version-pinned before the run). Decide the baseline tool at task 6: Atlas/Flyway at a pinned revision vs a frozen script encoding exactly their documented checks — user has not chosen yet, so do not fake the pin.
  
  Constraints: executed checks are the product (E3); no detection-rate promises (E4, P12 precedent); proptest/cargo tests over snapshots; kernel crates genesis-free; conservative git profile — commits fine, no push/deploy without explicit ask; bd changes need export + git add -f + commit; probe 2 needs a real extracted table in the bogart/Normai corpus (corpus check is task 8 step 1; record inconclusive, never fabricate).
  
  Ticket ownership: catherine-1od claimed by this session (assignee set). Open unclaimed: catherine-p8d, catherine-y4h, catherine-0g1, catherine-kq1, catherine-lgl (+10 children; lgl.5 and lgl.7 blocked on 1od). add-bogart-pilot sits at 8/10 with tasks 5-6 (models 4-5) now unblocked and unclaimed.
  
  Validation: cargo test -p catherine-ledger 48 passed / 0 failed; cargo check --workspace clean; ledger-check ledger/ = ok 5 entries (4 kata) survived; ledger-check ledger/rule-lens/ = ok 5 entries (2 kata) survived; original 8 ledger files byte-identical to pre-change digests; ah check 0 issues; ah check --changes add-domain-probe-validation = 2 no-tests-declared (survived/killed probe witness scenarios, tasks 7/9); cargo fmt clean; all pre-commit hooks green.
  
  Unresolved/blockers: baseline-tool decision for task 6 (user input); probe 2 corpus availability unverified; tasks.md task 11 path wording (.espectacular/domain-probes/) needs correcting at apply.
  
  Refs: openspec/changes/add-domain-probe-validation/{proposal,design,tasks}.md + specs/domain-probes/spec.md; crates/catherine-ledger/src/{catalog,entry,validate,harness}.rs; tests/{baseline_arm_red,ledger_correction,ledger_record_refuses,probe_catalog_green,probe_harness_green,probe_ledger_roundtrip}.rs; ledger/rule-lens/0004-0005; fixtures/rule_lens/README.md matrix; fixtures/migration/bogart_rehearsal.ct.json.
- 2026-11-13T16:38:24Z [id:78d8557717289142b32933f80f8de1337b0ec2a7688cc392e48ac7031b72db71] (#catherine-8v9) Format: 1
  Timestamp: 2026-11-13T16:38:24Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 8ab94df
  Session: omp renew session (harness exposed no id)
  Previous entry: none (superseded by 32c02397baf003b3166150d0fcf48beeec7641b765cf09a510635a512f405f92)
  
  Objective: close the catherine backlog — P11 domain probes, docs drift, seam recipe, positioning, granularity rule, canonicalization ticket, and the 10-ticket OOP pattern-kata suite.
  
  Current phase: commit (everything landed; only the filed follow-up catherine-8v9 remains open and it is bogart-side).
  
  Current state:
  - catherine-1od closed: probe 1 (migration Δ: control killed; 10 named cell losses + 1 round-trip witness unique) and probe 2 (extracted CCT 288/97 table: control killed; 4 unique; derivable recorded as unreached) recorded in ledger/probes/{migration,rule-lens-extracted}/ with committed executed baseline arms (prototype/{migration,rule_lens_extracted}_baseline.py); specs deployed to openspec/specs/domain-probes/spec.md; change archived 2026-11-13-add-domain-probe-validation; contracts deployed to .espectacular/domain-probes/.
  - Docs: p8d status drift fixed; y4h seam fixture (crates/catherine-cli/tests/fixtures/seam_multi_path_agreement.ct.json) + USING Recipe D; 0g1 PROPOSAL §7 CQL head-to-head + grounded Conexus patent note (US 11,256,672 B1, US 11,720,535 B2, app US 2024/0119034 A1; counsel required, no self-clearance); kq1 PROPOSED granularity rule + pilot calibration (fail ratios 2/3, 0/3, 4/5).
  - Kata suite complete: 7 executable ledgers (composite-null-object 4 entries, builder-monoid 5, visitor-sum-types 2, state-discovery 5, memento-roundtrip 4, wrapper-laws 2, cor-strategy-rules 2), weak-lens-set 6 (all killed/parity, Flyweight unexpressible), negative-control-bundle 4 (honesty gate, all killed, unique 0). Disposition table docs/ref/pattern-kata-dispositions.md (24 patterns). Epic catherine-lgl closed.
  - catherine-6fs closed after retest: no test hang (roundtrip suite 23s); genuine limitation is model_hash cost on large symmetric fixtures, which is why probe fixtures are small.
  - Open: catherine-8v9 (bogart-side granularity re-model + signal-density measurement; protocol in .wai/projects/catherine/research/2026-11-13-granularity-rule-calibration.md).
  
  Exact next action: on request, execute catherine-8v9: re-model the bogart FTS-anchor subdomain at the PROPOSED granularity rule, extract a real row slice, run `catherine check`, classify real vs noise, compare with the pilot's 2/69, record in the pinky ct-modelable-domains dossier; if it lands, add a probe ledger entry with a committed baseline arm. Do not claim the rule validated without that measurement.
  
  Key decisions/constraints:
  - Parity is the expected outcome; several katas are explicit kills/nulls, and the disposition table forbids a detection-rate reading. Wrapper uniques are coverage-only (0 under a complete cross-product sample). Disposed/inexpressible rows are gap statements, not theorems.
  - Probe entries carry executed baseline arms with sha256 pins; entries are content-addressed; probe verdicts gate no phases; the kata ledger rollup stays sealed/byte-identical.
  - Conservative git profile: commits fine; NOTHING pushed this session (local main ahead of origin/main).
  - bd invariant: after any bd change run `bd export -o .beads/issues.jsonl && git add -f .beads/issues.jsonl`.
  
  Ticket ownership: all processed tickets single-author (sasha/arltificial); no third-party custody. catherine-8v9 open and unclaimed.
  
  Validation: cargo fmt clean; workspace clippy -D warnings clean; cargo test --workspace green; cargo test -p catherine-core --test roundtrip 23s; ledger-check ok on every kata/probe dir; ah check 0 issues; openspec validate --specs 10/10 and --changes 1/1; per-slice baseline --assert green; pretender pre-commit gate green (repo-wide pretender still reports 7 pre-existing violations outside this session's files).
  
  Unresolved/blockers: none for the closed tickets. catherine-8v9 needs bogart-side execution (external repo, dossier in pinky). Push not authorized.
  
  Refs: ledger/probes/*; ledger/katas/*; fixtures/probes/*; fixtures/katas/*; docs/ref/pattern-kata-dispositions.md; .wai/projects/catherine/reviews/catherine-{1od,p8d,y4h,0g1,kq1,lgl.1,lgl.2,lgl.3,lgl.4,lgl.5,lgl.6,lgl.7,lgl.8,lgl.9,lgl.10}-ro5u.md; openspec/changes/archive/2026-11-13-add-domain-probe-validation/; openspec/specs/domain-probes/spec.md.
- 2026-11-13T16:54:03Z [id:32c02397baf003b3166150d0fcf48beeec7641b765cf09a510635a512f405f92] (#catherine-8v9) Format: 1
  Timestamp: 2026-11-13T16:53:52Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 8ab94df
  Session: omp renew session (harness exposed no id)
  Previous entry: 78d8557717289142b32933f80f8de1337b0ec2a7688cc392e48ac7031b72db71
  
  Objective: close the catherine backlog — P11 domain probes, docs drift, seam recipe, positioning, granularity rule, canonicalization ticket, and the 10-ticket OOP pattern-kata suite.
  
  Current phase: commit — all processed tickets closed and committed; only the filed follow-up catherine-8v9 remains open (bogart-side).
  
  Current state:
  - 14 tickets closed: 1od (P11 probes 1+2 recorded; specs deployed/archived), p8d (docs status), y4h (seam fixture + USING Recipe D), 0g1 (PROPOSAL §7 CQL/patent note), kq1 (PROPOSED granularity rule + calibration), 6fs (retest: no hang; model_hash cost on large symmetric fixtures only), lgl epic + all 10 children.
  - Ledgers: probes/migration 3 entries (unique 11), probes/rule-lens-extracted 2 (unique 4; derivable unreached), katas/* 34 entries across 9 dirs (unique 58; 11 killed/null, 23 survived-scoped). Every dir control-first, validated, baseline arms executed and sha-pinned.
  - Docs: README/USING match shipped state; captured check/migrate/rules runs; docs/ref/pattern-kata-dispositions.md covers all 24 patterns; honesty gate (negative-control-bundle) all killed, 0 unique.
  - Repo: clean except the turu notes mirror; 33 commits ahead of origin/main, NOTHING pushed (conservative profile).
  - Open ticket: catherine-8v9 — bogart-side granularity re-model + signal-density measurement; protocol in .wai/projects/catherine/research/2026-11-13-granularity-rule-calibration.md; falsifier vs the pilot 2/69.
  
  Exact next action: on request, execute catherine-8v9 (bogart repo): re-model the FTS-anchor subdomain at the PROPOSED granularity rule, extract a real row slice, run `catherine check`, classify real vs noise, compare with 2/69, record in the pinky ct-modelable-domains dossier; if the ratio improves, add a probe ledger entry with a committed baseline arm. Do not claim the rule validated without that measurement.
  
  Key decisions/constraints: parity is expected; several katas are explicit kills/nulls and wrapper uniques are coverage-only (0 under a complete cross-product sample); disposed/inexpressible rows are gap statements, not theorems. Probe verdicts gate no phases; kata rollup stays sealed/byte-identical. Conservative git profile: commits fine, no push/deploy without explicit ask. bd invariant: after any bd change, `bd export -o .beads/issues.jsonl && git add -f .beads/issues.jsonl`.
  
  Ticket ownership: all processed tickets single-author (sasha/arltificial); catherine-8v9 open, unclaimed.
  
  Validation: cargo fmt clean; workspace clippy -D warnings clean; cargo test --workspace green (274s); cargo test -p catherine-core --test roundtrip 23s; ledger-check ok on every kata/probe dir; ah check 0 issues; ah check --run-tests 90 passed; openspec validate --specs 10/10, --changes 1/1; per-slice baseline --assert green; pretender pre-commit gate green (repo-wide still reports 7 pre-existing violations outside this session's files).
  
  Unresolved/blockers: none for closed tickets. catherine-8v9 needs the external bogart repo + pinky dossier. Push not authorized.
  
  Refs: ledger/probes/*; ledger/katas/*; fixtures/probes/*; fixtures/katas/*; docs/ref/pattern-kata-dispositions.md; .wai/projects/catherine/reviews/catherine-{1od,p8d,y4h,0g1,kq1,lgl.1,lgl.2,lgl.3,lgl.4,lgl.5,lgl.6,lgl.7,lgl.8,lgl.9,lgl.10}-ro5u.md; openspec/specs/domain-probes/spec.md; openspec/changes/archive/2026-11-13-add-domain-probe-validation/. (supersedes 78d8557717289142b32933f80f8de1337b0ec2a7688cc392e48ac7031b72db71)
- 2026-11-13T17:03:43Z [id:2f5630abe17afe831054064cc8ca01ed8294a8fdfc4a983afa539a3adf3c34f1] (#catherine-8v9) Format: 1
  Timestamp: 2026-11-13T17:03:43Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: dbeb136
  Session: omp renew session (harness exposed no id)
  Previous entry: none (state continuation, not a supersession: 32c02397… is the prior session's pre-8v9 snapshot; its outstanding next action is now completed)
  
  Objective: execute catherine-8v9 — second-domain test of the PROPOSED arrow-granularity rule — and record the outcome.
  
  Current phase: commit (done; all tickets processed, bd ready empty).
  
  Current state:
  - catherine-8v9 CLOSED. Commit c25e507 = pre-registration (predictions P1-P4 + generator, committed before any run); commit 83b0c70 = result + docs; dbeb136 = changelog.
  - Model: 2 arrows with named storing columns (composite pages->documents via document_id+version; documents->edition_date), 0 equations. catherine check: 0 findings on the 289-page slice and on the full 74,257-page / 924-doc / 210-edition corpus (exit 0). Ratio undefined vs pilot 2/69.
  - Verdict: rule NOT validated; label stays PROPOSED and is RE-SCOPED to structural claims. It excludes the pilot's 67 planner-intent noise findings and its 2 real bogart-4wt ambiguity defects alike — no table stores a selector->capture column. Live census (backfill corpus, live/concurrent ingest): 1,441 selectors, 3 with >=2 matches (nacional/primera 2026-11-03, pba/JUDICIAL 2026-11-03 = pilot's two; nacional/segunda 2026-11-13 new, same class, cause beyond match-count unverified), 67 zero-match. No probe-ledger entry: falsifier negative-result branch (a probe needs an executed baseline arm + landed outcome).
  - Docs updated: docs/USING.md Recipe A note; docs/ref/ct_methodology_playbook.md §2; CHANGELOG Unreleased/Changed; pinky dossier ct-modelable-domains open question marked RESOLVED (audit ok).
  
  Exact next action: none in catherine — bd ready is empty. If the user wants more: (a) file/attach the third ambiguous selector (nacional/segunda 2026-11-13) to bogart-4wt from the bogart session (do NOT touch bogart's dirty tree from catherine sessions); (b) regenerate the playbook PDF (~/Downloads/ct_methodology_playbook.pdf) since §2 changed; (c) push is still unauthorized — local main is ahead of origin/main.
  
  Key decisions/constraints: conservative git profile — commits made, NOTHING pushed; bd invariant honored (export + git add -f .beads/issues.jsonl). Rule's pre-build one-third test passes on a blind model (0/2 here) — treat it as a smell test only. Probes/kata ledger untouched; kata rollup stays sealed/byte-identical. Generator obeys the pretender gate (split functions); regenerated models byte-identical (slice f8dd74cc…, full e54a56e5…).
  
  Validation: cargo build -p catherine-cli reused; catherine check exit 0 both runs; ah check 0 issues; pre-commit gates green (pretender, ah, bd); pinky audit ok; bd export fresh (45 issues).
  
  Unresolved/blockers: none for catherine. The rule's intent-level defects remain only expressible in P8 rule-lens models — that is now documented, not an open ticket.
  
  Refs: fixtures/probes/granularity-bogart/{README.md,fts-anchor-slice.ct.json}; prototype/bogart_storage_granularity.py; .wai/projects/catherine/research/2026-11-13-granularity-rule-calibration.md; docs/ref/ct_methodology_playbook.md §2; pinky ledger/topics/ct-modelable-domains.org.
- 2026-11-13T17:45:27Z [id:2264a841be1496e0d669620f0812613a2b690199c32015051cb1f1d8ab650380] (#post-8v9-gaps) Format: 1
  Timestamp: 2026-11-13T17:45:16Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: fbdb997
  Session: omp session (harness exposed no id)
  Previous entry: none (new topic; the catherine-8v9 thread is closed and was stashed separately)
  
  Objective: decide whether catherine helps in modeling, ground the resulting four-item proposal in current specs/code, and apply a Rule-of-5 review to that grounding. Successor to the closed catherine-8v9 re-model.
  
  Current phase: commit — research and review fixes committed; nothing in flight.
  
  Current state:
  - 8v9 result stands: storage-granularity re-model → 0 findings on 289-page slice and full 74,257-page corpus; rule NOT validated, re-scoped to structural claims; ticket closed; bd ready empty.
  - New note .wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md (commits 75a2847, fbdb997) grounds four additions: A composite arrows, B key/FD constraint, C scoped quantification, D extraction bridge. It contains an evidence ledger, traced blast-radius table, format-1.3 checklist, and per-step falsifiers.
  - Landed design decisions (all INFERENCE, none implemented): keys over arrows only — column FDs need the value-object convention, else use the rules bridge; partial_schema_map means missing entry only, composite-map rejection needs its own code; key collisions overload EquationWitness with an explicit field mapping (tuple values not carried) or need a new payload variant; composite dangling = one finding per null component; composite arrows are excluded from equations (cost stated); accepted keys judge bounded-search satisfiability; extraction mapping is versioned with sources[{id,path,sha256}].
  - Sequencing fixed: 0 fix two spec drifts (model-document KNOWN_MINOR "currently 0" vs code 2; cli Purpose six vs binary ten subcommands), 1 `rules --project` bridge (acceptance: 3 violations on the 8v9 capture slice), 2 declarative keys + schema_diff enforced-set delta, 3 composite arrows (same 1.3 bump), 4 extraction, 5 scoping unscheduled.
  - Git: 38 commits ahead of origin/main, nothing pushed. Tree clean except the turu notes mirror.
  
  Exact next action: on user request, (a) file the two spec-drift beads, (b) scaffold the openspec change for step 1 `rules --project` and implement to the 3-violation acceptance test. Do not start without a request; the user has not yet chosen.
  
  Key decisions/constraints: conservative git profile — commits fine, NO push/deploy without explicit ask; bd changes need export + git add -f .beads/issues.jsonl; no new finding kinds (rule-lens "No new finding variants are introduced" precedent); additive format minors only; existing model hashes and sealed ledger model_hashes must not change when new sections are absent; catherine-8v9 is closed, do not reopen.
  
  Ticket ownership: no open tickets in catherine bd; nothing claimed this session.
  
  Validation: both research commits passed pre-commit gates (pretender, ah check, bd, spk-lint); ah check = 0 issues; pinky audit ok after the dossier edit; no code changed, so cargo tests not rerun.
  
  Unresolved/blockers: none. Two review HIGH findings sit below the TypeSafe gate (CORR-002 witness fit, CORR-003 keys scope) and are now stated in the note rather than resolved in design — a human call if step 2 proceeds.
  
  Refs: .wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md; commits 75a2847, fbdb997; fixtures/probes/granularity-bogart/; prototype/bogart_storage_granularity.py.
- 2026-11-13T19:27:00Z [id:cb903770f4af22ca788ff8750f4a539ca8184a822867604f495cb40d95d0722d] (#rules-projection) Format: 1
  Timestamp: 2026-11-13T21:05:00Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 10438bf
  Session: omp session (harness exposed no id)
  Previous entry: none (new topic; supersedes neither post-8v9-gaps nor catherine-8v9, both read and honored)
  
  Objective: execute step 0 + step 1 of the post-8v9 capability-gap sequencing (`.wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md` §7): fix the two drifting spec texts, then land the `rules --project` bridge to the pre-registered acceptance.
  
  Current phase: commit — all work landed; nothing in flight.
  
  Current state:
  - Step 0 (drifts, beads catherine-rgh/0hn CLOSED, commit fa614c7): model-document "Current major parses" now states KNOWN_MINOR = 2 (1.1 schema_map, 1.2 rule_presentation); cli Purpose lists all ten subcommands.
  - Step 1 (catherine-x94 CLOSED): `catherine rules <model> --project <Object> --columns <arrow,...> [--action <arrow>] [--values ...]` synthesizes a RulePresentation from instance rows (one rule/row in dense order, string-equality predicates over target row names, identity/--action action, fixed DeclaredDisjoint) and feeds the shipped rule lens. Domain = distinct observed column tuples ∪ --values product, deduped and canonically sorted. No format change, KNOWN_MINOR stays 2, no new finding kind, envelope shape unchanged.
  - Kernel seams (hash-neutral): `parse::parse_with_rows -> (Model, RowNames)` with RowNames a side table in `parse/rows.rs` (Instance equality, canonical bytes, hash stay name-blind); `RuleDomain::from_tuples` exact-input mode in `rule_lens/domain.rs` (+ `RuleDomainError::WrongWidth`); `Literal` gained Ord for canonical sort.
  - Commits: ba673fa fixture, 58eb30c change scaffold, 15aba34 core seams, 382bc2e + cb3f039 gate refactors (file/function caps), 148a3bf CLI bridge, 10438bf bead export. 46 commits ahead of origin/main, nothing pushed.
  - Fixture: `fixtures/probes/capture-selector/{README.md,capture-slice.ct.json}` + generator `prototype/bogart_capture_slice.py` (read-only corpus; reproducible sha256 85178ffe…). Census: slice ≥2026-11-01 = 38 captures / 35 pairs / 65 selectors / 3 ambiguous / 30 zero-match; corpus 1,377 / 1,374 / 1,441 / 3 / 67.
  - Open change `openspec/changes/add-rules-projection/` (cli delta, 4 scenarios, change-scoped contracts in `.espectacular/changes/add-rules-projection/cli/`). Not archived — archiving is a deliberate later step.
  
  Exact next action: on user request, start step 2 of the sequencing — declarative `keys` + `schema_diff` enforced-set delta (format 1.3, KNOWN_MINOR 2→3; witness-fit question CORR-002 and keys scope CORR-003 are the documented human calls before design). Do not start without a request; step 3 (composite arrows), 4 (extraction), 5 (scoping) stay unscheduled.
  
  Key decisions/constraints:
  - Conservative git profile: commits fine, NO push/deploy without explicit ask; bd changes need export + `git add -f .beads/issues.jsonl`.
  - Projection claim is outside the hash and the ledger — evidence about a projection, never sealed model evidence; the 8v9 granularity rule is neither validated nor invalidated by it.
  - No new finding kinds (rule-lens Decision 1 precedent); additive format minors only; sealed ledger model_hashes must not change when new sections are absent.
  - `--values` keeps its shipped arity (one spec per flag); a multi-entry `--action` is refused rather than silently truncated. The JSON witness stays index-based (no rule table in the envelope) — documented limitation, map through dense row order.
  
  Ticket ownership: no open tickets in catherine bd; catherine-x94, rgh, 0hn closed this session; single-author (sasha/arltificial), nothing claimed.
  
  Validation (executed this session): acceptance A1/A2 `rules … --project backfill_captures --columns capture_source,capture_edition_date --json` → exit 3, exactly 3 violation/executed, each witness pair decoded to the two captures behind one ambiguous selector (nacional/primera + pba/JUDICIAL 2026-11-03, nacional/segunda 2026-11-13), zero underconstraint; A3 adding `--values capture_source=src_nacional_primera --values capture_edition_date=ed_2026_11_07` → 4 findings, the gap class names that exact input; exit 3. `cargo test --workspace` green; `just ci` green (fmt, clippy -D warnings, tests, wasm check, TMPDIR=$PWD/target/tmp); `openspec validate --strict --all` 12/12; `ah check --changes add-rules-projection --run-tests` 98 passed; pretender `--mode gate` 0 violations in every changed/new file; fixture regeneration byte-identical.
  
  Unresolved / notes:
  - pretender's pre-commit gate can under-report: violations first seen in a failed commit run are treated as pre-existing afterwards. Run `pretender check --diff-only --mode gate; echo exit=$?` (or scope it to the crate) before trusting the hook; it is how the 523/696-line violations were caught after a commit had already passed.
  - `ah scenario new <change> <spec> --requirement <req> "<heading>"` refuses with "already exists" for the second and later scenarios of a change; the remaining three contract stubs were written by hand in the deployed format (ah check accepts them).
  - Legacy gate hotspots (not ours, tolerated by the baseline): diff.rs::run_diff 44 lines, render/diff.rs::diff_envelope 43, standard_conformance nesting 3.
  
  Refs: openspec/changes/add-rules-projection/{proposal,design,tasks}.md (7 decisions + close note); fixtures/probes/capture-selector/README.md (A1–A4); crates/catherine-cli/src/projection/{mod,spec,build,tests}.rs; crates/catherine-core/src/parse/rows.rs; crates/catherine-core/src/rule_lens/domain.rs; crates/catherine-cli/tests/{cli_rules_projection,cli_rules_projection_acceptance,cli_rules_projection_errors,negative_matrix_projection}.rs; research note §3/§7.
- 2026-11-13T20:34:10Z [id:0c237fadcf0063f9d912693c0f88c89c263b2236d521c9e924e7acc8478cdd81] (#gate-strictness) Format: 1
  Timestamp: 2026-11-13T22:10:00Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 9b7a6f2
  Session: omp session (harness exposed no id)
  Previous entry: none (append-only follow-up to cb903770 on topic rules-projection; nothing superseded) (superseded by 9756160d841055443b2ed9166015dac3e0100dedadc9b5ec0ab9b5ad0c7df96e)
  
  Objective: make the complexity gate strict after catherine-x94 showed a 523-line module and a 696-line test passing a commit. Bead catherine-lsf (CLOSED).
  
  Current phase: commit — landed; nothing in flight.
  
  Current state:
  - Root cause: the ddl-managed pre-commit gate ran `pretender check --diff-only`, which diffs the UNSTAGED tree. At pre-commit everything is staged, so it scanned nothing and exited 0. Verified empirically: a staged new file and a staged modification of a tracked file were both invisible to `--diff-only` (exit 0) and caught by `--staged`/full scan (exit 1). `--diff-only` is therefore useless in a pre-commit hook.
  - Fix (commit dde4851): `lefthook.yml` gains `pretender-strict` OUTSIDE the ddl-managed block (`run: pretender check --mode gate`, whole tree, 0.2s); `just ci` runs the same command; `.github/workflows/ci.yml` runs it as its own step so `--no-verify` commits still fail in CI. `crates/catherine-cli/tests/gate_conformance.rs` pins both facts: the strict run line lives after `# ddl:managed:end (pre-commit)` with no `--diff-only`, and `pretender-baseline.json` does not exist (gate mode is fail-closed without one; adopting a baseline would silence every recorded violation). `.pretender/` is gitignored; no baseline was ever created.
  - Legacy debt cleared (commit 11ea85b) so the full tree is green: analysis/search.rs::enumerate 53 lines split; cli/diff.rs::run_diff 44 and render/diff.rs::{diff_envelope 43, render_diff 46} split; standard_conformance beads test (cyc 6/cog 9/nesting 3) and wasm purity test (cyc 4/cog 6/nesting 3) refactored; prototype/rule_lens_baseline.py::row 5 params -> ROW_FIELDS + *values (all call sites positional). Behavior-preserving; prototype smoke run still 45 assertions / 0 mismatches.
  - Proof: staged 63-line probe -> `lefthook run pre-commit --command pretender-strict` exit 1 naming the violation; clean tree -> exit 0 with "✓ pretender-strict"; `pretender check --mode gate` at repo root exit 0 / 0 violations; `just ci` green (includes the new step); guard test green.
  
  Exact next action: none for this thread. When the user asks, resume step 2 of the post-8v9 sequencing (declarative keys + schema_diff enforced-set delta; CORR-002/CORR-003 human calls first).
  
  Key decisions/constraints:
  - Strict = full-tree `--mode gate`, never `--diff-only`/`--staged` for the hook (a partial scan is exactly the hole). Keep the ddl-managed blind gate untouched (it is redundant but harmless; editing managed blocks invites regeneration churn) — the guard test makes its blindness irrelevant.
  - No baseline, ever; the guard test is the enforcement of that policy.
  - Conservative git profile: commits fine, no push.
  
  Unresolved: the ddl-managed `ddl-pretender-gate` still carries `--diff-only` and will remain a no-op at pre-commit until ddl changes it upstream; do not rely on it. If `ddl init --gates` is re-run, `pretender-strict` survives by construction but `gate_conformance` should be re-run to confirm.
  
  Refs: lefthook.yml (pretender-strict), justfile (ci), .github/workflows/ci.yml, crates/catherine-cli/tests/gate_conformance.rs, versions.ddl.toml (pretender pin 0.10.0; `--adopt`/baseline semantics), commits dde4851, 11ea85b.
- 2026-11-13T21:15:59Z [id:9756160d841055443b2ed9166015dac3e0100dedadc9b5ec0ab9b5ad0c7df96e] (#gate-strictness) Format: 1
  Timestamp: 2026-11-13T21:15:43Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 9b7a6f2
  Session: omp session (harness exposed no id)
  Previous entry: 0c237fadcf0063f9d912693c0f88c89c263b2236d521c9e924e7acc8478cdd81 (same session, same task; this snapshot replaces it with the landed + verified state and the upstream ticket)
  
  Objective: make the repo's complexity gate strict after catherine-x94 showed a 523-line module and a 696-line test passing a commit; then file the upstream defect. Bead catherine-lsf CLOSED.
  
  Current phase: commit — everything landed and verified; nothing in flight.
  
  Current state:
  - Root cause (proven): the ddl-managed pre-commit gate ran `pretender check --diff-only`, which diffs the UNSTAGED tree. At pre-commit everything is staged, so it scanned nothing and exited 0. Measured: staged new 63-line function and staged 60-line addition to a tracked file → `--diff-only` exit 0 / 0 hits; `--staged` exit 1 / named; `lefthook run pre-commit --command ddl-pretender-gate` exit 0 with the violation staged.
  - Fix (dde4851): `pretender-strict` in lefthook.yml OUTSIDE the ddl-managed block (`pretender check --mode gate`, whole tree, 0.2 s); same command in `just ci`; own step in `.github/workflows/ci.yml` (catches `--no-verify`); `crates/catherine-cli/tests/gate_conformance.rs` pins the strict run line's location (no `--diff-only` run line there) and asserts `pretender-baseline.json` does not exist (gate mode is fail-closed without one). `.pretender/` is gitignored; no baseline ever created.
  - Debt cleared (11ea85b), behavior-preserving: analysis/search.rs::enumerate 53 lines split (trace-equivalent budget-first while form); cli/diff.rs::run_diff 44, render/diff.rs::{diff_envelope 43, render_diff 46}; standard_conformance beads test and wasm purity test nesting/cyclomatic; prototype/rule_lens_baseline.py::row 5 params -> ROW_FIELDS + *values (all call sites positional).
  - Bead bookkeeping (9b7a6f2): catherine-lsf tracked and closed.
  - Upstream: DDL-sl9 filed in the dulce-de-leche workspace (`/var/home/sasha/para/areas/dev/gh/charly/dulce-de-leche`), P1 bug, labels gates/pretender, dependency `discovered-from DDL-3fd`. Content: reproduction table, DDL-3fd-intent analysis, fix = `run: pretender check --staged --mode gate` in `PRE_COMMIT_BLOCK` (`src/gates/lefthook.rs:28`; tests `src/gates/tests.rs:95`, `tests/gates_command.rs:78-81`), regression assertion forbidding `--diff-only` in the pre-commit block, migration note (managed blocks refresh only on `ddl init --gates`; doctor could flag stale blocks), and a lower-priority pretender help-text note. NOT exported or pushed (their repo, conservative profile) — needs `bd export` / `bd dolt push` there when authorized.
  
  Exact next action: none for this task. On request, resume step 2 of the post-8v9 sequencing (declarative keys + schema_diff enforced-set delta; CORR-002/CORR-003 human calls first).
  
  Key decisions/constraints:
  - Strict means full-tree `--mode gate`; never `--diff-only` or `--staged` for the hook (a partial scan is the hole). Leave the ddl-managed blind gate untouched (editing managed blocks invites regeneration); its blindness is neutralized by the guard test.
  - No pretender baseline, ever; the guard test enforces the absence.
  - Conservative git: commits fine, no push. 49 commits ahead of origin/main.
  
  Validation (executed): staged probe -> hook exit 1 naming probe(); clean tree -> hook exit 0 with "✓ pretender-strict"; `pretender check --mode gate` at repo root exit 0 / 0 violations; `just ci` exit 0 (now includes the gate); gate_conformance green; workspace tests green; prototype smoke run 45 assertions / 0 mismatches.
  
  Unresolved: managed `ddl-pretender-gate` stays `--diff-only` and a no-op until DDL-sl9 ships and repos regenerate; DDL-sl9 is local-only; the pretender help-text ticket offered but not filed.
  
  Refs: lefthook.yml (pretender-strict), justfile (ci), .github/workflows/ci.yml, crates/catherine-cli/tests/gate_conformance.rs, versions.ddl.toml (pretender 0.10.0 pin, `--adopt`/baseline semantics), commits dde4851, 11ea85b, 9b7a6f2; DDL-sl9 body in the dulce-de-leche beads DB. (supersedes 0c237fadcf0063f9d912693c0f88c89c263b2236d521c9e924e7acc8478cdd81)
- 2026-11-13T23:00:32Z [id:933104c300599e2d881ddd0dc8667d3bfd5500fe2ee2d665ac43492c3b7c5a59] (#declarative-keys) Format: 1
  Timestamp: 2026-11-13T23:00:32Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 16a25d6
  Session: omp session (harness exposed no id)
  Previous entry: none (new topic; post-8v9-gaps step 2 executed) (superseded by 3bc45604610eda14fe7c564deddc59c81dbe141c737a8deabf988cc745b59509)
  
  Objective: step 2 of the post-granularity sequencing — declarative keys (injectivity constraints) plus the schema_diff enforced-set delta. Beads catherine-2s6 (feature) and catherine-obc (refactor) CLOSED; bd ready empty.
  
  Current phase: commit — everything landed and verified; nothing in flight.
  
  Current state:
  - Format 1.3: schema.keys [{object, arrows[], status accepted|proposed}], KNOWN_MINOR 3, new stable code invalid_key (undeclared object/arrow, arrow not sourced at the key object, empty or duplicate arrows; invalid_status reused). Absent or empty keys omitted from canonical bytes -> every pre-1.3 document keeps its exact hash (pinned sha256:fb033e3c fixture constants stayed green); schema_map target_schema validates keys by the same passes.
  - Findings/checks: FindingKind::Violation(ViolationWitness), untagged union Equation(EquationWitness)|Key(KeyWitness{key, rows:[lo,hi]}); equation JSON byte-identical. key_findings groups rows by resolved arrow tuple, skips unresolved cells, one violation per unordered pair (ascending), accepted=executed/proposed=hypothesized, groups sorted defensively.
  - Analysis: EnforcedEquation -> EnforcedConstraint::{Equation{lhs,rhs}, Key{object,arrows}}; enforced set = accepted equations + accepted keys; is_clean_typed needed no change; added key -> accreted, dropped key -> weakened with the key witness via the existing bounded scan.
  - CLI: key witness block (violation: key-<i> <Object>(<arrows>) + rows), diff added key shape (untagged ConstraintView; equation shape unchanged), head text says constraint. Web: findings.ts union branch, key fixture, generated types regenerated.
  - CORR-003 closed: keys are arrow-only; column FDs need the value-object convention or rules --project until attribute columns (format 1.4). CORR-002 closed: witness union, not an overloaded EquationWitness (check output mixes both, so a reused equation index would be ambiguous).
  - Commits: 915f06c feature, 1e92e49 negative-matrix row (ill-formed key -> invalid_key, exit 1), 2d10292 web+types, 4ee1b09 docs, 06ad208 openspec change + contracts, 16a25d6 bead export. 55 commits ahead of origin/main; nothing pushed.
  
  Exact next action: none for this task. Unscheduled remains step 3 (composite arrows, same 1.3 bump), step 4 (extraction bridge), step 5 (scoped quantification). Optional housekeeping: archive the open openspec changes (add-rules-projection, add-bogart-pilot 8/10, add-declarative-keys) — a deliberate later step, not started.
  
  Key decisions/constraints: conservative git profile — commits fine, NO push/deploy; bd changes need bd export + git add -f .beads/issues.jsonl; no new finding kinds; additive format minors only; sealed ledger model_hashes must not move when keys are absent.
  
  Validation (executed): cargo test --workspace green; just ci green (fmt, clippy -D warnings, tests, wasm check, pretender check --mode gate); openspec validate --strict --all 13/13; ah check --changes add-declarative-keys --run-tests 107 passed; web npx vitest run 4/4; negative matrix green; cargo fmt/clippy rechecked after the last test edit; pretender gate exit 0 (whole tree).
  
  Unresolved: none blocking. Note: crates/catherine-cli/tests/negative_matrix.rs now sits exactly at the 400-line gate cap — the next row needs a split (a keys matrix binary) or a trim. Leftovers: the ddl-managed pretender gate is still --diff-only (upstream DDL-sl9 local-only); step 3+ unscheduled.
  
  Refs: openspec/changes/add-declarative-keys/{proposal,design,tasks}.md (Design Decisions 1-7 + close note), specs deltas (model-document, kernel-checks, findings, analysis), .espectacular/changes/add-declarative-keys/, crates/catherine-core/src/{model,findings,checks,parse/keys,parse/build,canonical/value}.rs, crates/catherine-analysis/src/schema_diff.rs, crates/catherine-core/tests/{keys,keys_checks}.rs, crates/catherine-analysis/tests/schema_diff_keys.rs, crates/catherine-cli/src/render{,.rs}/diff.rs, web/src/findings.ts.
- 2026-11-13T23:01:20Z [id:3bc45604610eda14fe7c564deddc59c81dbe141c737a8deabf988cc745b59509] (#declarative-keys) Format: 1
  Timestamp: 2026-11-13T23:05:00Z
  Repo: at/catherine (/var/home/sasha/para/areas/dev/gh/at/catherine)
  Branch: main
  HEAD: 16a25d6
  Session: omp session (harness exposed no id)
  Previous entry: 933104c300599e2d881ddd0dc8667d3bfd5500fe2ee2d665ac43492c3b7c5a59 (same session, same task; this snapshot is the same state re-published without blank lines because turu 0.11 ends an entry at the first blank line, leaving the body as freeform)
  Objective: step 2 of the post-granularity sequencing — declarative keys (injectivity constraints) plus the schema_diff enforced-set delta. Beads catherine-2s6 (feature) and catherine-obc (refactor) CLOSED; bd ready empty.
  Current phase: commit — everything landed and verified; nothing in flight.
  Current state: Format 1.3 — schema.keys [{object, arrows[], status accepted|proposed}], KNOWN_MINOR 3, new stable code invalid_key (undeclared object/arrow, arrow not sourced at the key object, empty or duplicate arrows; invalid_status reused); absent or empty keys are omitted from canonical bytes, so every pre-1.3 document keeps its exact hash (pinned sha256:fb033e3c fixture constants stayed green); schema_map target_schema validates keys by the same passes.
  Current state: findings/checks — FindingKind::Violation(ViolationWitness), an untagged union Equation(EquationWitness)|Key(KeyWitness{key, rows:[lo,hi]}); the equation JSON is byte-identical; key_findings groups rows by resolved arrow tuple, skips unresolved cells, emits one violation per unordered pair (ascending), accepted=executed / proposed=hypothesized, groups sorted defensively.
  Current state: analysis — EnforcedEquation became EnforcedConstraint::{Equation{lhs,rhs}, Key{object,arrows}}; the enforced set is accepted equations + accepted keys; is_clean_typed needed no change; added key -> accreted, dropped key -> weakened with the key witness through the existing bounded scan.
  Current state: CLI/web — key witness block (violation: key-<i> <Object>(<arrows>) plus the rows), diff added-key shape (untagged ConstraintView; equation shape unchanged), web findings.ts union branch + key fixture + regenerated generated types.
  Current state: decisions — CORR-003 closed: keys are arrow-only; column FDs need the value-object convention or rules --project until attribute columns (format 1.4). CORR-002 closed: witness union, not an overloaded EquationWitness (check output mixes both, so a reused equation index would be ambiguous).
  Current state: commits — 915f06c feature, 1e92e49 negative-matrix row (ill-formed key -> invalid_key, exit 1), 2d10292 web+types, 4ee1b09 docs, 06ad208 openspec change + contracts, 16a25d6 bead export. 55 commits ahead of origin/main; nothing pushed.
  Exact next action: none for this task. Unscheduled: step 3 (composite arrows, same 1.3 bump), step 4 (extraction bridge), step 5 (scoped quantification). Optional housekeeping: archive the open openspec changes (add-rules-projection, add-bogart-pilot 8/10, add-declarative-keys) — a deliberate later step, not started.
  Key decisions/constraints: conservative git profile — commits fine, NO push/deploy; bd changes need bd export + git add -f .beads/issues.jsonl; no new finding kinds; additive format minors only; sealed ledger model_hashes must not move when keys are absent.
  Validation (executed): cargo test --workspace green; just ci green (fmt, clippy -D warnings, tests, wasm check, pretender check --mode gate); openspec validate --strict --all 13/13; ah check --changes add-declarative-keys --run-tests 107 passed; web npx vitest run 4/4; negative matrix green; cargo fmt/clippy rechecked after the last test edit; pretender gate exit 0 (whole tree).
  Unresolved: none blocking. Note: crates/catherine-cli/tests/negative_matrix.rs now sits exactly at the 400-line gate cap — the next row needs a split (a keys matrix binary) or a trim. Leftovers: the ddl-managed pretender gate is still --diff-only (upstream DDL-sl9 local-only); step 3+ unscheduled.
  Refs: openspec/changes/add-declarative-keys/{proposal,design,tasks}.md (Design Decisions 1-7 + close note), its spec deltas (model-document, kernel-checks, findings, analysis), .espectacular/changes/add-declarative-keys/, crates/catherine-core/src/{model,findings,checks,parse/keys.rs,parse/build.rs,canonical/value.rs}, crates/catherine-analysis/src/schema_diff.rs, crates/catherine-core/tests/{keys.rs,keys_checks.rs}, crates/catherine-analysis/tests/schema_diff_keys.rs, crates/catherine-cli/src/render.rs + render/diff.rs, web/src/findings.ts. (supersedes 933104c300599e2d881ddd0dc8667d3bfd5500fe2ee2d665ac43492c3b7c5a59)
