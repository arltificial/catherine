# ro5u review — catherine-0g1 (CQL/FDM positioning + patent note)

Lead-side review. Deliverable: PROPOSAL §7 — a capability head-to-head
against CQL (the functorial data model) and a patent-exposure assessment.

Five passes:

1. Correctness: the section states the shared formalism (schemas as
   categories, instances as functors to Set, Δ migration) without
   claiming it as catherine's invention; the table's parity rows are
   narrow (equation enforcement, Δ) and the differences are the ones the
   repo actually implements (named witnesses, underconstraint as a
   domain question, bounded `UNKNOWN {scope}`, epistemic labels,
   JSON/CLI/wasm packaging). CQL's own enforcement is not misdescribed
   as documentation-only — that criticism stays scoped to Catlab.
2. Contract drift: docs only; no capability claim changes elsewhere.
   "Any public novelty claim cites this section" is stated as the rule.
3. Tests: N/A (positioning); the patent facts are grounded to the USPTO
   records found by search — US 11,256,672 B1 (2022-02-22), US
   11,720,535 B2 (2023-08-08), published application US 2024/0119034 A1,
   assignee Conexus AI. No numbers were invented.
4. Negative matrix: the assessment is explicitly not legal advice, says
   independent implementation does not settle risk, names the missing
   patent grant, and gates commercial/hosted/redistribution on counsel
   with an escalation rule — it does not self-clear.
5. Regression risk: none (one docs section).

Findings + dispositions:

- [NOTE, ACCEPTED] The patent analysis is an engineering assessment, not
  a freedom-to-operate opinion; the section says so and routes the
  decision to counsel/human.
- [NOTE, RECORDED] The user-facing novelty boundary is now citable:
  witnesses + ambiguity-as-output + packaging, not "we invented
  functorial migration".

max_critical = 0. Verdict: PASS.