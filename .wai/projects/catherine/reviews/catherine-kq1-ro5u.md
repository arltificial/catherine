# ro5u review — catherine-kq1 (granularity rule + re-model protocol)

Lead-side review. Deliverables: the PROPOSED granularity rule in
`docs/USING.md` (Recipe A step 1) and `docs/ref/ct_methodology_playbook.md`
§2, the calibration/protocol note
`.wai/projects/catherine/research/2026-11-13-granularity-rule-calibration.md`,
and the follow-up ticket `catherine-8v9` for the bogart-side execution.
Scope split is explicit: this ticket's catherine-side deliverables are done;
the second-domain measurement is filed, not silently dropped.

Five passes:

1. Correctness: the rule is stated as PROPOSED everywhere, with the
   arbitrary one-third threshold and the 2/69 calibration disclosed. The
   pilot retro-audit maps every arrow of the three committed models to a
   DDL column and reports the fail ratios (model1 2/3, model2 0/3, model3
   4/5) — consistent with model3 being a query-plan model. The audit's
   limits (hand-read DDL, no SQL executed, pilots not designed for this
   test) are stated, so it calibrates rather than validates.
2. Contract drift: docs only; no claim of EXECUTED for the rule. The
   playbook (vendored methodology) gains a clearly marked PROPOSED rule —
   the ticket asked for it there.
3. Tests: the re-model protocol is falsifiable (compare real/noise with the
   pilot's 2/69; a probe entry requires a committed baseline arm). It is
   filed as `catherine-8v9` because it is bogart-side and the dossier lives
   in pinky; that is the honest hand-off, not a pretend execution.
4. Negative matrix: the falsifier is written down, and the failure branch
   keeps the rule PROPOSED with a negative result attached. No fabrication
   of a second-domain measurement.
5. Regression risk: docs only.

Findings + dispositions:

- [NOTE, ACCEPTED] The follow-up ticket carries the execution. Closing kq1
   with a filed follow-up (rather than leaving it open) matches the
   project's per-ticket discipline; the remaining acceptance criterion is
   visible in bd.
- [NOTE, RECORDED] Threshold calibration is one domain (bogart); the docs
   say so and forbid treating it as a validated cutoff.

max_critical = 0. Verdict: PASS (catherine-side; execution delegated to
`catherine-8v9`).