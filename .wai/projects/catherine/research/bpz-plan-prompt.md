You are the PLANNER for catherine P0, ticket catherine-bpz, repo /var/home/sasha/para/areas/dev/gh/at/catherine (branch main, HEAD 9e52e78). Your only job: write the implementor brief. Do NOT implement anything.

READ FIRST (you have repo tools):
- openspec/changes/add-model-document-kernel/ — proposal.md, design.md, tasks.md, specs/
- bd show catherine-bpz
- docs/DESIGN.md sections 3 (reuse map) and 4.1 (kernel types)
- .wai/resources/pipelines/epic-orchestrator.toml (brief requirements)

WRITE the brief to .wai/projects/catherine/briefs/catherine-bpz.md with ALL of:
1. Ticket ID, desired outcome.
2. ## Why — delegation rationale (why a subagent implements instead of the lead session).
3. ## Completion criteria — runnable commands ONLY, exit 0 = done. Include: exact cargo test/proptest commands for catherine-core, `openspec validate --strict --all`, `ah check`. A criterion that cannot be executed is not a completion criterion.
4. ## Spawn model — implementor model glm-5.3-flash; compute the context ceiling as <= N% of the assumed window (state the window you assume, e.g. 1M) at spawn time; fixed token counts are invalid.
5. Out of scope — from ticket anti_goals: no lossless text writer; format_version never mutated in place; no schema-equivalence claims; ConstraintStatus semantics deferred to P1 (P0 fixes format field only: enum {proposed, accepted}, default accepted).
6. Exact files to create/touch: crates/catherine-core/** and openspec/changes/add-model-document-kernel/** only.
7. TDD red/green/refactor sequence with narrow + full test commands at each transition:
   RED: golden fixtures for malformed presentations (duplicate idents, unknown arrows, ill-typed equations, invalid ident charset, bad status value, unsupported version) with line/column error expectations; narrow command must fail for the expected missing behavior, not setup breakage.
   GREEN: minimal parser + canonical byte-stable serialization; canonical-JSON+sha256 hashing ported from wild formats.rs (adapt, do not author); round-trip proptest + row-id-canonicalization property.
   REFACTOR: shared error type; NOTICE attribution; no behavior expansion.
8. ro5u review as completion criterion: rule-of-5-universal review of the full diff, max_critical = 0; critical/high fixed, medium fixed-or-deferred-with-reason; tests added for behavior-impacting fixes.
9. Verifier evidence contract (wai-px4g): the verifier's review artifact must cite at least one read artifact path and include fresh test-run output (actual command + result) before recording CONFIRM; otherwise REVIEW_REQUIRED.
10. Follow-up threshold: stop and file follow-up tickets instead of expanding if the change grows beyond the 17 tasks in tasks.md.

HARD CONSTRAINTS: kernel crates genesis-free (genesis-vibes allowed in CLI crate only, stable modules only); adapt specodelic acset-core contracts, do not author from scratch; proptest not insta-snapshots; Apache-2.0 with NOTICE preserved on ported code; executed checks are the product (E3).

Finish by printing the brief path and a 5-line summary of the plan. Print nothing else after that.