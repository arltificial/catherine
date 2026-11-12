# ro5u review — catherine-7e8 (adjudicated limit: docs + ratchet test)

Adjudication (recorded via `wai add design`): `related()` stays
direct-only in v1. Accepted chains p=q, q=r do not suppress the (p,r)
question even though rewriting derives p=r. Congruence closure is a new
analysis capability beyond v1 (DESIGN non-goals: no prover), not a bug
fix — the ticket's own framing ("likely intentional for v1,
question-raiser, noise over silence, but undocumented") is accepted and
the gap closed by documentation + executable pinning.

Diff: parallel.rs module-header limit paragraph + related() doc;
README "Underconstraint relating is direct-only" limit bullet;
new fixture `transitive_chain_schema` + test
`transitive_equation_chain_does_not_suppress_the_end_pair_question`
(asserts exactly the (p,r) pair raised, hypothesized label, question
text); CHANGELOG Changed entry. No openspec change dir → artifact
recorded here (deviation noted in orchestrator report).

Five passes:

1. Correctness: test pins actual behavior (direct-only related(); level-1
   pairs (p,q),(q,r) suppressed, (p,r) raised; no composites within cap).
2. Contract drift (class B): docs now match code; the claim is
   scenario-backed by an executed test (za6 discipline satisfied).
3. Tests: ratchet test added for the documented limit (checklist:
   behavior-impacting fix → test — here the "fix" is the documentation,
   and it carries executable evidence per the repo's ratchet practice).
4. Negative matrix: no flag/surface changed. N/A.
5. Regression risk: none — no runtime behavior changed; test is
   deterministic and isolated.

Findings: none ≥ MEDIUM. max_critical = 0.

Verdict: PASS.
