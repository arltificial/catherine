---
tags: [pipeline-run:epic-orchestrator-2026-11-11-catherine-p4-bounded-analysis-catherine-1ay, pipeline-step:ship]
---

catherine-7e8 adjudication: parallel-path related() stays direct-only in v1 — accepted equation chains (p=q, q=r) do not suppress the (p,r) domain question even though rewriting derives p=r. Rationale: question-raiser semantics (DESIGN §4.2, noise over silence); congruence closure over the accepted set is a new analysis capability beyond v1, not a bug fix. Documented in parallel.rs module header + analysis README limits section; pinned by transitive_equation_chain_does_not_suppress_the_end_pair_question.
